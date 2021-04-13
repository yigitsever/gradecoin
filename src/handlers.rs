/// API handlers, the ends of each filter chain
use blake2::{Blake2s, Digest};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use log::debug;
use md5::Md5;
use parking_lot::RwLockUpgradableReadGuard;
use serde_json;
use std::convert::Infallible;
use std::fs;
use warp::{http::Response, http::StatusCode, reply};

use crate::schema::{AuthRequest, Block, Claims, Db, MetuId, NakedBlock, Transaction, User};

const BEARER: &str = "Bearer ";

/// POST request to /register endpoint
///
/// Lets a [`User`] (=student) to authenticate themselves to the system
/// This `request` can be rejected if the payload is malformed (= not authenticated properly) or if
/// the [`AuthRequest.user_id`] of the `request` is not in the list of users that can hold a Gradecoin account
pub async fn authenticate_user(
    request: AuthRequest,
    db: Db,
) -> Result<impl warp::Reply, warp::Rejection> {
    debug!("POST request to /register, authenticate_user");
    let given_id = request.student_id.clone();

    if let Some(priv_student_id) = MetuId::new(request.student_id) {
        let userlist = db.users.upgradable_read();

        if userlist.contains_key(&given_id) {
            let res = Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body("This user is already authenticated");

            Ok(res)
        } else {
            let new_user = User {
                user_id: priv_student_id,
                public_key: request.public_key,
                balance: 0,
            };

            let user_json = serde_json::to_string(&new_user).unwrap();

            fs::write(format!("users/{}.guy", new_user.user_id), user_json).unwrap();

            let mut userlist = RwLockUpgradableReadGuard::upgrade(userlist);
            userlist.insert(given_id, new_user);
            // TODO: signature of the public key, please <11-04-21, yigit> //

            let res = Response::builder()
                .status(StatusCode::CREATED)
                .body("Ready to use Gradecoin");

            Ok(res)
        }
    } else {
        let res = Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body("This user cannot have a gradecoin account");

        Ok(res)
    }
}

/// GET /transaction
/// Returns JSON array of transactions
/// Cannot fail
pub async fn list_transactions(db: Db) -> Result<impl warp::Reply, Infallible> {
    debug!("GET request to /transaction, list_transactions");
    let mut result = Vec::new();

    let transactions = db.pending_transactions.read();
    // let transactions = transactions.clone().into_iter().collect();

    for (_, value) in transactions.iter() {
        result.push(value)
    }

    Ok(reply::with_status(reply::json(&result), StatusCode::OK))
}

/// POST /block
/// Proposes a new block for the next round
/// Can reject the block
pub async fn auth_propose_block(
    new_block: Block,
    token: String,
    db: Db,
) -> Result<impl warp::Reply, warp::Rejection> {
    debug!("POST request to /block, auth_propose_block");

    // Authorization check
    let raw_jwt = token.trim_start_matches(BEARER).to_owned();
    debug!("raw_jwt: {:?}", raw_jwt);

    // TODO: WHO IS PROPOSING THIS BLOCK OH GOD <13-04-21, yigit> // ok let's say the proposer has
    // to put their transaction as the first transaction of the transaction_list
    // that's not going to backfire in any way
    // TODO: after a block is accepted, it's transactions should play out and the proposer should
    // get something for their efforts <13-04-21, yigit> //
    if let Some(user) = db.users.read().get(&new_block.transaction_list[0]) {
        let proposer_public_key = &user.public_key;

        if let Ok(decoded) = decode::<Claims>(
            &raw_jwt,
            &DecodingKey::from_rsa_pem(proposer_public_key.as_bytes()).unwrap(),
            &Validation::new(Algorithm::RS256),
        ) {
            if decoded.claims.tha != new_block.hash {
                debug!("Authorization unsuccessful");
                return Ok(StatusCode::BAD_REQUEST);
            }

            debug!("authorized for block proposal");

            let pending_transactions = db.pending_transactions.upgradable_read();
            let blockchain = db.blockchain.upgradable_read();

            for transaction_hash in new_block.transaction_list.iter() {
                if !pending_transactions.contains_key(transaction_hash) {
                    return Ok(StatusCode::BAD_REQUEST);
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

            // 6 rightmost bits are zero?
            let should_zero = hashvalue[31] as i32 + hashvalue[30] as i32 + hashvalue[29] as i32;

            if should_zero == 0 {
                // one last check to see if block is telling the truth
                if hash_string == new_block.hash {
                    let mut blockchain = RwLockUpgradableReadGuard::upgrade(blockchain);

                    let block_json = serde_json::to_string(&new_block).unwrap();

                    fs::write(
                        format!("blocks/{}.block", new_block.timestamp.timestamp()),
                        block_json,
                    )
                    .unwrap();

                    *blockchain = new_block;

                    let mut pending_transactions =
                        RwLockUpgradableReadGuard::upgrade(pending_transactions);
                    pending_transactions.clear();

                    Ok(StatusCode::CREATED)
                } else {
                    debug!("request was not telling the truth, hash values do not match");
                    // TODO: does this condition make more sense _before_ the hash 0s check? <13-04-21, yigit> //
                    Ok(StatusCode::BAD_REQUEST)
                }
            } else {
                debug!("the hash does not have 6 rightmost zero bits");
                Ok(StatusCode::BAD_REQUEST)
            }
        } else {
            debug!("authorization failed");
            Ok(StatusCode::BAD_REQUEST)
        }
    } else {
        debug!(
            "A user with public key signature {:?} is not found in the database",
            new_block.transaction_list[0]
        );
        Ok(StatusCode::BAD_REQUEST)
    }
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
///
/// TODO: refactor this https://refactoring.com/catalog/replaceNestedConditionalWithGuardClauses.html
pub async fn auth_propose_transaction(
    new_transaction: Transaction,
    token: String,
    db: Db,
) -> Result<impl warp::Reply, warp::Rejection> {
    debug!("POST request to /transaction, propose_transaction");
    debug!("The transaction request: {:?}", new_transaction);

    let raw_jwt = token.trim_start_matches(BEARER).to_owned();
    println!("raw_jwt: {:?}", raw_jwt);

    // Authorization check first
    if let Some(user) = db.users.read().get(&new_transaction.by) {
        // This public key was already written to the database, we can panic if it's not valid at
        // *this* point
        let by_public_key = &user.public_key;

        if let Ok(decoded) = decode::<Claims>(
            &raw_jwt,
            &DecodingKey::from_rsa_pem(by_public_key.as_bytes()).unwrap(),
            &Validation::new(Algorithm::RS256),
        ) {
            // this transaction was already checked for correctness at custom_filters, we can panic
            // here if it has been changed since
            debug!("authorized for transaction proposal");

            let hashed_transaction = Md5::digest(&serde_json::to_vec(&new_transaction).unwrap());

            if decoded.claims.tha == format!("{:x}", hashed_transaction) {
                let mut transactions = db.pending_transactions.write();

                transactions.insert(new_transaction.source.to_owned(), new_transaction);

                Ok(StatusCode::CREATED)
            } else {
                debug!(
                    "the hash of the request {:x} did not match with the hash given in jwt {:?}",
                    hashed_transaction, decoded.claims.tha
                );
                Ok(StatusCode::BAD_REQUEST)
            }
        } else {
            debug!("raw_jwt was malformed {:?}", raw_jwt);
            Ok(StatusCode::BAD_REQUEST)
        }
    } else {
        debug!(
            "A user with public key signature {:?} is not found in the database",
            new_transaction.by
        );
        Ok(StatusCode::BAD_REQUEST)
    }
}

/// GET /block
/// Returns JSON array of blocks
/// Cannot fail
/// Mostly around for debug purposes
pub async fn list_blocks(db: Db) -> Result<impl warp::Reply, Infallible> {
    debug!("GET request to /block, list_blocks");

    let block = db.blockchain.read();

    Ok(reply::with_status(reply::json(&*block), StatusCode::OK))
}
