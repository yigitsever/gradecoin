/// API handlers, the ends of each filter chain
use log::debug;
use parking_lot::RwLockUpgradableReadGuard;
use serde_json;
use serde_json::json;
use std::convert::Infallible;
use warp::{http::Response, http::StatusCode, reply};

use blake2::{Blake2s, Digest};

use std::fs;

use gradecoin::schema::{AuthRequest, Block, Db, MetuId, NakedBlock, Transaction, User};

/// POST /register
/// Enables a student to introduce themselves to the system
/// Can fail
pub async fn authenticate_user(
    request: AuthRequest,
    db: Db,
) -> Result<impl warp::Reply, warp::Rejection> {
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
    debug!("list all transactions");
    let mut result = Vec::new();

    let transactions = db.pending_transactions.read();
    // let transactions = transactions.clone().into_iter().collect();

    for (_, value) in transactions.iter() {
        result.push(value)
    }

    Ok(reply::with_status(reply::json(&result), StatusCode::OK))
}

/// GET /block
/// Returns JSON array of blocks
/// Cannot fail
/// Mostly around for debug purposes
pub async fn list_blocks(db: Db) -> Result<impl warp::Reply, Infallible> {
    debug!("list all block");

    let block = db.blockchain.read();

    Ok(reply::with_status(reply::json(&*block), StatusCode::OK))
}

/// POST /transaction
/// Pushes a new transaction for pending transaction pool
/// Can reject the transaction proposal
/// TODO: when is a new transaction rejected <07-04-21, yigit> //
pub async fn propose_transaction(
    new_transaction: Transaction,
    db: Db,
) -> Result<impl warp::Reply, warp::Rejection> {
    debug!("new transaction request {:?}", new_transaction);

    // let mut transactions = db.lock().await;
    let mut transactions = db.pending_transactions.write();

    transactions.insert(new_transaction.source.to_owned(), new_transaction);

    Ok(StatusCode::CREATED)
}

/// POST /block
/// Proposes a new block for the next round
/// Can reject the block
pub async fn propose_block(new_block: Block, db: Db) -> Result<impl warp::Reply, warp::Rejection> {
    debug!("new block request {:?}", new_block);

    // https://blog.logrocket.com/create-an-async-crud-web-service-in-rust-with-warp/ (this has
    // error.rs, error struct, looks very clean)

    let pending_transactions = db.pending_transactions.upgradable_read();
    let blockchain = db.blockchain.upgradable_read();

    // check 1, new_block.transaction_list from pending_transactions pool? <07-04-21, yigit> //
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

    // 6 rightmost bits are zero
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

            let mut pending_transactions = RwLockUpgradableReadGuard::upgrade(pending_transactions);
            pending_transactions.clear();

            Ok(StatusCode::CREATED)
        } else {
            Ok(StatusCode::BAD_REQUEST)
        }
    } else {
        // reject
        Ok(StatusCode::BAD_REQUEST)
    }
}
