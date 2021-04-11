use warp::{Filter, Rejection, Reply};

use crate::custom_filters;
use crate::handlers;
use crate::schema::Db;

/// Root, all routes combined
pub fn consensus_routes(db: Db) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    transaction_list(db.clone())
        .or(register_user(db.clone()))
        .or(transaction_propose(db.clone()))
        .or(block_propose(db.clone()))
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

#[cfg(test)]
mod tests {
    use super::*;

    // use chrono::prelude::*;
    // use parking_lot::RwLock;
    // use std::sync::Arc;
    use warp::http::StatusCode;

    use crate::schema;
    use crate::schema::{AuthRequest, Block, Transaction};

    /// Create a mock database to be used in tests
    fn mocked_db() -> Db {
        let db = schema::create_database();

        db.pending_transactions.write().insert(
            "hash_value".to_owned(),
            Transaction {
                source: "source_account".to_owned(),
                target: "target_account".to_owned(),
                amount: 20,
                timestamp: chrono::NaiveDate::from_ymd(2021, 04, 09).and_hms(1, 30, 30),
            },
        );

        *db.blockchain.write() = Block {
            transaction_list: vec![
                "old_transaction_hash_1".to_owned(),
                "old_transaction_hash_2".to_owned(),
                "old_transaction_hash_3".to_owned(),
            ],
            nonce: "not_a_thing_yet".to_owned(),
            timestamp: chrono::NaiveDate::from_ymd(2021, 04, 08).and_hms(12, 30, 30),
            hash: "not_a_thing_yet".to_owned(),
        };

        db
    }

    /// Create a mock user that is allowed to be in gradecoin to be used in tests
    fn priviliged_mocked_user() -> AuthRequest {
        AuthRequest {
            student_id: String::from("e254275"),
            public_key: "NOT IMPLEMENTED".to_owned(),
        }
    }

    /// Create a mock user that is NOT allowed to be in gradecoin to be used in tests
    fn unpriviliged_mocked_user() -> AuthRequest {
        AuthRequest {
            student_id: String::from("foobarbaz"),
            public_key: "NOT IMPLEMENTED".to_owned(),
        }
    }

    /// Create a mock transaction to be used in tests
    fn mocked_transaction() -> Transaction {
        Transaction {
            source: "mock_transaction_source".to_owned(),
            target: "mock_transaction_target".to_owned(),
            amount: 25,
            timestamp: chrono::NaiveDate::from_ymd(2021, 04, 09).and_hms(14, 30, 00),
        }
    }

    /// Test simple GET request to /transaction, resource that exists
    /// https://tools.ietf.org/html/rfc7231#section-6.3.1
    /// We should get the only pending transaction available in the database as json
    #[tokio::test]
    async fn get_pending_transactions() {
        let db = mocked_db();

        let reply = consensus_routes(db);

        let res = warp::test::request()
            .method("GET")
            .path("/transaction")
            .reply(&reply)
            .await;

        assert_eq!(res.status(), StatusCode::OK);

        let expected_json_body = r#"[{"source":"source_account","target":"target_account","amount":20,"timestamp":"2021-04-09T01:30:30"}]"#;

        assert_eq!(res.body(), expected_json_body);
    }

    /// Test simple GET request to /block, resource that exists
    /// https://tools.ietf.org/html/rfc7231#section-6.3.1
    /// Should return the single block available in the database as json
    #[tokio::test]
    async fn get_blockchain() {
        let db = mocked_db();
        let filter = consensus_routes(db);

        let res = warp::test::request()
            .method("GET")
            .path("/block")
            .reply(&filter)
            .await;

        assert_eq!(res.status(), StatusCode::OK);

        let expected_json_body = r#"{"transaction_list":["old_transaction_hash_1","old_transaction_hash_2","old_transaction_hash_3"],"nonce":"not_a_thing_yet","timestamp":"2021-04-08T12:30:30","hash":"not_a_thing_yet"}"#;
        assert_eq!(res.body(), expected_json_body);
    }

    /// Test a simple GET request to a nonexisting path
    /// https://tools.ietf.org/html/rfc7231#section-6.5.4
    /// Should respond with 404 and stop
    #[tokio::test]
    async fn get_nonexisting_path_404() {
        let db = mocked_db();
        let filter = consensus_routes(db);

        let res = warp::test::request()
            .method("GET")
            .path("/this_path_does_not_exist")
            .reply(&filter)
            .await;

        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }

    /// Test a POST request to /transaction, a resource that exists
    /// https://tools.ietf.org/html/rfc7231#section-6.3.2
    /// Should accept the json request, create
    /// the transaction and add it to pending transactions in the db
    #[tokio::test]
    async fn post_json_201() {
        let db = mocked_db();
        let filter = consensus_routes(db.clone());

        let res = warp::test::request()
            .method("POST")
            .json(&mocked_transaction())
            .path("/transaction")
            .reply(&filter)
            .await;

        assert_eq!(res.status(), StatusCode::CREATED);
        assert_eq!(db.pending_transactions.read().len(), 2);
    }

    /// TEST a POST request to /transaction, an endpoint that exists
    /// https://tools.ietf.org/html/rfc7231#section-6.3.2
    /// Should accept the json request, create a new user and
    /// add it to the user hashmap in the db
    #[tokio::test]
    async fn post_register_priviliged_user() {
        let db = mocked_db();
        let filter = consensus_routes(db.clone());

        let res = warp::test::request()
            .method("POST")
            .json(&priviliged_mocked_user())
            .path("/register")
            .reply(&filter)
            .await;

        println!("{:?}", res.body());
        assert_eq!(res.status(), StatusCode::CREATED);
        assert_eq!(db.users.read().len(), 1);
    }
    /// TEST a POST request to /transaction, an endpoint that exists
    /// https://tools.ietf.org/html/rfc7231#section-6.3.2
    /// Should NOT accept the json request
    #[tokio::test]
    async fn post_register_unpriviliged_user() {
        let db = mocked_db();
        let filter = consensus_routes(db.clone());

        let res = warp::test::request()
            .method("POST")
            .json(&unpriviliged_mocked_user())
            .path("/register")
            .reply(&filter)
            .await;

        println!("{:?}", res.body());
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        assert_eq!(db.users.read().len(), 0);
    }

    /// Test a POST request to /transaction, a resource that exists with a longer than expected
    /// payload
    /// https://tools.ietf.org/html/rfc7231#section-6.5.11
    /// Should return 413 to user
    #[tokio::test]
    async fn post_too_long_content_413() {
        let db = mocked_db();
        let filter = consensus_routes(db);

        let res = warp::test::request()
            .method("POST")
            .header("content-length", 1024 * 36)
            .path("/transaction")
            .reply(&filter)
            .await;

        assert_eq!(res.status(), StatusCode::PAYLOAD_TOO_LARGE);
    }
}

// TODO: POST block test <09-04-21, yigit> //
// TODO: POST block without correct transactions test <09-04-21, yigit> //
// TODO: POST transaction while that source has pending transaction test <09-04-21, yigit> //
