use warp::{Filter, Rejection, Reply};

use crate::custom_filters;
use crate::handlers;
use crate::schema::Db;

/// Root, all routes combined
pub fn consensus_routes(db: Db) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    transaction_list(db.clone())
        .or(transaction_propose(db.clone()))
        .or(block_propose(db.clone()))
        .or(block_list(db.clone()))
}

/// GET /transaction warp route
pub fn transaction_list(db: Db) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("transaction")
        .and(warp::get())
        .and(custom_filters::with_db(db))
        .and_then(handlers::list_transactions)
}

/// GET /block warp route
pub fn block_list(db: Db) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("block")
        .and(warp::get())
        .and(custom_filters::with_db(db))
        .and_then(handlers::list_blocks)
}

/// POST /transaction warp route
pub fn transaction_propose(db: Db) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("transaction")
        .and(warp::post())
        .and(custom_filters::transaction_json_body())
        .and(custom_filters::with_db(db))
        .and_then(handlers::propose_transaction)
}

/// POST /block warp route
pub fn block_propose(db: Db) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("block")
        .and(warp::post())
        .and(custom_filters::block_json_body())
        .and(custom_filters::with_db(db))
        .and_then(handlers::propose_block)
}

