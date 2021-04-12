use gradecoin::schema::{AuthRequest, Block, Db, Transaction};
use std::convert::Infallible;
use warp::{Filter, Rejection};

// Database context for routes
pub fn with_db(db: Db) -> impl Filter<Extract = (Db,), Error = Infallible> + Clone {
    warp::any().map(move || db.clone())
}

// Accept only json encoded User body and reject big payloads
// TODO: find a good limit for this, (=e2482057; 8 char String + rsa pem) <11-04-21, yigit> //
pub fn auth_request_json_body() -> impl Filter<Extract = (AuthRequest,), Error = Rejection> + Clone
{
    warp::body::content_length_limit(1024 * 32).and(warp::body::json())
}

// Accept only json encoded Transaction body and reject big payloads
// TODO: find a good limit for this <11-04-21, yigit> //
pub fn transaction_json_body() -> impl Filter<Extract = (Transaction,), Error = Rejection> + Clone {
    warp::body::content_length_limit(1024 * 32).and(warp::body::json())
}

// Accept only json encoded Block body and reject big payloads
// TODO: find a good limit for this <11-04-21, yigit> //
pub fn block_json_body() -> impl Filter<Extract = (Block,), Error = Rejection> + Clone {
    warp::body::content_length_limit(1024 * 32).and(warp::body::json())
}
