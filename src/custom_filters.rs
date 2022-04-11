//! # Functions that extracts Structs to be used in warp routines
use crate::block::{Block, InitialAuthRequest, Transaction};
use crate::Db;
use std::convert::Infallible;
use warp::{Filter, Rejection};

/// Wraps the database to be used in warp routes
pub fn with_db(db: Db) -> impl Filter<Extract = (Db,), Error = Infallible> + Clone {
    warp::any().map(move || db.clone())
}

/// Extracts an `InitialAuthRequest` JSON body from the request
/// Accepts only JSON encoded `AuthRequest` body and rejects big payloads
///
pub fn auth_request_json_body(
) -> impl Filter<Extract = (InitialAuthRequest,), Error = Rejection> + Clone {
    warp::body::content_length_limit(1024 * 32).and(warp::body::json())
}

/// Extracts an `Transaction` JSON body from the request
/// Accepts only JSON encoded `Transaction` body and rejects big payloads
pub fn transaction_json_body() -> impl Filter<Extract = (Transaction,), Error = Rejection> + Clone {
    warp::body::content_length_limit(1024 * 32).and(warp::body::json())
}

/// Extracts the value of the `Authorization` header field, hopefully a valid JWT
/// Used in Authorization for `Block` and `Transaction` proposals
/// Rejects the request if the Authorization header does not exist
pub fn auth_header() -> impl Filter<Extract = (String,), Error = Rejection> + Clone {
    warp::header::<String>("Authorization")
}

/// Extracts an `Block` JSON body from the request
/// Accepts only JSON encoded `Block` body and rejects big payloads
pub fn block_json_body() -> impl Filter<Extract = (Block,), Error = Rejection> + Clone {
    warp::body::content_length_limit(1024 * 32).and(warp::body::json())
}
