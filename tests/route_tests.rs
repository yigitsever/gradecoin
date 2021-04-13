#[cfg(test)]
mod tests {
    use gradecoin::schema::{create_database, AuthRequest, Block, Db, MetuId, Transaction, User};

    use gradecoin::routes::consensus_routes;
    use warp::http::StatusCode;

    /// Create a mock database to be used in tests
    fn mocked_db() -> Db {
        let db = create_database();

        db.users.write().insert(
            "mock_transaction_source".to_owned(),
            User {
                user_id: MetuId::new("e254275".to_owned()).unwrap(),
                public_key: "-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA4nU0G4WjkmcQUx0hq6LQ
uV5Q+ACmUFL/OjoYMDwC/O/6pCd1UZgCfgHN2xEffDPznzcTn8OiFRxr4oWyBiny
rUpnY4mhy0SQUwoeCw7YkcHAyhCjNT74aR/ohX0MCj0qRRdbt5ZQXM/GC3HJuXE1
ptSuhFgQxziItamn8maoJ6JUSVEXVO1NOrrjoM3r7Q+BK2B+sX4/bLZ+VG5g1q2n
EbFdTHS6pHqtZNHQndTmEKwRfh0RYtzEzOXuO6e1gQY42Tujkof40dhGCIU7TeIG
GHwdFxy1niLkXwtHNjV7lnIOkTbx6+sSPamRfQAlZqUWM2Lf5o+7h3qWP3ENB138
sQIDAQAB
-----END PUBLIC KEY-----"
                    .to_owned(),
                balance: 0,
            },
        );

        db.pending_transactions.write().insert(
            "source_public_key_signature".to_owned(),
            Transaction {
                by: "source_public_key_signature".to_owned(),
                source: "source_public_key_signature".to_owned(),
                target: "target_public_key_signature".to_owned(),
                amount: 3,
                timestamp: chrono::NaiveDate::from_ymd(2021, 04, 13).and_hms(20, 55, 30),
            },
        );

        *db.blockchain.write() = Block {
            transaction_list: vec![
                "foo_public_key_signature".to_owned(),
                "bar_public_key_signature".to_owned(),
                "baz_public_key_signature".to_owned(),
            ],
            nonce: 6920405,
            timestamp: chrono::NaiveDate::from_ymd(2021, 04, 13).and_hms(20, 55, 00),
            hash: "0000009745f2f09c968c095af75e8ab87eba9be90a93e5df464f83ea7ec08537".to_owned(),
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
            by: "mock_transaction_source".to_owned(),
            source: "mock_transaction_source".to_owned(),
            target: "mock_transaction_target".to_owned(),
            amount: 25,
            timestamp: chrono::NaiveDate::from_ymd(2021, 04, 09).and_hms(14, 30, 00),
        }
    }

    /// Test simple GET request to /transaction, an endpoint that exists
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

        let expected_json_body = r#"[{"by":"source_public_key_signature","source":"source_public_key_signature","target":"target_public_key_signature","amount":3,"timestamp":"2021-04-13T20:55:30"}]"#;

        assert_eq!(res.body(), expected_json_body);
    }

    /// Test simple GET request to /block, an enpoint that exists
    ///
    /// https://tools.ietf.org/html/rfc7231#section-6.3.1
    ///
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

        let expected_json_body = r#"{"transaction_list":["foo_public_key_signature","bar_public_key_signature","baz_public_key_signature"],"nonce":6920405,"timestamp":"2021-04-13T20:55:00","hash":"0000009745f2f09c968c095af75e8ab87eba9be90a93e5df464f83ea7ec08537"}"#;
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

    /// Test a POST request to /transaction, an endpoint that exists
    ///
    /// https://tools.ietf.org/html/rfc7231#section-6.3.2
    ///
    /// Should accept the json request, create
    /// the transaction and add it to pending transactions in the db
    #[tokio::test]
    async fn post_auth_json_201() {
        let db = mocked_db();
        let filter = consensus_routes(db.clone());

        let res = warp::test::request()
            .method("POST")
            .json(&mocked_transaction())
            .header("Authorization", "Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJ0aGEiOiI2NjkyZTc3NGViYTdmYjkyZGMwZmU2Y2Y3MzQ3NTkxZSIsImlhdCI6MTYxODI2MDY0MSwiZXhwIjoxNzE4MjYwNjQxfQ.M_FVVE5F_aYcDsprkcqV8n2DAhnM6jImAUEXChI9qYn55meE_0Pmp6AaJlTzclYUT1ZUQfFuehYTYu5UkigQ_AimDhqM5VWxPdnyfTQscV916arbNn4qXW6-3oHGUR93xK7-mX6mxeXyDZLxr1SD_JEvVzGWTU4Xo9SMYSIcaHjROAg_ChxJdD4WLe5T4He7O443jpXdAeeVVYfKoJyBfINx_bxiF58-ni1vur9q6-nrjnMw6sMMbtWD3qvzKZHN7HzfwNXM-90D-9VX1KiaJN05jIxLzCYacLeBUH595I4--XfgpLmqrV_P3Sucmny0yvagbZtjYjswmf0DjR99ug")
            .path("/transaction")
            .reply(&filter)
            .await;

        println!("{:?}", res.body());
        assert_eq!(res.status(), StatusCode::CREATED);
        assert_eq!(db.pending_transactions.read().len(), 2);
    }

    /// Test a POST request to /transaction, an endpoint that exists with an incorrect JWT in the
    /// Authorization header
    ///
    /// https://tools.ietf.org/html/rfc7231#section-6.3.2
    ///
    /// Should reject the request
    #[tokio::test]
    async fn post_auth_json_400() {
        let db = mocked_db();
        let filter = consensus_routes(db.clone());

        let res = warp::test::request()
            .method("POST")
            .json(&mocked_transaction())
            .header(
                "Authorization",
                "Bearer aaaaaaaasdlkjaldkasljdaskjlaaaaaaaaaaaaaa",
            )
            .path("/transaction")
            .reply(&filter)
            .await;

        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        assert_eq!(db.pending_transactions.read().len(), 1);
    }

    /// Test a POST request to /block, an endpoint that exists
    ///
    /// https://tools.ietf.org/html/rfc7231#section-6.3.2
    ///
    /// Should accept the json request, create
    /// the block
    #[tokio::test]
    async fn post_block_auth_201() {
        let db = mocked_db();
        let filter = consensus_routes(db.clone());

        let res = warp::test::request()
            .method("POST")
            .json(&Block {
            transaction_list: vec!["mock_transaction_source".to_owned()],
            nonce: 2686215,
            timestamp: chrono::NaiveDate::from_ymd(2021, 04, 13).and_hms(23, 38, 00),
            hash: "0000007c52e4486359f62b2d19781fafaf059bd691bc6d835b666f6eac1d01d9".to_owned(),
        } )
            .header("Authorization", "Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJ0aGEiOiIyYjY0OGZmYWI1ZDlhZjFkNWQ1ZmMwNTJmYzllNTFiODgyZmM0ZmIwYzk5ODYwOGM5OTIzMmY5MjgyMDAwMDAwIiwiaWF0IjoxNjE4MzYwNjQxLCJleHAiOjE3MTgyNjA2NDF9.P5L_uZ9lOhRZCbsG9GDXn_rmZat3dP9Y2lbk8GY4Kg4pOxJIklBUxot-TtJzB0vEJFcjnxVnT2lFLCgfdQLHTJvURiW0KRHi94e1Kj8aDXxJ0qjlq4-c1JCZnAIbDpvkFtHNKz04yfyeSR2htJ6kOjlqVpeUhLVokHhi1x-ZUZZSpeGnlIXgi-AcmkEoyOypZGSZgQ1hjID2f18zgfbshgPK4Dr0hiN36wYMB0y0YiikRbvDuGgDzRLN2nitih46-CXTGZMqIRz3eAfM2wuUSH1yhdKi5_vavz8L3EPVCGMO-CKlPUDkYA-duQZf_q3tG2fkdaFlTAcCik_kVMprdw")
            .path("/block")
            .reply(&filter)
            .await;

        // should be reflectled on the db as well
        assert_eq!(
            *db.blockchain.read().hash,
            "0000007c52e4486359f62b2d19781fafaf059bd691bc6d835b666f6eac1d01d9".to_owned()
        );
        assert_eq!(res.status(), StatusCode::CREATED);
    }

    /// Test a POST request to /block, an endpoint that exists
    ///
    /// https://tools.ietf.org/html/rfc7231#section-6.3.2
    ///
    /// Should reject the block because there aren't enough zeroes in the hash
    #[tokio::test]
    async fn post_block_wrong_hash() {
        let db = mocked_db();
        let filter = consensus_routes(db.clone());

        let res = warp::test::request()
            .method("POST")
            .header("Authorization", "Bearer foo.bar.baz")
            .json(&Block {
                transaction_list: vec!["foobarbaz".to_owned(), "dazsaz".to_owned()],
                nonce: 1000, // not valid
                timestamp: chrono::NaiveDate::from_ymd(2021, 04, 12).and_hms(05, 29, 30),
                hash: "tnarstnarsuthnarsthlarjstk".to_owned(),
            })
            .path("/block")
            .reply(&filter)
            .await;

        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    }

    /// Test a POST request to /block, an endpoint that exists
    ///
    /// https://tools.ietf.org/html/rfc7231#section-6.3.2
    ///
    /// Should reject the block because transaction list is empty
    #[tokio::test]
    async fn post_block_with_empty_transaction_list() {
        let db = mocked_db();
        let filter = consensus_routes(db.clone());

        let res = warp::test::request()
            .method("POST")
            .header("Authorization", "Bearer foo.bar.baz")
            .json(&Block {
                transaction_list: vec![],
                nonce: 1000, // not valid
                timestamp: chrono::NaiveDate::from_ymd(2021, 04, 12).and_hms(05, 29, 30),
                hash: "thisisnotavalidhash".to_owned(),
            })
            .path("/block")
            .reply(&filter)
            .await;

        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    }

    /// Test a POST request to /block, an endpoint that exists
    ///
    /// https://tools.ietf.org/html/rfc7231#section-6.3.2
    ///
    /// Should reject the block because hash has enough zeroes but is not the actual hash of the
    /// block
    #[tokio::test]
    async fn post_block_incorrect_hash() {
        let db = mocked_db();
        let filter = consensus_routes(db.clone());

        let res = warp::test::request()
            .method("POST")
            .json(&Block {
                transaction_list: vec![],
                nonce: 12314,
                timestamp: chrono::NaiveDate::from_ymd(2021, 04, 13).and_hms(20, 55, 00),
                hash: "0000001111111111111111111111111111111111111111111111111111111111".to_owned(),
            })
            .path("/block")
            .reply(&filter)
            .await;

        println!("{:?}", res.body());
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            db.blockchain.read().hash,
            "0000009745f2f09c968c095af75e8ab87eba9be90a93e5df464f83ea7ec08537"
        );
    }

    /// Test a POST request to /register, an endpoint that exists
    ///
    /// https://tools.ietf.org/html/rfc7231#section-6.3.2
    ///
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
        assert_eq!(db.users.read().len(), 2);
    }

    /// Test a POST request to /transaction, an endpoint that exists
    /// https://tools.ietf.org/html/rfc7231#section-6.3.2
    /// Should NOT accept the json request as the user is unpriviliged
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
        assert_eq!(db.users.read().len(), 1);
    }

    /// Test a POST request to /transaction, an endpoint that exists with a longer than expected
    /// payload
    ///
    /// https://tools.ietf.org/html/rfc7231#section-6.5.11
    ///
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

// TODO: POST block without correct transactions test <09-04-21, yigit> //
// TODO: POST transaction while that source has pending transaction test <09-04-21, yigit> //
