//! # Endpoints and their construction
//
use crate::custom_filters;
use crate::handlers;
use crate::Db;
use log::info;
use warp::{filters::BoxedFilter, Filter, Rejection, Reply};

/// Every route combined for a single network
pub fn network(db: Db) -> BoxedFilter<(impl Reply,)> {
    let url_prefix = db.config.url_prefix.clone();
    info!(
        "{} will be served at endpoint /{}",
        db.config.name, url_prefix
    );
    let root = if url_prefix.is_empty() {
        // warp::path does not like empty url_prefix
        // We need to handle this case separately
        warp::any().boxed()
    } else {
        warp::path(url_prefix).boxed()
    };
    root.and(
        transaction_list(db.clone())
            .or(get_config_route(db.clone()))
            .or(get_version())
            .or(register_user(db.clone()))
            .or(auth_transaction_propose(db.clone()))
            .or(auth_block_propose(db.clone()))
            .or(list_users(db.clone()))
            .or(block_list(db)),
    )
    .boxed()
}

/// GET /config warp route
pub fn get_config_route(db: Db) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("config")
        .and(warp::get())
        .and(custom_filters::with_db(db))
        .and_then(handlers::get_config)
}

/// GET /version warp route
pub fn get_version() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("version")
        .and(warp::get())
        .and_then(handlers::get_version)
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
