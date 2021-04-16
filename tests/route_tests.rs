#[cfg(test)]
mod tests {
    use gradecoin::schema::{Block, Db, InitialAuthRequest, MetuId, Transaction, User};

    use gradecoin::routes::consensus_routes;
    use warp::http::StatusCode;

    /// Create a mock database to be used in tests
    fn mocked_db() -> Db {
        let db = Db::new();

        db.users.write().insert(
            "mock_transaction_source".to_owned(),
            User {
                user_id: MetuId::new("e254275".to_owned(), "DtNX1qk4YF4saRH".to_owned()).unwrap(),
                public_key: "-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA5yWTMeFqr2nvOC9oR5Wq
/nzcNlwCIaziojt7rJ4BBvuwkT0tERDz8AgvUsaewiB+Fz5OXTeb3WAB1FEXnBXG
ekrGzvC8jHQMKHyNoWzUlpQJ9UMtdQIWPOCuMyLpc+rNPL3428U8UpldjbTHHyq2
/ef6abkdj+XWg/slYtrFeOf3ktc1l50R4k8VO8L6kQuh2+YIjXGPLShRaqnUQPtH
8LFPX4bO9lJ9mAoMZFec6XVwumn/uqu9jyWQL6qh6gtwQHgN+A9wGvzVvltJ9h8s
shSHWWtBD0M19ilbXhKyBsHSSZkpx+TAvFhfQ8JURw7KqahUPVlCwJ5OIKccJ/6F
FQIDAQAB
-----END PUBLIC KEY-----"
                    .to_owned(),
                balance: 30,
            },
        );

        db.users.write().insert(
            "mock_transaction_source2".to_owned(),
            User {
                user_id: MetuId::new("e254275".to_owned(), "DtNX1qk4YF4saRH".to_owned()).unwrap(),
                public_key: "-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA5yWTMeFqr2nvOC9oR5Wq
/nzcNlwCIaziojt7rJ4BBvuwkT0tERDz8AgvUsaewiB+Fz5OXTeb3WAB1FEXnBXG
ekrGzvC8jHQMKHyNoWzUlpQJ9UMtdQIWPOCuMyLpc+rNPL3428U8UpldjbTHHyq2
/ef6abkdj+XWg/slYtrFeOf3ktc1l50R4k8VO8L6kQuh2+YIjXGPLShRaqnUQPtH
8LFPX4bO9lJ9mAoMZFec6XVwumn/uqu9jyWQL6qh6gtwQHgN+A9wGvzVvltJ9h8s
shSHWWtBD0M19ilbXhKyBsHSSZkpx+TAvFhfQ8JURw7KqahUPVlCwJ5OIKccJ/6F
FQIDAQAB
-----END PUBLIC KEY-----"
                    .to_owned(),
                balance: 30,
            },
        );

        /*-----BEGIN RSA PRIVATE KEY-----
        MIIEpAIBAAKCAQEA5yWTMeFqr2nvOC9oR5Wq/nzcNlwCIaziojt7rJ4BBvuwkT0t
        ERDz8AgvUsaewiB+Fz5OXTeb3WAB1FEXnBXGekrGzvC8jHQMKHyNoWzUlpQJ9UMt
        dQIWPOCuMyLpc+rNPL3428U8UpldjbTHHyq2/ef6abkdj+XWg/slYtrFeOf3ktc1
        l50R4k8VO8L6kQuh2+YIjXGPLShRaqnUQPtH8LFPX4bO9lJ9mAoMZFec6XVwumn/
        uqu9jyWQL6qh6gtwQHgN+A9wGvzVvltJ9h8sshSHWWtBD0M19ilbXhKyBsHSSZkp
        x+TAvFhfQ8JURw7KqahUPVlCwJ5OIKccJ/6FFQIDAQABAoIBADTZGnZlG4dPqSon
        bKgxSA83bQHgt3wLkyWUhApLdeCq2wvZ+NvWDG/s7yT11IZ991ZJIJGfjTtoIALz
        J3rAX8jGH/5gfDuArOb000z9HP3wivZQjawa9gqlNC7s5INkQ9iHdsaIqeoYtpMX
        qg8uLPiQeWiCsoeb/Rff7ARWEKA7udoZ2uZcZFMHTKx+mBpk8IiepQAJPBRVwmXk
        x/3LTaezi6Tkvp/k/gf4IeSICiRGFRmm2Vxciduj11/CrdTHPQLz/Rh5/IN8Bkry
        xdQdQxxhwxF/ap6OJIJyguq7gximn2uK0jbHY3nRmrF8SsEtIT+Gd7I46L/goR8c
        jQOQRmECgYEA9RJSOBUkZMLoUcC2LGJBZOAnJZ7WToCVdu3LrPceRYtQHwcznW4O
        NAHF+blQRzqvbMi11ap8NVpkDDu0ki/Yi2VdSVjQmlaOcpAXjN6T5ZrKoz61xj4g
        2T2/K6d6ypkZRKPhKCC1iI419rq/APVEZHYCl7jZp4iD2izHiegZYccCgYEA8XRK
        rfVuPiYsaB07eJrRKKjuoM1Jcr19jZyXY8sbALRcExaTX2CRaPA7binVeDBXayQ1
        I0+kA1nV1EI+ROegV+b6gs2YaUmMJzI1yLqMqGDgHFxFvhkDsZaI+/V+G9eOLEt4
        5ic5tImfZITLE/GSC8b+C16gxMGUN4t9gHq2okMCgYAKyNedaDDFzl3y2wwpP9mo
        2sReP3Mm2Tm6lhRUdDt8y/impOZ8kw9E8p8HskP6HncBzoNR98KnhmbIswfrNvfM
        ipVkWOg1IoH6QKUIqfLQM9OfA290Xd+ML89t2Fzq9XnLL3sFDQtwCvIM/YLSQ/jS
        gu7yRkwttzA2NapCQ1h6mQKBgQClwBwn8Qyd01y2mCKkNzsP+2/cqTAbeSNAXFe8
        pMfDowx1+hBu7/7CF+/kPwmQuTa5kSB9PgWsWzYjwNm4OX1j+mbL9lEDLf7tRVWQ
        lydJyz7tmRYzWj6j4V/l/u90M3QgyiqTbCf73GG0AkjaRwHn3dG1gl9A0lZqDvK3
        iQXouwKBgQCrx6SCnEkhLISSZpzdDehtWmyCQJIwcdlRQlAmFLVn+TJHTXR7xUm2
        VpTrPTfaYWx83OQUn/OZqY5gIQ+jlfwqnVg+PDQQ/P09/4xygRCLvjL6NCSvtkj1
        MRArEl4y68+jZLRu74TVG0lXi6ht6KhNHF6GiWKU9FHZ4B+btLicsg==
        -----END RSA PRIVATE KEY-----*/

        db.pending_transactions.write().insert(
            "mock_transaction_source".to_owned(),
            Transaction {
                by: "mock_transaction_source".to_owned(),
                source: "31415926535897932384626433832795028841971693993751058209749445923"
                    .to_owned(),
                target: "mock_transaction_source".to_owned(),
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
    fn mocked_transaction2() -> Transaction {
        Transaction {
            by: "mock_transaction_source2".to_owned(),
            source: "mock_transaction_source2".to_owned(),
            target: "mock_transaction_target".to_owned(),
            amount: 25,
            timestamp: chrono::NaiveDate::from_ymd(2021, 04, 09).and_hms(14, 30, 00),
        }
    }
    // r#"{"by":"mock_transaction_source","source":"mock_transaction_source","target":"mock_transaction_target","amount":25,"timestamp":"2021-04-09T14:30:00"}"#

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

        let expected_json_body = r#"{"mock_transaction_source":{"by":"mock_transaction_source","source":"31415926535897932384626433832795028841971693993751058209749445923","target":"mock_transaction_source","amount":3,"timestamp":"2021-04-13T20:55:30"}}"#;

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
            .json(&mocked_transaction2())
            .header("Authorization", "Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJ0aGEiOiI2NDNmOGYzMjkxNTMzNTMzOTcwMDZmNjVmOWQ1ZmViMyIsImlhdCI6MTUxNjIzOTAyMiwiZXhwIjoyMDE2MjM5MDIyfQ.yNca1RJOkSEKoF7S4PSF4iB8zmnj13ujcfsdVRcJMcQNN4CxP-pJwbUBdRgR8kNwdfLP3nLo0UBwevP42TBoujMKx7oaIl-JXsO37x7Y9GWMAHYBxEOoq1EsBeaxv9pCdyZvuVeJYIMrOpzW7oTcF4tHHvmvySD2ITnQTWu_ioCXEFdX21QQIvsqpRn7XumfCMvWfUy_C2XTFIQEAGdakPmkZ2Xt66k9zhT9hazJgAwELv5VyMV54iF8vyvvmnLkiODwTt_8VdqC6fr6jPwYaP1mzgd58r0fM76Wu0g9tIXVU83rcFMRsm_faXGbsrDJIQ06-fAO_D1sh74fhndK_g")
            .path("/transaction")
            .reply(&filter)
            .await;

        println!("{:?}", res.body());
        assert_eq!(res.status(), StatusCode::CREATED);
        for i in db.pending_transactions.read().iter() {
            println!("{:?}", i);
        }
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
        println!("Wtf");
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
            .header("Authorization", "Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJ0aGEiOiIwMDAwMDA3YzUyZTQ0ODYzNTlmNjJiMmQxOTc4MWZhZmFmMDU5YmQ2OTFiYzZkODM1YjY2NmY2ZWFjMWQwMWQ5IiwiaWF0IjoxMDAwMDAwMDAwMCwiZXhwIjoxMDkwMDAwMDAwMH0.JrzYlamBnT3qcjttzLTLXpiO5qfBu1e2HjQWueJ8l__aas6I1xq77UO8kCWn2Sm-zwUOI_155Pbd4xAqL6pokjLHZSFnAi9ZJ8cpqgw4ZXdI-Z3tDpZMUSiI018CGMZQZ_BwdGDIBbjEy0P-MX590DW9ofLVZckJKoXU5fFYi47OBegh4-8cchco_Z4wDPVamyhZXo8YmIN_ioSQNBQT2gNJnWsVvsXAQ7IdX9fhwS19t1kdnyk_WlezGbkrQ5xW-XAs4qMCgybbW9ErRwIruxI0PLlILFw2-m-UtH7fGdSIAaG-q6gKy79rPQLEE2kI9I39SVdIfMTadfnu6bduag")
            .path("/block")
            .reply(&filter)
            .await;

        println!("ISSUE: {:?}", res.body());

        // should be reflectled on the db as well
        assert_eq!(
            db.blockchain.read().hash,
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
    // #[tokio::test]
    // async fn post_register_priviliged_user() {
    //     let db = mocked_db();
    //     let filter = consensus_routes(db.clone());

    //     let res = warp::test::request()
    //         .method("POST")
    //         .json(&priviliged_mocked_user())
    //         .path("/register")
    //         .reply(&filter)
    //         .await;

    //     println!("{:?}", res.body());
    //     assert_eq!(res.status(), StatusCode::CREATED);
    //     assert_eq!(db.users.read().len(), 2);
    // }

    /// Test a POST request to /transaction, an endpoint that exists
    /// https://tools.ietf.org/html/rfc7231#section-6.3.2
    /// Should NOT accept the json request as the user is unpriviliged
    // #[tokio::test]
    // async fn post_register_unpriviliged_user() {
    //     let db = mocked_db();
    //     let filter = consensus_routes(db.clone());

    //     let res = warp::test::request()
    //         .method("POST")
    //         .json(&unpriviliged_mocked_user())
    //         .path("/register")
    //         .reply(&filter)
    //         .await;

    //     println!("{:?}", res.body());
    //     assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    //     assert_eq!(db.users.read().len(), 1);
    // }

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

    /// Test the User Authentication Process
    #[tokio::test]
    async fn user_authentication() {
        let db = mocked_db();
        let filter = consensus_routes(db);

        let res = warp::test::request()
            .method("POST")
            .json(&InitialAuthRequest {
                c: "D9OKSp4XD+niltqhoiTEyz3pTxGm5ZKYVNFPofW40M6Km7wE7FgIpfTkurBZ6tQsG/rYPRsd6C/Qo+o3HrgOYC8BDprwpnYb7UnJdL2pe44ZMEsPAmDAdwTP9WozY0lr+bjEjtTM1mVQnIdfknychFek/FNi3l8MrapeFTxFaTMGxWuS1+wEuAkcz4AR4+jooaXVAEpKrPiSXqbywF9OQ41tk0kRiXn234dj40ndND+GlfMgghITuBJrJx6tzLppAZNIIGwUjQDt5Oib5dEGrPOe+rran1D26YNhZOtrfYEGyUSN+/58HbItQlLrgFhL6zRT7ojw/Eg4jYXndK0xNgYGyhAn5UI/qnI2NPpZU7Wd3sJKlWc7HfrjNnKVKlcrhHtYy3FXfN/hLg7SFmuSfXqqvVbNVT6pEDU6Y5NahOYaE/vkL0no7F7lz0UjAlgQCmn5yN7mKs3yLSnlx6hmsK/fVoqGBcOIbYY5gzYMlAQ3E+lq0p2MPEoWC8NYxStSeo9M8uLYT6Jl3hYVf8aLgd1l0HEiCyT+kWxvcR5hw42I7gqaoUcnr53Zm1mYK30/fvZ6lxsrb4FphldgQC5fx6nwEgjaLUeB4n0oZTSRLbrd9ZXCjUG4FNmM+sOklhIXyTYUj4VcBSwZuAvJZEFf2em68e7ySJs/ysz+TGu3eVeRc+voAvI9mGLxWnSEjWx64po7PO61uG6ikadHZH+wIw==".to_owned(),
                iv: "bmV2ZXJtaW5kdGhlbmZ1aw==".to_owned(),
                key: "s4cn9BSmuForX6PxJAa55Es4t2puXuDtdII1lxEArqVlP+uYd5jDKofFtn9PCAoY7jyTgBIhQW7Ah5MGCcufWTaKHAjFVfSZ+qGwbGbBcklbNGH/F7cJ0Pe7kOCddUpIvLG6WH6+mnvyPs8PwDyagsx1Jc2PSSOYLAwkECvPbjiUjQiBixguTRNsU2eKaqzLimPE0w2ztvdA+IgCv94UPhjQfQrnMGK+Ppn3oK7IfKQJ7v2DLVNuz4d/BpwuqD+lYYAu4B4qn3daNR32a/mqAAlPg/RbPlH69N44Qh/NYux90FOY0XKxUskEwsAUw8dHFzzdKPcGx4C0s5e4KSLGkw==".to_owned(),
            })
            .path("/register")
            .reply(&filter)
            .await;

        println!("{:?}", res);
        assert_eq!(res.status(), StatusCode::CREATED);
    }
}

// TODO: POST block without correct transactions test <09-04-21, yigit> //
// TODO: POST transaction while that source has pending transaction test <09-04-21, yigit> //
