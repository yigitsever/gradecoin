/// API handlers, the ends of each filter chain
use aes::Aes128;
use base64;
use blake2::{Blake2s, Digest};
use block_modes::block_padding::Pkcs7;
use block_modes::{BlockMode, Cbc};
use jsonwebtoken::errors::ErrorKind;
use jsonwebtoken::{decode, Algorithm, DecodingKey, TokenData, Validation};
use log::{debug, warn};
use md5::Md5;
use parking_lot::RwLockUpgradableReadGuard;
use rsa::{PaddingScheme, RSAPrivateKey};
use serde::Serialize;
use serde_json;
use sha2::Sha256;
use std::collections::HashMap;
use std::convert::Infallible;
use std::fs;
use warp::{http::StatusCode, reply};

use crate::PRIVATE_KEY;
const BLOCK_TRANSACTION_COUNT: u8 = 10;
const BLOCK_REWARD: u8 = 3;

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
};

const BEARER: &str = "Bearer ";

/// POST request to /register endpoint
///
/// Lets a [`User`] (=student) to authenticate themselves to the system
/// This `request` can be rejected if the payload is malformed (=not authenticated properly) or if
/// the [`AuthRequest.user_id`] of the `request` is not in the list of users that can hold a Gradecoin account
///
/// # Authentication Process
/// - Gradecoin's Public Key (`gradecoin_public_key`) is listed on moodle.
/// - Gradecoin's Private Key (`gradecoin_private_key`) is loaded here
///
/// - Student picks a short temporary key (`k_temp`)
/// - Creates a JSON object (`auth_plaintext`) with their `metu_id` and `public key` in base64 (PEM) format (`S_PK`):
/// {
///     student_id: "e12345",
///     passwd: "15 char secret"
///     public_key: "---BEGIN PUBLIC KEY..."
/// }
///
/// - Encrypts the serialized string of `auth_plaintext` with 128 bit block AES in CBC mode with Pkcs7 padding using the temporary key (`k_temp`), the result is `auth_ciphertext` TODO should this be base64'd?
/// - The temporary key student has picked `k_temp` is encrypted using RSA with OAEP padding scheme
/// using sha256 with `gradecoin_public_key` (TODO base64? same as above), giving us `key_ciphertext`
/// - The payload JSON object (`auth_request`) can be JSON serialized now:
/// {
///     c: "auth_ciphertext"
///     key: "key_ciphertext"
/// }
///
/// ## Gradecoin Side
///
/// - Upon receiving, we first RSA decrypt with OAEP padding scheme using SHA256 with `gradecoin_private_key` as the key and auth_request.key `key` as the ciphertext, receiving `temp_key` (this is the temporary key chosen by student)
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
    debug!("POST request to /register, authenticate_user");

    // In essence PEM files are just base64 encoded versions of the DER encoded data.
    // ~tls.mbed.org

    // TODO: lazyload or something <14-04-21, yigit> //
    // Load our RSA Private Key as DER
    let der_encoded = PRIVATE_KEY
        .lines()
        .filter(|line| !line.starts_with("-"))
        .fold(String::new(), |mut data, line| {
            data.push_str(&line);
            data
        });

    // base64(der(pem))
    // Our private key is saved in PEM (base64) format
    let der_bytes = base64::decode(&der_encoded).expect("failed to decode base64 content");
    let gradecoin_private_key = RSAPrivateKey::from_pkcs1(&der_bytes).expect("failed to parse key");

    let padding = PaddingScheme::new_oaep::<sha2::Sha256>();
    let temp_key = gradecoin_private_key
        .decrypt(padding, &request.key.as_bytes())
        .expect("failed to decrypt");

    // decrypt c using key dec_key
    let cipher = Aes128Cbc::new_var(&temp_key, &request.iv).unwrap();
    let auth_plaintext = cipher
        .decrypt_vec(&base64::decode(request.c).unwrap())
        .unwrap();

    let request: AuthRequest =
        serde_json::from_str(&String::from_utf8(auth_plaintext).unwrap()).unwrap();

    let provided_id = request.student_id.clone();

    let privileged_student_id = match MetuId::new(request.student_id, request.passwd) {
        Some(id) => id,
        None => {
            let res_json = warp::reply::json(&GradeCoinResponse {
                res: ResponseType::Error,
                message: "This user cannot have a gradecoin account".to_owned(),
            });

            return Ok(warp::reply::with_status(res_json, StatusCode::BAD_REQUEST));
        }
    };

    let userlist = db.users.upgradable_read();

    if userlist.contains_key(&provided_id) {
        let res_json = warp::reply::json(&GradeCoinResponse {
            res: ResponseType::Error,
            message:
                "This user is already authenticated, do you think this is a mistake? Contact me"
                    .to_owned(),
        });

        return Ok(warp::reply::with_status(res_json, StatusCode::BAD_REQUEST));
    }

    // We're using this as the validator
    // I hate myself
    if let Err(_) = DecodingKey::from_rsa_pem(request.public_key.as_bytes()) {
        let res_json = warp::reply::json(&GradeCoinResponse {
            res: ResponseType::Error,
            message: "The supplied RSA public key is not in valid PEM format".to_owned(),
        });

        return Ok(warp::reply::with_status(res_json, StatusCode::BAD_REQUEST));
    }

    // We're using this as the validator
    // I hate myself
    if let Err(_) = DecodingKey::from_rsa_pem(request.public_key.as_bytes()) {
        let res_json = warp::reply::json(&GradeCoinResponse {
            res: ResponseType::Error,
            message: "The supplied RSA public key is not in valid PEM format".to_owned(),
        });

        return Ok(warp::reply::with_status(res_json, StatusCode::BAD_REQUEST));
    }

    let fingerprint = format!("{:x}", Sha256::digest(&request.public_key.as_bytes()));

    let new_user = User {
        user_id: privileged_student_id,
        public_key: request.public_key,
        balance: 0,
    };

    let user_json = serde_json::to_string(&new_user).unwrap();

    fs::write(format!("users/{}.guy", new_user.user_id), user_json).unwrap();

    let mut userlist = RwLockUpgradableReadGuard::upgrade(userlist);

    userlist.insert(fingerprint.clone(), new_user);

    let res_json = warp::reply::json(&GradeCoinResponse {
        res: ResponseType::Success,
        message: format!(
            "User authenticated to use Gradecoin with identifier {}",
            fingerprint
        ),
    });

    Ok(warp::reply::with_status(res_json, StatusCode::CREATED))
}

// fn shed_pem_header_footer(maybe_key: String) -> Result<Vec<u8>, String> {
//     let der_encoded = maybe_key
//         .lines()
//         .filter(|line| !line.starts_with("-"))
//         .fold(String::new(), |mut data, line| {
//             data.push_str(&line);
//             data
//         });
//     Ok(base64::decode(&der_encoded).expect("failed to decode base64 content"))
// }

/// GET /transaction
/// Returns JSON array of transactions
/// Cannot fail
pub async fn list_transactions(db: Db) -> Result<impl warp::Reply, Infallible> {
    debug!("GET request to /transaction, list_transactions");
    let mut result = HashMap::new();

    let transactions = db.pending_transactions.read();
    // let transactions = transactions.clone().into_iter().collect();

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
pub async fn authorized_propose_block(
    new_block: Block,
    token: String,
    db: Db,
) -> Result<impl warp::Reply, warp::Rejection> {
    debug!("POST request to /block, authorized_propose_block");

    let users_store = db.users.upgradable_read();

    println!("{:?}", &new_block);

    if new_block.transaction_list.len() < 1 {
        let res_json = warp::reply::json(&GradeCoinResponse {
            res: ResponseType::Error,
            message: format!(
                "There should be {} transactions in the block",
                BLOCK_TRANSACTION_COUNT
            ),
        });

        return Ok(warp::reply::with_status(res_json, StatusCode::BAD_REQUEST));
    }

    // proposer (first transaction fingerprint) checks
    let internal_user = match users_store.get(&new_block.transaction_list[0]) {
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
            debug!("Something went wrong below {:?}", below);

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
            message: "The hash of the block did not match the hash given in JWT".to_owned(),
        });

        return Ok(warp::reply::with_status(res_json, StatusCode::BAD_REQUEST));
    }

    // Scope the RwLocks, there are hashing stuff below
    {
        let pending_transactions = db.pending_transactions.read();

        // Are transactions in the block valid?
        for transaction_hash in new_block.transaction_list.iter() {
            if !pending_transactions.contains_key(transaction_hash) {
                let res_json = warp::reply::json(&GradeCoinResponse {
                    res: ResponseType::Error,
                    message: "Block contains unknown transaction".to_owned(),
                });

                return Ok(warp::reply::with_status(res_json, StatusCode::BAD_REQUEST));
            }
        }
    }

    let naked_block = NakedBlock {
        transaction_list: new_block.transaction_list.clone(),
        nonce: new_block.nonce.clone(),
        timestamp: new_block.timestamp.clone(),
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

    // Are the 6 rightmost characters (=24 bits) zero?
    let should_zero = hashvalue[31] as i32 + hashvalue[30] as i32 + hashvalue[29] as i32;

    if should_zero != 0 {
        debug!("the hash does not have 6 rightmost zero bits");
        let res_json = warp::reply::json(&GradeCoinResponse {
            res: ResponseType::Error,
            message: "Given block hash is larger than target value".to_owned(),
        });

        return Ok(warp::reply::with_status(res_json, StatusCode::BAD_REQUEST));
    }

    // All clear, block accepted!
    debug!("We have a new block!");

    {
        let pending_transactions = db.pending_transactions.read();
        let mut users = db.users.write();

        let coinbase_fingerprint = new_block.transaction_list.get(0).unwrap();

        for fingerprint in new_block.transaction_list.iter() {
            if let Some(transaction) = pending_transactions.get(fingerprint) {
                let source = &transaction.source;
                let target = &transaction.target;

                if let Some(from) = users.get_mut(source) {
                    from.balance -= transaction.amount;
                }

                if let Some(to) = users.get_mut(target) {
                    to.balance += transaction.amount;
                }
            }
        }

        if let Some(coinbase_user) = users.get_mut(coinbase_fingerprint) {
            coinbase_user.balance += BLOCK_REWARD as i32;
        }
    }

    {
        let mut pending_transactions = db.pending_transactions.write();
        pending_transactions.clear();
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
            message: "Block accepted coinbase reward awarded".to_owned(),
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
/// TODO This method should check if the user has enough balance for the transaction
pub async fn authorized_propose_transaction(
    new_transaction: Transaction,
    token: String,
    db: Db,
) -> Result<impl warp::Reply, warp::Rejection> {
    debug!("POST request to /transaction, authorized_propose_transaction");

    let users_store = db.users.read();

    // Is this transaction from an authorized source?
    let internal_user = match users_store.get(&new_transaction.by) {
        Some(existing_user) => existing_user,
        None => {
            debug!(
                "User with public key signature {:?} is not found in the database",
                new_transaction.by
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

    // `internal_user` is an authenticated student, can propose

    // check if user can afford the transaction
    if new_transaction.by == new_transaction.source {
        // Propose to transact with another user
        if internal_user.balance < new_transaction.amount {
            return Ok(warp::reply::with_status(
                warp::reply::json(&GradeCoinResponse {
                    res: ResponseType::Error,
                    message: "User does not have enough balance in their account".to_owned(),
                }),
                StatusCode::BAD_REQUEST,
            ));
        }
    } else {
        // TODO: add bank mechanism <14-04-21, keles> //
        return Ok(warp::reply::with_status(
            warp::reply::json(&GradeCoinResponse {
                res: ResponseType::Error,
                message: "Invalid by field for the proposed transaction".to_owned(),
            }),
            StatusCode::BAD_REQUEST,
        ));
    }

    // This public key was already written to the database, we can panic if it's not valid at
    // *this* point
    let proposer_public_key = &internal_user.public_key;

    let token_payload = match authorize_proposer(token, &proposer_public_key) {
        Ok(data) => data,
        Err(below) => {
            debug!("Something went wrong below {:?}", below);
            return Ok(warp::reply::with_status(
                warp::reply::json(&GradeCoinResponse {
                    res: ResponseType::Error,
                    message: below,
                }),
                StatusCode::BAD_REQUEST,
            ));
        }
    };

    // authorized for transaction proposal

    // this transaction was already checked for correctness at custom_filters, we can panic here if
    // it has been changed since

    let hashed_transaction = Md5::digest(&serde_json::to_vec(&new_transaction).unwrap());

    if token_payload.claims.tha != format!("{:x}", hashed_transaction) {
        debug!(
            "the hash of the request {:x} did not match the hash given in jwt {:?}",
            hashed_transaction, token_payload.claims.tha
        );
        return Ok(warp::reply::with_status(
            warp::reply::json(&GradeCoinResponse {
                res: ResponseType::Error,
                message: "The hash of the block did not match the hash given in JWT".to_owned(),
            }),
            StatusCode::BAD_REQUEST,
        ));
    }

    debug!("clear for transaction proposal");

    let mut transactions = db.pending_transactions.write();
    transactions.insert(new_transaction.source.to_owned(), new_transaction);
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
    debug!("GET request to /block, list_blocks");

    let block = db.blockchain.read();

    Ok(reply::with_status(reply::json(&*block), StatusCode::OK))
}

/// Handles the JWT Authorization
///
/// *[`jwt_token`]: The raw JWT token, "Bearer aaa.bbb.ccc"
/// *[`user_pem`]: User Public Key, "BEGIN RSA"
/// NOT async, might look into it if this becomes a bottleneck
fn authorize_proposer(jwt_token: String, user_pem: &String) -> Result<TokenData<Claims>, String> {
    // Throw away the "Bearer " part
    let raw_jwt = jwt_token.trim_start_matches(BEARER).to_owned();
    debug!("raw_jwt: {:?}", raw_jwt);

    // Extract a jsonwebtoken compatible decoding_key from user's public key
    let decoding_key = match DecodingKey::from_rsa_pem(user_pem.as_bytes()) {
        Ok(key) => key,
        Err(j) => {
            warn!(
                "user has invalid RSA key we should crash and burn here {:?}",
                j
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
                    warn!("AN UNSPECIFIED ERROR: {:?}", err);
                    return Err(String::from("Unspecified error"));
                }
            },
        };

    Ok(token_payload)
}
