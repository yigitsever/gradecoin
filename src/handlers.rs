/// API handlers, the ends of each filter chain
use log::debug; // this is more useful than debug! learn how to use this
use parking_lot::RwLockUpgradableReadGuard;
use std::convert::Infallible;
use warp::{http::StatusCode, reply};

use crate::schema::{Block, Db, Transaction};

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
    debug!("list all blocks");

    let mut result = Vec::new();
    let blocks = db.blockchain.read();

    for block in blocks.iter() {
        result.push(block);
    }

    Ok(reply::with_status(reply::json(&result), StatusCode::OK))
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

    // TODO: check 1, new_block.transaction_list from pending_transactions pool? <07-04-21, yigit> //
    for transaction_hash in new_block.transaction_list.iter() {
        if !pending_transactions.contains_key(transaction_hash) {
            return Ok(StatusCode::BAD_REQUEST);
        }
    }

    // TODO: check 2, block hash (\w nonce) asserts $hash_condition? <07-04-21, yigit> //
    // assume it is for now

    let mut blockchain = RwLockUpgradableReadGuard::upgrade(blockchain);
    blockchain.push(new_block);

    let mut pending_transactions = RwLockUpgradableReadGuard::upgrade(pending_transactions);
    pending_transactions.clear();

    Ok(StatusCode::CREATED)
}
