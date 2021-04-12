use chrono::{NaiveDate, NaiveDateTime};
use lazy_static::lazy_static;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fs;
use std::sync::Arc;

// use crate::validators;

/// We need persistence for blocks and users, not so much for transactions
/// There are around 30 students, a full fledged database would be an overkill (for next year?)
/// Pending transactions are held in memory, these are cleared with every new block
/// Only the last block is held in memory, every block is written to a file
/// Users are held in memory and they're also backed up to text files

/// Creates a new database connection
pub fn create_database() -> Db {
    fs::create_dir_all("blocks").unwrap();
    fs::create_dir_all("users").unwrap();
    Db::new()
}

#[derive(Debug, Clone)]
pub struct Db {
    // heh. also https://doc.rust-lang.org/std/collections/struct.LinkedList.html says Vec is generally faster
    pub blockchain: Arc<RwLock<Block>>,
    // every proposer can have _one_ pending transaction, a way to enforce this, String is proposer identifier
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
    // TODO: new field by <11-04-21, yigit> //
    pub source: String,
    pub target: String,
    pub amount: i32,
    pub timestamp: NaiveDateTime,
}

/// A block that was proposed with `transaction_list` and `nonce` that made `hash` valid
/// https://serde.rs/container-attrs.html might be valueable to normalize the serialize/deserialize
/// conventions as these will be hashed
#[derive(Serialize, Deserialize, Debug)]
pub struct Block {
    // TODO: transaction list should hold transaction hash values <09-04-21, yigit> //
    // but do we link them somehow? (like a log of old transactions?)
    // we can leave this as is and whenever we have a new block we _could_ just log it to file
    // somewhere
    // I want to keep this as a String vector because it makes things easier elsewhere
    pub transaction_list: Vec<String>, // hashes of the transactions (or just "source" for now)
    pub nonce: u32,
    pub timestamp: NaiveDateTime,
    pub hash: String, // future proof'd baby
}

/// For prototyping and letting serde handle everything json
#[derive(Serialize, Deserialize, Debug)]
pub struct NakedBlock {
    pub transaction_list: Vec<String>,
    pub nonce: u32,
    pub timestamp: NaiveDateTime,
}

impl Block {
    /// Genesis block
    pub fn new() -> Block {
        Block {
            transaction_list: vec![],
            nonce: 0,
            timestamp: NaiveDate::from_ymd(2021, 04, 11).and_hms(20, 45, 00),
            hash: String::from(""),
        }
    }
}

/// Or simply a Student
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
