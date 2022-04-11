//! # Endpoints and their construction
//
use warp::{Filter, Rejection, Reply};
use crate::custom_filters;
use crate::handlers;
use crate::Db;

/// Every route combined
pub fn application(db: Db) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    // Remember when we wanted to implement templating
    // Why would we? Just put a staic webpage under /public (next to Cargo.toml) and place it and
    // the end of the filter chain

    // Fully fledged website support, phew!
    let static_route = warp::any().and(warp::fs::dir("public"));

    transaction_list(db.clone())
        .or(register_user(db.clone()))
        .or(auth_transaction_propose(db.clone()))
        .or(auth_block_propose(db.clone()))
        .or(list_users(db.clone()))
        .or(block_list(db))
        .or(static_route)
}

/// GET /user warp route
pub fn list_users(db: Db) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("user")
        .and(warp::get())
        .and(custom_filters::with_db(db))
        .and_then(handlers::user_list_handler)
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
        .and_then(handlers::propose_transaction)
}

/// POST /block warp route
pub fn auth_block_propose(db: Db) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("block")
        .and(warp::post())
        .and(custom_filters::block_json_body())
        .and(custom_filters::auth_header())
        .and(custom_filters::with_db(db))
        .and_then(handlers::propose_block)
}
