use log::warn;
use serde::Serialize;
use std::convert::Infallible;
use warp::{http::StatusCode, Rejection, Reply};

#[derive(Serialize)]
struct ErrorResponse {
    message: String,
}

pub async fn handle_rejection(err: Rejection) -> std::result::Result<impl Reply, Infallible> {
    let code;
    let message;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "Requested resource is not found";
    } else if let Some(_) = err.find::<warp::filters::body::BodyDeserializeError>() {
        code = StatusCode::BAD_REQUEST;
        message = "Error: JSON body is not formatted correctly, check your payload";
    } else if let Some(_) = err.find::<warp::reject::MissingHeader>() {
        code = StatusCode::METHOD_NOT_ALLOWED;
        message = "Error: Authorization header missing, cannot authorize";
    } else if let Some(_) = err.find::<warp::reject::MethodNotAllowed>() {
        code = StatusCode::METHOD_NOT_ALLOWED;
        message = "Error: method not allowed on this endpoint";
    } else {
        warn!("unhandled error: {:?}", err);
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "Internal Server Error";
    }

    let json = warp::reply::json(&ErrorResponse {
        message: message.to_owned(),
    });

    Ok(warp::reply::with_status(json, code))
}
