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

// `GET /games`
// Returns JSON array of todos
// Allows pagination, for example: `GET /games?offset=10&limit=5`
// pub async fn list_games(options: ListOptions, db: Db) -> Result<impl Reply, Infallible> {
//     debug!("list all games");

//     let games = db.lock().await;
//     let games: Vec<Game> = games
//         .clone()
//         .into_iter()
//         .skip(options.offset.unwrap_or(0))
//         .take(options.limit.unwrap_or(std::usize::MAX))
//         .collect();

//     Ok(warp::reply::json(&games))
// }

// `POST /games`
// Create new game entry with JSON body
// pub async fn create_game(new_game: Game, db: Db) -> Result<impl Reply, Infallible> {
//     debug!("create new game: {:?}", new_game);

//     let mut games = db.lock().await;

//     match games.iter().find(|game| game.id == new_game.id) {
//         Some(game) => {
//             debug!("game of given id already exists: {}", game.id);

//             Ok(StatusCode::BAD_REQUEST)
//         }
//         None => {
//             games.push(new_game);
//             Ok(StatusCode::CREATED)
//         }
//     }
// }

// `PUT /games/:id`
// pub async fn update_game(id: u64, updated_game: Game, db: Db) -> Result<impl Reply, Infallible> {
//     debug!("update existing game: id={}, game={:?}", id, updated_game);

//     let mut games = db.lock().await;

//     match games.iter_mut().find(|game| game.id == id) {
//         Some(game) => {
//             *game = updated_game;

//             Ok(StatusCode::OK)
//         }
//         None => {
//             debug!("game of given id not found");

//             Ok(StatusCode::NOT_FOUND)
//         }
//     }
// }

// `DELETE /games/:id`
// pub async fn delete_game(id: u64, db: Db) -> Result<impl Reply, Infallible> {
//     debug!("delete game: id={}", id);

//     let mut games = db.lock().await;

//     let len = games.len();

//     // Removes all games with given id
//     games.retain(|game| game.id != id);

//     // If games length was smaller that means specyfic game was found and removed
//     let deleted = games.len() != len;

//     if deleted {
//         Ok(StatusCode::NO_CONTENT)
//     } else {
//         debug!("game of given id not found");

//         Ok(StatusCode::NOT_FOUND)
//     }
// }
