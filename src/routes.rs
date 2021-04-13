/// Endpoints and their construction
use warp::{Filter, Rejection, Reply};

use crate::custom_filters;
use crate::handlers;
use crate::schema::Db;

/// Every route combined
pub fn consensus_routes(db: Db) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    transaction_list(db.clone())
        .or(register_user(db.clone()))
        .or(auth_transaction_propose(db.clone()))
        .or(auth_block_propose(db.clone()))
        .or(block_list(db.clone()))
}

/// POST /register warp route
pub fn register_user(db: Db) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("register")
        .and(warp::post())
        .and(custom_filters::auth_request_json_body())
        .and(custom_filters::with_db(db))
        .and_then(handlers::authenticate_user)
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
pub fn auth_transaction_propose(
    db: Db,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("transaction")
        .and(warp::post())
        .and(custom_filters::transaction_json_body())
        .and(custom_filters::auth_header())
        .and(custom_filters::with_db(db))
        .and_then(handlers::auth_propose_transaction)
}

/// POST /block warp route
pub fn auth_block_propose(db: Db) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("block")
        .and(warp::post())
        .and(custom_filters::block_json_body())
        .and(custom_filters::auth_header())
        .and(custom_filters::with_db(db))
        .and_then(handlers::auth_propose_block)
}

