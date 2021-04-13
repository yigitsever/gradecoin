//! # Data Representations
//!
//! We need persistence for [`Block`]s and [`User`]s, not so much for [`Transaction`]s
//!
//! There are around 30 students, a full fledged database would be an overkill (for next year?)
//!
//! Pending transactions are held in memory, these are cleared with every new block
//! Only the last block is held in memory, every block is written to a file
//! Users are held in memory and they're also backed up to text files
use chrono::{NaiveDate, NaiveDateTime};
use lazy_static::lazy_static;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fs;
use std::sync::Arc;

// use crate::validators;

pub type PublicKeySignature = String;

/// Creates a new database
pub fn create_database() -> Db {
    fs::create_dir_all("blocks").unwrap();
    fs::create_dir_all("users").unwrap();
    Db::new()
}

/// A JWT Payload/Claims representation
///
/// https://tools.ietf.org/html/rfc7519#section-4.1
///
/// - `tha`: Transaction Hash, String (custom field)
/// - `iat`: Issued At, Unix Time, epoch
/// - `exp`: Expiration Time, epoch
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub tha: String,
    pub iat: usize,
    pub exp: usize,
}

/// Global Database representation
///
/// [`Db::blockchain`] is just the last block that was mined. All the blocks are written to disk as text
/// files whenever they are accepted.
///
/// [`Db::pending_transactions`] is the in memory representation of the waiting transactions. Every
/// user can have only one outstanding transaction at any given time.
///
/// [`Db::users`] is the in memory representation of the users, with their public keys, metu_ids and
/// gradecoin balances.
///
/// TODO: Replace the pending_transactions HashMap<String, Transaction> with
/// HashMap<PublicKeySignature, Transaction>
#[derive(Debug, Clone)]
pub struct Db {
    pub blockchain: Arc<RwLock<Block>>,
    pub pending_transactions: Arc<RwLock<HashMap<String, Transaction>>>,
    pub users: Arc<RwLock<HashMap<String, User>>>,
}

impl Db {
    fn new() -> Self {
        Db {
            blockchain: Arc::new(RwLock::new(Block::new())),
            pending_transactions: Arc::new(RwLock::new(HashMap::new())),
            users: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

/// A transaction between `source` and `target` that moves `amount`
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Transaction {
    pub by: PublicKeySignature,
    pub source: PublicKeySignature,
    pub target: PublicKeySignature,
    pub amount: i32,
    pub timestamp: NaiveDateTime,
}

/// A block that was proposed with `transaction_list` and `nonce` that made `hash` valid, 6 zeroes
/// at the right hand side of the hash (24 bytes)
///
/// We are mining using blake2s algorithm, which produces 256 bit hashes. Hash/second is roughly
/// 20x10^3.
///
/// https://serde.rs/container-attrs.html might be valuable to normalize the serialize/deserialize
/// conventions as these will be hashed
///
#[derive(Serialize, Deserialize, Debug)]
pub struct Block {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub transaction_list: Vec<PublicKeySignature>,
    pub nonce: u32,
    pub timestamp: NaiveDateTime,
    pub hash: String,
}

/// For prototyping and letting serde handle everything json
#[derive(Serialize, Deserialize, Debug)]
pub struct NakedBlock {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub transaction_list: Vec<PublicKeySignature>,
    pub nonce: u32,
    pub timestamp: NaiveDateTime,
}

impl Block {
    /// Genesis block
    pub fn new() -> Block {
        Block {
            transaction_list: vec!["gradecoin_bank".to_owned()],
            nonce: 0,
            timestamp: NaiveDate::from_ymd(2021, 04, 11).and_hms(20, 45, 00),
            hash: String::from("not_actually_mined"),
        }
    }
}

/// Simply a Student
#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub user_id: MetuId,
    pub public_key: String,
    pub balance: i32,
}

/// The values will be hard coded so MetuId::new() can accept/reject values based on that
#[derive(Serialize, Deserialize, Debug)]
pub struct MetuId {
    id: String,
}

// TODO: this will arrive encrypted <13-04-21, yigit> //
#[derive(Serialize, Deserialize, Debug)]
pub struct AuthRequest {
    pub student_id: String,
    pub public_key: String,
}

lazy_static! {
    static ref OUR_STUDENTS: HashSet<&'static str> = {
        [
            "e254275", "e223687", "e211024", "e209888", "e223725", "e209362", "e209898", "e230995",
            "e223743", "e223747", "e223749", "e223751", "e188126", "e209913", "e203608", "e233013",
            "e216982", "e217185", "e223780", "e194931", "e223783", "e254550", "e217203", "e217477",
            "e223786", "e231060", "e223795",
        ]
        .iter()
        .cloned()
        .collect()
    };
}

impl fmt::Display for MetuId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl MetuId {
    pub fn new(id: String) -> Option<Self> {
        if OUR_STUDENTS.contains(&*id) {
            Some(MetuId { id: id })
        } else {
            None
        }
    }
}

// TODO: write schema tests using the original repo <09-04-21, yigit> //
