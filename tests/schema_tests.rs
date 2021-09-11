// #[cfg(test)]
// mod tests {
//     use drocoin::schema::*;
//     use serde_test::{assert_tokens, Token};
//     use chrono::NaiveDate;

//     #[test]
//     fn claims_serialize_correctly() {
//         let claims = Claims {
//             tha: "hashed_string".to_owned(),
//             iat: 0,
//             exp: 100,
//         };
//         assert_tokens(
//             &claims,
//             &[
//                 Token::Struct{name: "Claims", len: 3},
//                 Token::String("tha"),
//                 Token::String("hashed_string"),
//                 Token::String("iat"),
//                 Token::U64(0),
//                 Token::String("exp"),
//                 Token::U64(100),
//                 Token::StructEnd,
//             ]
//         )
//     }

//     #[test]
//     fn claims_deserialize_correctly() {
//         let data = r#"{"tha":"hashed_string","iat":0,"exp":100}"#;
//         let claims: Claims = serde_json::from_str(data).unwrap();
//         let expected_claims = Claims {
//             tha: "hashed_string".to_owned(),
//             iat: 0,
//             exp: 100,
//         };
//         assert_eq!(claims, expected_claims);
//     }

//     #[test]
//     fn transaction_serialize_correctly() {
//         let transaction = Transaction {
//             by: "source".to_owned(),
//             source: "source".to_owned(),
//             target: "target".to_owned(),
//             amount: 0,
//             timestamp: NaiveDate::from_ymd(2021, 4, 2).and_hms(4, 2, 42),
//         };

//         assert_tokens(
//             &transaction,
//             &[
//                 Token::Struct{name: "Transaction", len: 5},
//                 Token::String("by"),
//                 Token::String("source"),
//                 Token::String("source"),
//                 Token::String("source"),
//                 Token::String("target"),
//                 Token::String("target"),
//                 Token::String("amount"),
//                 Token::I32(0),
//                 Token::String("timestamp"),
//                 Token::String("2021-04-02T04:02:42"),
//                 Token::StructEnd,
//             ]
//         )
//     }

//     #[test]
//     fn transaction_deserialize_correctly() {
//         let data = r#"{"by":"source","source":"source","target":"target","amount":0,"timestamp":"2021-04-02T04:02:42"}"#;
//         let transaction: Transaction = serde_json::from_str(data).unwrap();
//         let expected_transaction = Transaction {
//             by: "source".to_owned(),
//             source: "source".to_owned(),
//             target: "target".to_owned(),
//             amount: 0,
//             timestamp: NaiveDate::from_ymd(2021, 4, 2).and_hms(4, 2, 42),
//         };

//         assert_eq!(transaction, expected_transaction);
//     }

//     #[test]
//     fn block_serialize_correctly() {
//         let block = Block {
//             transaction_list: vec!["transaction1".to_owned()],
//             nonce: 0,
//             timestamp: NaiveDate::from_ymd(2021, 4, 2).and_hms(4, 2, 42),
//             hash: "hash".to_owned()
//         };

//         assert_tokens(
//             &block,
//             &[
//                 Token::Struct{name: "Block", len: 4},
//                 Token::String("transaction_list"),
//                 Token::Seq {len: Some(1)},
//                 Token::String("transaction1"),
//                 Token::SeqEnd,
//                 Token::String("nonce"),
//                 Token::U32(0),
//                 Token::String("timestamp"),
//                 Token::String("2021-04-02T04:02:42"),
//                 Token::String("hash"),
//                 Token::String("hash"),
//                 Token::StructEnd,
//             ]
//         )
//     }

//     #[test]
//     fn block_deserialize_correctly() {
//         let expected_block = Block {
//             transaction_list: vec!["transaction1".to_owned()],
//             nonce: 0,
//             timestamp: NaiveDate::from_ymd(2021, 4, 2).and_hms(4, 2, 42),
//             hash: "hash".to_owned()
//         };
//         let data = r#"{"transaction_list":["transaction1"],"nonce":0,"timestamp":"2021-04-02T04:02:42","hash":"hash"}"#;
//         let block: Block = serde_json::from_str(data).unwrap();

//         assert_eq!(block, expected_block);

//     }

//     #[test]
//     fn block_serialize_when_vec_emptpy() {
//         let block = Block {
//             transaction_list: vec![],
//             nonce: 0,
//             timestamp: NaiveDate::from_ymd(2021, 4, 2).and_hms(4, 2, 42),
//             hash: "hash".to_owned()
//         };

//         let json = serde_json::to_string(&block).unwrap();
//         assert_eq!(json, r#"{"nonce":0,"timestamp":"2021-04-02T04:02:42","hash":"hash"}"#)
//     }

//     #[test]
//     fn naked_block_serialize_correctly() {
//         let naked_block = NakedBlock {
//             transaction_list: vec!["transaction1".to_owned()],
//             nonce: 0,
//             timestamp: NaiveDate::from_ymd(2021, 4, 2).and_hms(4, 2, 42),
//         };

//         assert_tokens(
//             &naked_block,
//             &[
//                 Token::Struct{name: "NakedBlock", len: 3},
//                 Token::String("transaction_list"),
//                 Token::Seq {len: Some(1)},
//                 Token::String("transaction1"),
//                 Token::SeqEnd,
//                 Token::String("nonce"),
//                 Token::U32(0),
//                 Token::String("timestamp"),
//                 Token::String("2021-04-02T04:02:42"),
//                 Token::StructEnd,
//             ]
//         )
//     }

//     #[test]
//     fn naked_block_deserialize_correctly() {
//         let expected_naked_block = NakedBlock {
//             transaction_list: vec!["transaction1".to_owned()],
//             nonce: 0,
//             timestamp: NaiveDate::from_ymd(2021, 4, 2).and_hms(4, 2, 42),
//         };
//         let data = r#"{"transaction_list":["transaction1"],"nonce":0,"timestamp":"2021-04-02T04:02:42"}"#;
//         let naked_block: NakedBlock = serde_json::from_str(data).unwrap();

//         assert_eq!(naked_block, expected_naked_block);

//     }

//     #[test]
//     fn naked_block_serialize_when_vec_emptpy() {
//         let naked_block = NakedBlock {
//             transaction_list: vec![],
//             nonce: 0,
//             timestamp: NaiveDate::from_ymd(2021, 4, 2).and_hms(4, 2, 42),
//         };

//         let json = serde_json::to_string(&naked_block).unwrap();
//         assert_eq!(json, r#"{"nonce":0,"timestamp":"2021-04-02T04:02:42"}"#)
//     }

//     #[test]
//     fn user_serialize_correctly() {
//         let user = User {
//             user_id: MetuId::new("e254275".to_owned(), "DtNX1qk4YF4saRH".to_owned()).unwrap(),
//             public_key: "public_key".to_owned(),
//             balance: 0
//         };

//         assert_tokens(
//             &user,
//             &[
//                 Token::Struct{name: "User", len: 3},
//                 Token::String("user_id"),
//                 Token::Struct {name: "MetuId", len: 2},
//                 Token::String("id"),
//                 Token::String("e254275"),
//                 Token::String("passwd"),
//                 Token::String("DtNX1qk4YF4saRH"),
//                 Token::StructEnd,
//                 Token::String("public_key"),
//                 Token::String("public_key"),
//                 Token::String("balance"),
//                 Token::I32(0),
//                 Token::StructEnd,
//             ]
//         )
//     }

//     #[test]
//     fn user_deserialize_correctly() {
//         let expected_user = User {
//             user_id: MetuId::new("e254275".to_owned(), "DtNX1qk4YF4saRH".to_owned()).unwrap(),
//             public_key: "public_key".to_owned(),
//             balance: 0
//         };
//         let data = r#"{"user_id":{"id":"e254275","passwd":"DtNX1qk4YF4saRH"},"public_key":"public_key","balance":0}"#;
//         let user: User = serde_json::from_str(data).unwrap();

//         assert_eq!(user, expected_user);

//     }

//     #[test]
//     fn metu_id_serialize_correctly() {
//         let metu_id = MetuId::new ("e254275".to_owned(), "DtNX1qk4YF4saRH".to_owned()).unwrap();

//         assert_tokens(
//             &metu_id,
//             &[
//                 Token::Struct{name: "MetuId", len: 2},
//                 Token::String("id"),
//                 Token::String("e254275"),
//                 Token::String("passwd"),
//                 Token::String("DtNX1qk4YF4saRH"),
//                 Token::StructEnd,
//             ]
//         )
//     }

//     #[test]
//     fn metu_id_deserialize_correctly() {
//         let expected_metu_id = MetuId::new ("e254275".to_owned(), "DtNX1qk4YF4saRH".to_owned()).unwrap();
//         let data = r#"{"id":"e254275","passwd":"DtNX1qk4YF4saRH"}"#;
//         let metu_id: MetuId = serde_json::from_str(data).unwrap();

//         assert_eq!(metu_id, expected_metu_id);
//     }

//     #[test]
//     fn auth_request_serialize_correctly() {
//         let auth_request = AuthRequest {
//             student_id: "e254275".to_owned(),
//             passwd: "DtNX1qk4YF4saRH".to_owned(),
//             public_key: "public_key".to_owned()
//         };

//         assert_tokens(
//             &auth_request,
//             &[
//                 Token::Struct{name: "AuthRequest", len: 3},
//                 Token::String("student_id"),
//                 Token::String("e254275"),
//                 Token::String("passwd"),
//                 Token::String("DtNX1qk4YF4saRH"),
//                 Token::String("public_key"),
//                 Token::String("public_key"),
//                 Token::StructEnd,
//             ]
//         )
//     }

//     #[test]
//     fn auth_request_deserialize_correctly() {
//         let expected_auth_request = AuthRequest {
//             student_id: "e254275".to_owned(),
//             passwd: "DtNX1qk4YF4saRH".to_owned(),
//             public_key: "public_key".to_owned()
//         };
//         let data = r#"{"student_id":"e254275","passwd":"DtNX1qk4YF4saRH","public_key":"public_key"}"#;
//         let auth_request: AuthRequest = serde_json::from_str(data).unwrap();

//         assert_eq!(auth_request, expected_auth_request);

//     }














// }
