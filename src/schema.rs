use chrono::NaiveDateTime;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

// use crate::validators;

// In memory data structure
// Two approaches here
// 1. Db is a type pub type Db = Arc<RwLock<Vec<Ledger>>>; Ledger is a struct, we wrap the ledger
//    with arc + mutex in ledger() to access transactions we need to unwrap blocks as well, vice
//    versa
//
// 2. Db is a struct attributes are wrapped we can offload ::new() to it's member method blocks and
//    transactions are accessible separately, which is the biggest pro
//
// 3. use an actual database (for blockchain and users this makes the most sense tbh but pending
//    transactions are perfectly fine in memory)

/// Creates a new database
pub fn create_database() -> Db {
    Db::new()
}

#[derive(Debug, Clone)]
pub struct Db {
    // heh. also https://doc.rust-lang.org/std/collections/struct.LinkedList.html says Vec is generally faster
    pub blockchain: Arc<RwLock<Vec<Block>>>,
    // every proposer can have _one_ pending transaction, a way to enforce this, String is proposer identifier
    pub pending_transactions: Arc<RwLock<HashMap<String, Transaction>>>,
}

impl Db {
    fn new() -> Self {
        Db {
            blockchain: Arc::new(RwLock::new(Vec::new())),
            pending_transactions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

/// A transaction between `source` and `target` that moves `amount`
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Transaction {
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
    pub nonce: String,
    pub timestamp: NaiveDateTime,
    pub hash: String, // future proof'd baby
}


// TODO: write schema tests using the original repo <09-04-21, yigit> //
