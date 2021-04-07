// Common filters ment to be shared between many endpoints

use std::convert::Infallible;
use warp::{Filter, Rejection};

use crate::schema::{Block, Db, Transaction};

// Database context for routes
pub fn with_db(db: Db) -> impl Filter<Extract = (Db,), Error = Infallible> + Clone {
    warp::any().map(move || db.clone())
}

// Optional query params to allow pagination
// pub fn list_options() -> impl Filter<Extract = (ListOptions,), Error = Rejection> + Clone {
//     warp::query::<ListOptions>()
// }

// Accept only json encoded Transaction body and reject big payloads
pub fn transaction_json_body() -> impl Filter<Extract = (Transaction,), Error = Rejection> + Clone {
    warp::body::content_length_limit(1024 * 32).and(warp::body::json())
}

// Accept only json encoded Transaction body and reject big payloads
pub fn block_json_body() -> impl Filter<Extract = (Block,), Error = Rejection> + Clone {
    warp::body::content_length_limit(1024 * 32).and(warp::body::json())
}
