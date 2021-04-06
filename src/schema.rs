// Common types used across API

use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

// use crate::validators;

pub fn ledger() -> Db {
    // TODO: there was something simpler in one of the other tutorials? <07-04-21, yigit> //

    Arc::new(Mutex::new(vec![
    Transaction {
        source: String::from("Myself"),
        target: String::from("Nobody"),
        amount: 4,
        timestamp: NaiveDate::from_ymd(2021, 4, 7).and_hms(00, 17, 00),
    },
    ]))
}


// For presentation purposes keep mocked data in in-memory structure
// In real life scenario connection with regular database would be established

pub type Db = Arc<Mutex<Vec<Transaction>>>;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Transaction {
    pub source: String,
    pub target: String,
    pub amount: i32,
    pub timestamp: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Block {
    pub transaction_list: Vec<Transaction>, // [Transaction; N]
    pub nonce: i32,
    pub timestamp: NaiveDateTime,
    pub hash: String, // future proof'd baby
}

// #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
// #[serde(rename_all = "camelCase")]
// pub struct Game {
//     pub id: u64,
//     pub title: String,
//     #[serde(with = "validators::validate_game_rating")]
//     pub rating: u8,
//     pub genre: Genre,
//     pub description: Option<String>,
//     pub release_date: NaiveDateTime,
// }

// #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
// #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
// pub enum Genre {
//     RolePlaying,
//     Strategy,
//     Shooter,
// }

// #[derive(Deserialize, Debug, PartialEq)]
// pub struct ListOptions {
//     pub offset: Option<usize>,
//     pub limit: Option<usize>,
// }

// pub fn example_db() -> Db {
//     Arc::new(Mutex::new(
//         vec![
//         Game {
//             id: 1,
//             title: String::from("Dark Souls"),
//             rating: 91,
//             genre: Genre::RolePlaying,
//             description: Some(String::from("Takes place in the fictional kingdom of Lordran, where players assume the role of a cursed undead character who begins a pilgrimage to discover the fate of their kind.")),
//             release_date: NaiveDate::from_ymd(2011, 9, 22).and_hms(0, 0, 0),
//         },
//         Game {
//             id: 2,
//             title: String::from("Dark Souls 2"),
//             rating: 87,
//             genre: Genre::RolePlaying,
//             description: None,
//             release_date: NaiveDate::from_ymd(2014, 3, 11).and_hms(0, 0, 0),
//         },
//         Game {
//             id: 3,
//             title: String::from("Dark Souls 3"),
//             rating: 89,
//             genre: Genre::RolePlaying,
//             description: Some(String::from("The latest chapter in the series with its trademark sword and sorcery combat and rewarding action RPG gameplay.")),
//             release_date: NaiveDate::from_ymd(2016, 3, 24).and_hms(0, 0, 0),
//         },
//     ]
//     ))
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     use serde_json::error::Error;
//     use serde_test::{assert_tokens, Token};

//     #[test]
//     fn game_serialize_correctly() {
//         let game = Game {
//             id: 1,
//             title: String::from("Test"),
//             rating: 90,
//             genre: Genre::Shooter,
//             description: None,
//             release_date: NaiveDate::from_ymd(2019, 11, 12).and_hms(0, 0, 0),
//         };

//         assert_tokens(
//             &game,
//             &[
//                 Token::Struct {
//                     name: "Game",
//                     len: 6,
//                 },
//                 Token::String("id"),
//                 Token::U64(1),
//                 Token::String("title"),
//                 Token::String("Test"),
//                 Token::String("rating"),
//                 Token::U8(90),
//                 Token::String("genre"),
//                 Token::UnitVariant {
//                     name: "Genre",
//                     variant: "SHOOTER",
//                 },
//                 Token::String("description"),
//                 Token::None,
//                 Token::String("releaseDate"),
//                 Token::String("2019-11-12T00:00:00"),
//                 Token::StructEnd,
//             ],
//         );
//     }

//     #[test]
//     fn game_deserialize_correctly() {
//         let data = r#"{"id":3,"title":"Another game","rating":65,"genre":"STRATEGY","description":null,"releaseDate":"2016-03-11T00:00:00"}"#;
//         let game: Game = serde_json::from_str(data).unwrap();
//         let expected_game = Game {
//             id: 3,
//             title: String::from("Another game"),
//             rating: 65,
//             genre: Genre::Strategy,
//             description: None,
//             release_date: NaiveDate::from_ymd(2016, 3, 11).and_hms(0, 0, 0),
//         };

//         assert_eq!(game, expected_game);
//     }

//     #[test]
//     fn game_error_when_wrong_rating_passed() {
//         let data = r#"{"id":3,"title":"Another game","rating":120,"genre":"STRATEGY","description":null,"releaseDate":"2016-03-11T00:00:00"}"#;
//         let err: Error = serde_json::from_str::<Game>(data).unwrap_err();

//         assert_eq!(err.is_data(), true);
//     }

//     #[test]
//     fn genre_serialize_correctly() {
//         let genre = Genre::Shooter;
//         assert_tokens(
//             &genre,
//             &[Token::UnitVariant {
//                 name: "Genre",
//                 variant: "SHOOTER",
//             }],
//         );

//         let genre = Genre::RolePlaying;
//         assert_tokens(
//             &genre,
//             &[Token::UnitVariant {
//                 name: "Genre",
//                 variant: "ROLE_PLAYING",
//             }],
//         );

//         let genre = Genre::Strategy;
//         assert_tokens(
//             &genre,
//             &[Token::UnitVariant {
//                 name: "Genre",
//                 variant: "STRATEGY",
//             }],
//         );
//     }

//     #[test]
//     fn genre_deserialize_correctly() {
//         let data = r#""SHOOTER""#;
//         let genre: Genre = serde_json::from_str(data).unwrap();
//         let expected_genre = Genre::Shooter;

//         assert_eq!(genre, expected_genre);

//         let data = r#""ROLE_PLAYING""#;
//         let genre: Genre = serde_json::from_str(data).unwrap();
//         let expected_genre = Genre::RolePlaying;

//         assert_eq!(genre, expected_genre);
//     }

//     #[test]
//     fn genre_error_when_wrong_rating_passed() {
//         let data = r#""SPORT""#;
//         let err: Error = serde_json::from_str::<Genre>(data).unwrap_err();

//         assert_eq!(err.is_data(), true);
//     }
// }
