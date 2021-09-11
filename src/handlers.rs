/// API handlers, the ends of each filter chain
use aes::Aes128;
use askama::Template;
use blake2::{Blake2s, Digest};
use block_modes::block_padding::Pkcs7;
use block_modes::{BlockMode, Cbc};
use chrono::Utc;
use jsonwebtoken::errors::ErrorKind;
use jsonwebtoken::{decode, Algorithm, DecodingKey, TokenData, Validation};
use lazy_static::lazy_static;
use log::{debug, warn};
use math;
use md5::Md5;
use parking_lot::RwLockUpgradableReadGuard;
use rsa::{PaddingScheme, RSAPrivateKey};
use serde::Serialize;
use sha2::Sha256;
use std::collections::{HashMap, HashSet};
use std::convert::Infallible;
use std::fs;
use std::hash::Hash;
use warp::{http::StatusCode, reply};

use crate::PRIVATE_KEY;

// Valid blocks should have this many transactions
const BLOCK_TRANSACTION_COUNT: u8 = 8;
// Inital registration bonus
const REGISTER_BONUS: u16 = 40;
// Coinbase reward
const BLOCK_REWARD: u16 = 3;
// Transaction amount limit
const TX_UPPER_LIMIT: u16 = 10;
const TX_LOWER_LIMIT: u16 = 1;
// Transaction traffic reward
const TX_TRAFFIC_REWARD: u16 = 1;
// Staking reward
const STAKING_REWARD: f64 = 0.1; // Percentage

// Encryption primitive
type Aes128Cbc = Cbc<Aes128, Pkcs7>;

#[derive(Serialize, Debug)]
struct GradeCoinResponse {
    res: ResponseType,
    message: String,
}

#[derive(Debug, Serialize)]
enum ResponseType {
    Success,
    Error,
}

use crate::schema::{
    AuthRequest, Block, Claims, Db, InitialAuthRequest, MetuId, NakedBlock, Transaction, User,
    UserAtRest,
};

const BEARER: &str = "Bearer ";

lazy_static! {
    static ref DER_ENCODED: String = PRIVATE_KEY
        .lines()
        .filter(|line| !line.starts_with('-'))
        .fold(String::new(), |mut data, line| {
            data.push_str(&line);
            data
        });

    // base64(der(pem))
    // Our private key is saved in PEM (base64) format
    static ref DER_BYTES: Vec<u8> = base64::decode(&*DER_ENCODED).expect("failed to decode base64 content");
    static ref GRADECOIN_PRIVATE_KEY: RSAPrivateKey = RSAPrivateKey::from_pkcs1(&DER_BYTES).expect("failed to parse key");
}

/// POST request to /register endpoint
///
/// Lets a [`User`] (=student) to authenticate themselves to the system
/// This `request` can be rejected if the payload is malformed (=not authenticated properly) or if
/// the [`AuthRequest.user_id`] of the `request` is not in the list of users that can hold a Drocoin account
///
/// # Authentication Process
/// - Drocoin's Public Key (`drocoin_public_key`) is listed on moodle.
/// - Drocoin's Private Key (`drocoin_private_key`) is loaded here
///
/// - Student picks a short temporary key (`k_temp`)
/// - Creates a JSON object (`auth_plaintext`) with their `metu_id` and `public key` in base64 (PEM) format (`S_PK`):
/// {
///     student_id: "e12345",
///     passwd: "15 char secret"
///     public_key: "---BEGIN PUBLIC KEY..."
/// }
///
/// - Encrypts the serialized string of `auth_plaintext` with 128 bit block AES in CBC mode with Pkcs7 padding using the temporary key (`k_temp`), the result is `auth_ciphertext`
/// - The temporary key student has picked `k_temp` is encrypted using RSA with OAEP padding scheme
/// using sha256 with `drocoin_public_key`, giving us `key_ciphertext`
/// - The payload JSON object (`auth_request`) can be JSON serialized now:
/// {
///     c: "auth_ciphertext"
///     key: "key_ciphertext"
/// }
///
/// ## Drocoin Side
///
/// - Upon receiving, we first RSA decrypt with OAEP padding scheme using SHA256 with `drocoin_private_key` as the key and auth_request.key `key` as the ciphertext, receiving `temp_key` (this is the temporary key chosen by student)
/// - With `temp_key`, we can AES 128 Cbc Pkcs7 decrypt the `auth_request.c`, giving us
/// auth_plaintext
/// - The `auth_plaintext` String can be deserialized to [`AuthRequest`]
/// - We then verify the payload and calculate the User fingerprint
/// - Finally, create the new [`User`] object, insert to users HashMap `<fingerprint, User>`
///
pub async fn authenticate_user(
    request: InitialAuthRequest,
    db: Db,
) -> Result<impl warp::Reply, warp::Rejection> {
    debug!("POST /register, authenticate_user() is handling");

    // In essence PEM files are just base64 encoded versions of the DER encoded data.
    // ~tls.mbed.org

    let padding = PaddingScheme::new_oaep::<sha2::Sha256>();

    // Peel away the base64 layer from "key" field
    let key_ciphertext = match base64::decode(&request.key) {
        Ok(c) => c,
        Err(err) => {
            debug!(
                "\"key\" field of initial auth request was not base64 encoded: {}, {}",
                &request.key, err
            );

            let res_json = warp::reply::json(&GradeCoinResponse {
                res: ResponseType::Error,
                message: format!(
                    "\"key\" field of initial auth request was not base64 encoded: {}, {}",
                    &request.key, err
                ),
            });

            return Ok(warp::reply::with_status(res_json, StatusCode::BAD_REQUEST));
        }
    };

    // Decrypt the "key" field using Drocoin's private key
    let temp_key = match GRADECOIN_PRIVATE_KEY.decrypt(padding, &key_ciphertext) {
        Ok(k) => k,
        Err(err) => {
            debug!(
                "Failed to decrypt ciphertext of the key with Drocoin's public key: {}. Key was {:?}",
                err, &key_ciphertext
            );

            let res_json = warp::reply::json(&GradeCoinResponse {
                res: ResponseType::Error,
                message: "Failed to decrypt the 'key_ciphertext' field of the auth request"
                    .to_owned(),
            });

            return Ok(warp::reply::with_status(res_json, StatusCode::BAD_REQUEST));
        }
    };

    // Peel away the base64 from the iv field as well
    let byte_iv = match base64::decode(&request.iv) {
        Ok(iv) => iv,
        Err(err) => {
            debug!(
                "\"iv\" field of initial auth request was not base64 encoded: {}, {}",
                &request.iv, err
            );

            let res_json = warp::reply::json(&GradeCoinResponse {
                res: ResponseType::Error,
                message: format!(
                    "\"iv\" field of initial auth request was not base64 encoded: {}, {}",
                    &request.iv, err
                ),
            });

            return Ok(warp::reply::with_status(res_json, StatusCode::BAD_REQUEST));
        }
    };

    // we have key and iv, time to decrypt the "c" field, first prepare the decryptor
    let cipher = match Aes128Cbc::new_var(&temp_key, &byte_iv) {
        Ok(c) => c,
        Err(err) => {
            debug!(
                "Could not create a cipher from temp_key and request.iv {:?}, {}, {}",
                &temp_key, &request.iv, err
            );

            let res_json = warp::reply::json(&GradeCoinResponse {
                res: ResponseType::Error,
                message: format!(
                    "Could not create a cipher from given 'temp_key': {:?} and 'IV': {}",
                    &temp_key, &request.iv
                ),
            });

            return Ok(warp::reply::with_status(res_json, StatusCode::BAD_REQUEST));
        }
    };

    // peel away the base64 from the auth packet
    let auth_packet = match base64::decode(&request.c) {
        Ok(a) => a,
        Err(err) => {
            debug!(
                "\"c\" field of initial auth request was not base64 encoded: {}, {}",
                &request.c, err
            );

            let res_json = warp::reply::json(&GradeCoinResponse {
                res: ResponseType::Error,
                message: format!(
                    "\"c\" field of initial auth request was not base64 encoded: {}, {}",
                    &request.c, err
                ),
            });

            return Ok(warp::reply::with_status(res_json, StatusCode::BAD_REQUEST));
        }
    };

    // c field was properly base64 encoded, now available in auth_packet
    // decryptor was setup properly, with the correct lenght key
    let mut buf = auth_packet.to_vec();
    let auth_plaintext = match cipher.decrypt(&mut buf) {
        Ok(p) => p,
        Err(err) => {
            println!(
                "auth request (c) did not decrypt correctly {:?} {}",
                &buf, err
            );

            let res_json = warp::reply::json(&GradeCoinResponse {
                res: ResponseType::Error,
                message: "Failed to decrypt the 'c' field of the auth request, 'iv' and 'k_temp' were valid so far though"
                    .to_owned(),
            });

            return Ok(warp::reply::with_status(res_json, StatusCode::BAD_REQUEST));
        }
    };

    // we have a decrypted c field, create a string from the bytes mess
    let utf8_auth_plaintext = match String::from_utf8(auth_plaintext.to_vec()) {
        Ok(text) => text,
        Err(err) => {
            debug!(
                "Auth plaintext did not convert into utf8 {:?} {}",
                &auth_plaintext, err
            );

            let res_json = warp::reply::json(&GradeCoinResponse {
                res: ResponseType::Error,
                message: "P_AR couldn't get converted to UTF-8, please check your encoding"
                    .to_owned(),
            });

            return Ok(warp::reply::with_status(res_json, StatusCode::BAD_REQUEST));
        }
    };

    // finally create an AuthRequest object from the plaintext
    let request: AuthRequest = match serde_json::from_str(&utf8_auth_plaintext) {
        Ok(req) => req,
        Err(err) => {
            debug!(
                "Auth plaintext did not serialize correctly {:?} {}",
                &utf8_auth_plaintext, err
            );

            let res_json = warp::reply::json(&GradeCoinResponse {
                res: ResponseType::Error,
                message: "The P_AR JSON did not serialize correctly, did it include all 3 fields 'student_id', 'passwd' and 'public_key'?".to_owned(),
            });

            return Ok(warp::reply::with_status(res_json, StatusCode::BAD_REQUEST));
        }
    };

    // is the student in AuthRequest privileged?
    let privileged_student_id =
        match MetuId::new(request.student_id.clone(), request.passwd.clone()) {
            Some(id) => id,
            None => {
                debug!(
                    "Someone tried to auth with invalid credentials: {} {}",
                    &request.student_id, &request.passwd
                );
                let res_json = warp::reply::json(&GradeCoinResponse {
                res: ResponseType::Error,
                message:
                    "The credentials given ('student_id', 'passwd') cannot hold a Drocoin account"
                        .to_owned(),
            });

                return Ok(warp::reply::with_status(res_json, StatusCode::BAD_REQUEST));
            }
        };

    // Students should be able to authenticate once
    {
        let userlist = db.users.read();

        for (_, user) in userlist.iter() {
            if user.user_id == privileged_student_id {
                let res_json = warp::reply::json(&GradeCoinResponse {
                res: ResponseType::Error,
                message:
                    "This user is already authenticated, do you think this is a mistake? Contact me"
                        .to_owned(),
            });
                return Ok(warp::reply::with_status(res_json, StatusCode::BAD_REQUEST));
            }
        }
    }

    // We're using this as the validator instead of anything reasonable
    if DecodingKey::from_rsa_pem(request.public_key.as_bytes()).is_err() {
        let res_json = warp::reply::json(&GradeCoinResponse {
            res: ResponseType::Error,
            message: "The RSA 'public_key' in 'P_AR' is not in valid PEM format".to_owned(),
        });

        return Ok(warp::reply::with_status(res_json, StatusCode::BAD_REQUEST));
    }

    let fingerprint = format!("{:x}", Sha256::digest(&request.public_key.as_bytes()));

    let new_user = User {
        user_id: privileged_student_id,
        public_key: request.public_key,
        balance: REGISTER_BONUS,
        is_bot: false,
    };

    debug!("NEW USER: {:?}", &new_user);

    // save the user to disk
    let user_at_rest_json = serde_json::to_string(&UserAtRest {
        fingerprint: fingerprint.clone(),
        user: User {
            user_id: new_user.user_id.clone(),
            public_key: new_user.public_key.clone(),
            balance: new_user.balance,
            is_bot: false,
        },
    })
    .unwrap();

    fs::write(format!("users/{}.guy", new_user.user_id), user_at_rest_json).unwrap();

    let mut userlist = db.users.write();
    userlist.insert(fingerprint.clone(), new_user);

    let res_json = warp::reply::json(&GradeCoinResponse {
        res: ResponseType::Success,
        message: format!(
            "You have authenticated to use Drocoin with identifier {}",
            fingerprint
        ),
    });

    Ok(warp::reply::with_status(res_json, StatusCode::CREATED))
}

/// GET /transaction
/// Returns JSON array of transactions
pub async fn list_transactions(db: Db) -> Result<impl warp::Reply, Infallible> {
    let mut result = HashMap::new();

    let transactions = db.pending_transactions.read();

    for (fp, tx) in transactions.iter() {
        result.insert(fp, tx);
    }

    Ok(reply::with_status(reply::json(&result), StatusCode::OK))
}

/// POST /block
///
/// Proposes a new block for the next round.
/// Can reject the block
///
/// The proposer has to put their transaction as the first transaction of the [`transaction_list`].
/// This is the analogue of `coinbase` in Bitcoin works
///
/// The `coinbase` transaction also gets something for their efforts.
pub async fn propose_block(
    new_block: Block,
    token: String,
    db: Db,
) -> Result<impl warp::Reply, warp::Rejection> {
    warn!("New block proposal: {:?}", &new_block);

    // Check if there are enough transactions in the block
    if new_block.transaction_list.len() < BLOCK_TRANSACTION_COUNT as usize {
        debug!(
            "{} transactions offered, needed {}",
            new_block.transaction_list.len(),
            BLOCK_TRANSACTION_COUNT
        );
        let res_json = warp::reply::json(&GradeCoinResponse {
            res: ResponseType::Error,
            message: format!(
                "There should be at least {} transactions in the block",
                BLOCK_TRANSACTION_COUNT
            ),
        });

        return Ok(warp::reply::with_status(res_json, StatusCode::BAD_REQUEST));
    }

    // proposer (first transaction fingerprint) checks
    let pending_transactions = db.pending_transactions.upgradable_read();

    // we get the proposers fingerprint by finding the transaction (id) then extracting the source
    let internal_user_fingerprint = match pending_transactions.get(&new_block.transaction_list[0]) {
        Some(coinbase) => &coinbase.source,
        None => {
            debug!(
                "Transaction with id {} is not found in the pending_transactions",
                new_block.transaction_list[0]
            );

            let res_json = warp::reply::json(&GradeCoinResponse {
                res: ResponseType::Error,
                message: "First transaction in the block is not found in the system".to_owned(),
            });

            return Ok(warp::reply::with_status(res_json, StatusCode::BAD_REQUEST));
        }
    };

    let users_store = db.users.upgradable_read();

    // this probably cannot fail, if the transaction is valid then it must've been checked already
    let internal_user = match users_store.get(internal_user_fingerprint) {
        Some(existing_user) => existing_user,
        None => {
            debug!(
                "User with public key signature {:?} is not found in the database",
                new_block.transaction_list[0]
            );

            let res_json = warp::reply::json(&GradeCoinResponse {
                res: ResponseType::Error,
                message: "User with that public key signature is not found in the database"
                    .to_owned(),
            });

            return Ok(warp::reply::with_status(res_json, StatusCode::BAD_REQUEST));
        }
    };

    let proposer_public_key = &internal_user.public_key;

    // JWT Check
    let token_payload = match authorize_proposer(token, &proposer_public_key) {
        Ok(data) => data,
        Err(below) => {
            debug!("Something went wrong with the JWT {:?}", below);

            let res_json = warp::reply::json(&GradeCoinResponse {
                res: ResponseType::Error,
                message: below,
            });

            return Ok(warp::reply::with_status(res_json, StatusCode::BAD_REQUEST));
        }
    };

    // Block hash check
    if token_payload.claims.tha != new_block.hash {
        debug!(
            "The Hash of the block {:?} did not match the hash given in jwt {:?}",
            new_block.hash, token_payload.claims.tha
        );
        let res_json = warp::reply::json(&GradeCoinResponse {
            res: ResponseType::Error,
            message: "The hash of the block did not match the hash given in JWT tha field"
                .to_owned(),
        });

        return Ok(warp::reply::with_status(res_json, StatusCode::BAD_REQUEST));
    }

    if !has_unique_elements(&new_block.transaction_list) {
        debug!("Block contains duplicate transactions!");
        let res_json = warp::reply::json(&GradeCoinResponse {
            res: ResponseType::Error,
            message: "Block cannot contain duplicate transactions".to_owned(),
        });

        return Ok(warp::reply::with_status(res_json, StatusCode::BAD_REQUEST));
    }

    // Are transactions in the block valid?
    for transaction_hash in new_block.transaction_list.iter() {
        if !pending_transactions.contains_key(transaction_hash) {
            let res_json = warp::reply::json(&GradeCoinResponse {
                res: ResponseType::Error,
                message: "Block contains an unknown transaction".to_owned(),
            });

            return Ok(warp::reply::with_status(res_json, StatusCode::BAD_REQUEST));
        }
    }

    // hash the block ourselves to double check
    let naked_block = NakedBlock {
        transaction_list: new_block.transaction_list.clone(),
        nonce: new_block.nonce,
        timestamp: new_block.timestamp,
    };

    let naked_block_flat = serde_json::to_vec(&naked_block).unwrap();

    let hashvalue = Blake2s::digest(&naked_block_flat);
    let hash_string = format!("{:x}", hashvalue);

    // Does the hash claimed in block match with the actual hash?
    if hash_string != new_block.hash {
        debug!("request was not telling the truth, hash values do not match");
        let res_json = warp::reply::json(&GradeCoinResponse {
            res: ResponseType::Error,
            message: "Given hash value does not match the actual block hash".to_owned(),
        });

        return Ok(warp::reply::with_status(res_json, StatusCode::BAD_REQUEST));
    }

    // Are the 6 leftmost characters (=24 bits) zero?
    let should_zero = hashvalue[0] as i32 + hashvalue[1] as i32 + hashvalue[2] as i32;

    if should_zero != 0 {
        debug!("the hash does not have 6 rightmost zero bits");
        let res_json = warp::reply::json(&GradeCoinResponse {
            res: ResponseType::Error,
            message: "Given block hash is larger than target value".to_owned(),
        });

        return Ok(warp::reply::with_status(res_json, StatusCode::BAD_REQUEST));
    }

    // All clear, block accepted!
    warn!("ACCEPTED BLOCK {:?}", new_block);

    // Scope the read guards
    {
        let mut pending_transactions = RwLockUpgradableReadGuard::upgrade(pending_transactions);
        let mut users_store = RwLockUpgradableReadGuard::upgrade(users_store);

        // Reward the block proposer
        let coinbase_fingerprint = new_block.transaction_list.get(0).unwrap();

        if let Some(coinbase_user) = users_store.get_mut(coinbase_fingerprint) {
            coinbase_user.balance += BLOCK_REWARD;
        }

        let holding: HashMap<String, Transaction> = HashMap::new();

        // Play out the transactions
        for fingerprint in new_block.transaction_list.iter() {
            if let Some(transaction) = pending_transactions.remove(fingerprint) {
                let source = &transaction.source;
                let target = &transaction.target;
                let is_source_bot = users_store.get(source).unwrap().is_bot;

                if let Some(from) = users_store.get_mut(source) {
                    from.balance -= transaction.amount - TX_TRAFFIC_REWARD;
                }

                if let Some(to) = users_store.get_mut(target) {
                    to.balance += transaction.amount;
                    if is_source_bot {
                        // Add staking reward
                        to.balance +=
                            math::round::ceil((transaction.amount as f64) * STAKING_REWARD, 0)
                                as u16;
                    }
                }

                // if the receiver is a bot, they will reciprocate
                if users_store.get(target).unwrap().is_bot {
                    let transaction_id = calculate_transaction_id(target, source);
                    pending_transactions.insert(
                        transaction_id,
                        Transaction {
                            source: target.to_owned(),
                            target: source.to_owned(),
                            amount: transaction.amount,
                            timestamp: Utc::now().naive_local(),
                        },
                    );
                }
            }
        }

        for (fp, tx) in holding.iter() {
            pending_transactions.insert(fp.to_owned(), tx.to_owned());
        }

        // just update everyone's .guy file
        for (fp, guy) in users_store.iter() {
            if !guy.is_bot {
                let user_at_rest_json = serde_json::to_string(&UserAtRest {
                    fingerprint: fp.clone(),
                    user: User {
                        user_id: guy.user_id.clone(),
                        public_key: guy.public_key.clone(),
                        balance: guy.balance,
                        is_bot: false,
                    },
                })
                .unwrap();
                fs::write(format!("users/{}.guy", guy.user_id), user_at_rest_json).unwrap();
            }
        }
    }

    let block_json = serde_json::to_string(&new_block).unwrap();

    fs::write(
        format!("blocks/{}.block", new_block.timestamp.timestamp()),
        block_json,
    )
    .unwrap();

    {
        let mut blockchain = db.blockchain.write();
        *blockchain = new_block;
    }

    Ok(warp::reply::with_status(
        warp::reply::json(&GradeCoinResponse {
            res: ResponseType::Success,
            message: "Block accepted, coinbase reward awarded".to_owned(),
        }),
        StatusCode::CREATED,
    ))
}

/// POST /transaction
///
/// Handles the new transaction requests
/// Can reject the block if;
/// # Arguments
/// * `new_transaction` - Valid JSON of a [`Transaction`]
/// * `token` - An Authorization header value such as `Bearer aaa.bbb.ccc`
/// * `db` - Global [`Db`] instance
///
pub async fn propose_transaction(
    new_transaction: Transaction,
    token: String,
    db: Db,
) -> Result<impl warp::Reply, warp::Rejection> {
    warn!("New transaction proposal: {:?}", &new_transaction);

    let users_store = db.users.read();

    // Is this transaction from an authorized source?
    let internal_user = match users_store.get(&new_transaction.source) {
        Some(existing_user) => existing_user,
        None => {
            debug!(
                "User with public key signature {:?} is not found in the database",
                new_transaction.source
            );

            return Ok(warp::reply::with_status(
                warp::reply::json(&GradeCoinResponse {
                    res: ResponseType::Error,
                    message: "User with the given public key signature is not authorized"
                        .to_owned(),
                }),
                StatusCode::BAD_REQUEST,
            ));
        }
    };

    if internal_user.is_bot {
        debug!("Someone tried to send as the bot");

        return Ok(warp::reply::with_status(
            warp::reply::json(&GradeCoinResponse {
                res: ResponseType::Error,
                message: "Don's send transactions on behalf of bots".to_owned(),
            }),
            StatusCode::BAD_REQUEST,
        ));
    }

    // `internal_user` is an authenticated student and not a bot, can propose

    // This public key was already written to the database, we can panic if it's not valid at
    // *this* point
    let proposer_public_key = &internal_user.public_key;

    let token_payload = match authorize_proposer(token, &proposer_public_key) {
        Ok(data) => data,
        Err(below) => {
            debug!("JWT Error: {:?}", below);
            return Ok(warp::reply::with_status(
                warp::reply::json(&GradeCoinResponse {
                    res: ResponseType::Error,
                    message: below,
                }),
                StatusCode::BAD_REQUEST,
            ));
        }
    };

    // is the target of the transaction in the system?
    if !users_store.contains_key(&new_transaction.target) {
        debug!(
            "Target of the transaction is not in the system {}",
            new_transaction.target
        );

        return Ok(warp::reply::with_status(
            warp::reply::json(&GradeCoinResponse {
                res: ResponseType::Error,
                message: format!(
                    "Target of the transaction {} is not found in the system",
                    new_transaction.target
                ),
            }),
            StatusCode::BAD_REQUEST,
        ));
    }

    let transaction_id = calculate_transaction_id(&new_transaction.source, &new_transaction.target);

    // OLD: Does this user have a pending transaction?
    // NEW: Is this source:target pair unqiue?
    {
        let transactions = db.pending_transactions.read();
        debug!(
            "This is a transaction from {} to {}",
            new_transaction.source, new_transaction.target,
        );

        if transactions.contains_key(&transaction_id) {
            debug!(
                "this source/target combination {} already has a pending transaction",
                transaction_id
            );

            return Ok(warp::reply::with_status(
                warp::reply::json(&GradeCoinResponse {
                    res: ResponseType::Error,
                    message: "This user already has another pending transaction".to_owned(),
                }),
                StatusCode::BAD_REQUEST,
            ));
        }
    }

    if new_transaction.source == new_transaction.target {
        debug!("transaction source and target are the same",);

        return Ok(warp::reply::with_status(
            warp::reply::json(&GradeCoinResponse {
                res: ResponseType::Error,
                message: "transaction to yourself, you had to try didn't you? :)".to_owned(),
            }),
            StatusCode::BAD_REQUEST,
        ));
    }

    // Is transaction amount within bounds
    if new_transaction.amount > TX_UPPER_LIMIT || new_transaction.amount < TX_LOWER_LIMIT {
        debug!(
            "Transaction amount is not between {} and {}, was {}",
            TX_LOWER_LIMIT, TX_UPPER_LIMIT, new_transaction.amount
        );
        return Ok(warp::reply::with_status(
            warp::reply::json(&GradeCoinResponse {
                res: ResponseType::Error,
                message: format!(
                    "Transaction amount should be between {} and {}",
                    TX_LOWER_LIMIT, TX_UPPER_LIMIT
                ),
            }),
            StatusCode::BAD_REQUEST,
        ));
    }

    // check if user can afford the transaction
    if internal_user.balance < new_transaction.amount {
        debug!(
            "User does not have enough balance ({}) for this TX {}",
            internal_user.balance, new_transaction.amount
        );
        return Ok(warp::reply::with_status(
            warp::reply::json(&GradeCoinResponse {
                res: ResponseType::Error,
                message: "User does not have enough balance in their account for this transaction"
                    .to_owned(),
            }),
            StatusCode::BAD_REQUEST,
        ));
    }

    // this transaction was already checked for correctness at custom_filters, we can panic here if
    // it has been changed since

    let serd_tx = serde_json::to_string(&new_transaction).unwrap();

    debug!("Taking the hash of {}", serd_tx);

    let hashed_transaction = Md5::digest(&serd_tx.as_bytes());
    if token_payload.claims.tha != format!("{:x}", hashed_transaction) {
        return Ok(warp::reply::with_status(
            warp::reply::json(&GradeCoinResponse {
                res: ResponseType::Error,
                message: "The hash of the transaction did not match the hash given in JWT"
                    .to_owned(),
            }),
            StatusCode::BAD_REQUEST,
        ));
    }

    warn!("ACCEPTED TRANSACTION {:?}", new_transaction);

    let mut transactions = db.pending_transactions.write();

    transactions.insert(transaction_id, new_transaction);

    Ok(warp::reply::with_status(
        warp::reply::json(&GradeCoinResponse {
            res: ResponseType::Success,
            message: "Transaction accepted".to_owned(),
        }),
        StatusCode::CREATED,
    ))
}

/// GET /block
/// Returns the last block's JSON
/// Cannot fail
/// Mostly around for debug purposes
pub async fn list_blocks(db: Db) -> Result<impl warp::Reply, Infallible> {
    debug!("GET /block, list_blocks() is handling");

    let block = db.blockchain.read();

    Ok(reply::with_status(reply::json(&*block), StatusCode::OK))
}

/// Handles the JWT Authorization
///
/// *[`jwt_token`]: The raw JWT token, "Bearer aaa.bbb.ccc"
/// *[`user_pem`]: User Public Key, "BEGIN RSA"
/// NOT async, might look into it if this becomes a bottleneck
fn authorize_proposer(jwt_token: String, user_pem: &str) -> Result<TokenData<Claims>, String> {
    // Throw away the "Bearer " part
    let raw_jwt = jwt_token.trim_start_matches(BEARER).to_owned();

    // Extract a jsonwebtoken compatible decoding_key from user's public key
    let decoding_key = match DecodingKey::from_rsa_pem(user_pem.as_bytes()) {
        Ok(key) => key,
        Err(j) => {
            warn!(
                "given RSA key {} is invalid, we should crash and burn here {:?}",
                user_pem, j
            );
            return Err(String::from("This User's RSA key is invalid"));
        }
    };

    // Extract the payload inside the JWT
    let token_payload =
        match decode::<Claims>(&raw_jwt, &decoding_key, &Validation::new(Algorithm::RS256)) {
            Ok(decoded) => decoded,
            Err(err) => match *err.kind() {
                ErrorKind::InvalidToken => {
                    debug!("raw_jwt={:?} was malformed err={:?}", raw_jwt, err);
                    return Err(String::from("Invalid Token"));
                }
                ErrorKind::InvalidRsaKey => {
                    debug!("The RSA key does not have a valid format, {:?}", err);
                    return Err(String::from("The RSA key does not have a valid format"));
                }
                ErrorKind::ExpiredSignature => {
                    debug!("this token has expired {:?}", err);
                    return Err(String::from("This token has expired"));
                }
                _ => {
                    warn!(
                        "AN UNSPECIFIED ERROR from token: {}\nerr: {:?} key was {}",
                        raw_jwt, err, user_pem
                    );
                    return Err(format!("JWT Error: {}", err));
                }
            },
        };

    Ok(token_payload)
}

fn calculate_transaction_id(source: &str, target: &str) -> String {
    let long_fingerprint = format!("{}{}", source, target);
    let id = format!("{:x}", Sha256::digest(long_fingerprint.as_bytes()));
    id
}

#[derive(Template)]
#[template(path = "list.html")]
struct UserTemplate<'a> {
    users: &'a Vec<DisplayUsers>,
}

struct DisplayUsers {
    fingerprint: String,
    balance: u16,
    is_bot: bool,
}

pub async fn user_list_handler(db: Db) -> Result<impl warp::Reply, warp::Rejection> {
    let users = db.users.read();
    let mut sane_users = Vec::new();

    for (fingerprint, user) in users.iter() {
        sane_users.push(DisplayUsers {
            fingerprint: fingerprint.to_owned(),
            balance: user.balance,
            is_bot: user.is_bot,
        });
    }

    let template = UserTemplate { users: &sane_users };
    let res = template.render().unwrap();
    Ok(warp::reply::html(res))
}

fn has_unique_elements<T>(iter: T) -> bool
where
    T: IntoIterator,
    T::Item: Eq + Hash,
{
    let mut uniq = HashSet::new();
    iter.into_iter().all(move |x| uniq.insert(x))
}
