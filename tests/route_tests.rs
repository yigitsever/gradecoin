#[cfg(test)]
mod tests {
    use gradecoin::schema::{
        create_database, AuthRequest, Block, Claims, Db, MetuId, Transaction, User,
    };

    use gradecoin::routes::consensus_routes;
    use warp::http::StatusCode;

    use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
    const PRIVATE_KEY_PEM: &str = "-----BEGIN RSA PRIVATE KEY-----
MIIEpAIBAAKCAQEA4nU0G4WjkmcQUx0hq6LQuV5Q+ACmUFL/OjoYMDwC/O/6pCd1
UZgCfgHN2xEffDPznzcTn8OiFRxr4oWyBinyrUpnY4mhy0SQUwoeCw7YkcHAyhCj
NT74aR/ohX0MCj0qRRdbt5ZQXM/GC3HJuXE1ptSuhFgQxziItamn8maoJ6JUSVEX
VO1NOrrjoM3r7Q+BK2B+sX4/bLZ+VG5g1q2nEbFdTHS6pHqtZNHQndTmEKwRfh0R
YtzEzOXuO6e1gQY42Tujkof40dhGCIU7TeIGGHwdFxy1niLkXwtHNjV7lnIOkTbx
6+sSPamRfQAlZqUWM2Lf5o+7h3qWP3ENB138sQIDAQABAoIBAD23nYTmrganag6M
wPFrBSGP79c3Lhx0EjUHQjJbGKFgsdltG48qM3ut+DF9ACy0Z+/7bbC7+39vaIOq
1jLR2d6aiYTaLKseO4s2FawD1sgamvU3BZPsXn0gAhnnU5Gyy8Nas1dccvhoc9wI
neaZUPrvucQ90AzLfo6r9yacDbYHB1lOyomApUvpJxOgHISGEtc9qGPDrdH19aF0
8fCv2bbQRh+TChgN3IB0o5w0wXaI7YAyAouAv/AzHCoEMpt7OGjFTkjh/ujlPL9O
+FLuJNsQRHDN0gJo2pcvwGwDCsioMixQ9bZ7ZrUu2BNpEQygyeSbj9ZI1iRvhosO
JU3rwEECgYEA9MppTYA6A9WQbCCwPH1QMpUAmPNVSWVhUVag4lGOEhdCDRcz9ook
DohQMKctiEB1luKuvDokxo0uMOfMO9/YwjsRB7qjQip7Th1zMJIjD+A+juLzHK4r
/RiRtWYGAnF8mptDvE+93JsPb3C/lQLvIhio5GQYWBqPJu6SpeosIskCgYEA7NPi
Gbffzr2UQhW8BNKmctEEh8yFRVojFo3wwwWxSNUVXGSmSm31CL+Q8h817R+2OkPV
1ZMUOBU4UJiqFt28kIvTDFqbAJlJQGCpY2mY7OLQiD2A+TVLcFrHmoCaPfCAK1Qd
hQ0PmFK7Mf8qClpA3E5chop/WfKQfiu46sZv1qkCgYAhGdXPcw1lQ1W6KVlrdI6J
qHhiNlVMDXdxZkNvFxQdAiQeXQrbxaZGiMw/J/wSNpUwCAsUzM/4QVMDrfSCDCzl
ZtNQtj4pTlFKKNVQthIjrXEIJUw2jp7IJLBfVSJu5iWxSlmId0f3MsiNizN81N69
P5Rm/doE3+KHoy8VXGsHcQKBgQCkNh62enqjHWypjex6450qS6f6iWN3PRLLVsw0
TcQpniZblCaBwVCAKmRUnjOEIdL2/4ZLutnwMTaFG/YEOOfAylMiY8jKV38lNmD9
X4D78CFr9klxgvS2CRwSE03f2NzmLkLxuKaxldvaxPTfjMkgeO1LFMlNExYBhkuH
7uQpUQKBgQCKX6qMNh2gSdgG7qyxfTFZ4y5EGOBoKe/dE+IcVF3Vnh6DZVbCAbBL
5EdFWZSrCnDjA4xiKW55mwp95Ud9EZsZAb13L8V9t82eK+UDBoWlb7VRNYpda/x1
5/i4qQJ28x2UNJDStpYFpnp4Ba1lvXjKngIbDPkjU+hbBJ+BNGAIeg==
-----END RSA PRIVATE KEY-----";

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
            "hash_value".to_owned(),
            Transaction {
                by: "source_account".to_owned(),
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
            nonce: 0,
            timestamp: chrono::NaiveDate::from_ymd(2021, 04, 08).and_hms(12, 30, 30),
            hash: "not_a_thing_yet".to_owned(),
        };

        db
    }

    fn mocked_jwt() -> String {
        let claims = Claims {
            tha: "6692e774eba7fb92dc0fe6cf7347591e".to_owned(),
            iat: 1618275851,
            exp: 1648275851,
        };
        let header = Header::new(Algorithm::RS256);
        encode(
            &header,
            &claims,
            &EncodingKey::from_rsa_pem(PRIVATE_KEY_PEM.as_bytes()).unwrap(),
        )
        .unwrap()
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

    /// Create a mock block with a correct mined hash to be used in tests
    fn mocked_block() -> Block {
        Block {
            transaction_list: vec!["hash_value".to_owned()],
            nonce: 3831993,
            timestamp: chrono::NaiveDate::from_ymd(2021, 04, 08).and_hms(12, 30, 30),
            hash: "2b648ffab5d9af1d5d5fc052fc9e51b882fc4fb0c998608c99232f9282000000".to_owned(),
        }
    }

    /// Create a mock block with a wrong hash and nonce
    fn mocked_wrong_block() -> Block {
        Block {
            transaction_list: vec!["foobarbaz".to_owned(), "dazsaz".to_owned()],
            nonce: 1000, // can you imagine
            timestamp: chrono::NaiveDate::from_ymd(2021, 04, 12).and_hms(05, 29, 30),
            hash: "tnarstnarsuthnarsthlarjstk".to_owned(),
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

        let expected_json_body = r#"[{"by":"source_account","source":"source_account","target":"target_account","amount":20,"timestamp":"2021-04-09T01:30:30"}]"#;

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

        let expected_json_body = r#"{"transaction_list":["old_transaction_hash_1","old_transaction_hash_2","old_transaction_hash_3"],"nonce":0,"timestamp":"2021-04-08T12:30:30","hash":"not_a_thing_yet"}"#;
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
    async fn post_block_201() {
        let db = mocked_db();
        let filter = consensus_routes(db.clone());

        let res = warp::test::request()
            .method("POST")
            .json(&mocked_block())
            .path("/block")
            .reply(&filter)
            .await;

        assert_eq!(res.status(), StatusCode::CREATED);
        assert_eq!(
            *db.blockchain.read().hash,
            "2b648ffab5d9af1d5d5fc052fc9e51b882fc4fb0c998608c99232f9282000000".to_owned()
        );
    }

    /// Test a POST request to /block, an endpoint that exists
    ///
    /// https://tools.ietf.org/html/rfc7231#section-6.3.2
    ///
    /// Should reject the block because of the wrong hash/nonce
    /// // TODO: split this into two tests
    #[tokio::test]
    async fn post_block_wrong_hash() {
        let db = mocked_db();
        let filter = consensus_routes(db.clone());

        let res = warp::test::request()
            .method("POST")
            .json(&mocked_wrong_block())
            .path("/block")
            .reply(&filter)
            .await;

        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
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
