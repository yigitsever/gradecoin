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
use std::io;
use std::vec::Vec;
use std::string::String;
use std::path::PathBuf;
// use crate::validators;

pub type PublicKeySignature = String;


fn last_block_exists() -> (bool, String) {
    let blocks = read_block_name().unwrap();
    for block in blocks {
        let block = block.to_str().unwrap();
        if block.contains("last.block") {
            return (true, block.to_string());
        }
    }
    (false, "".to_string())
}

fn read_block_name() -> io::Result<Vec<PathBuf>> {
    let mut entries = fs::read_dir("./blocks")?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;

    // The order in which `read_dir` returns entries is not guaranteed. If reproducible
    // ordering is required the entries should be explicitly sorted.

    entries.sort();

    // The entries have now been sorted by their path.

    Ok(entries)
}

fn create_db_with_last_block(path: String) -> Db {
    let file = fs::read(path).unwrap();
    let json = std::str::from_utf8(&file).unwrap();
    let block: Block = serde_json::from_str(json).unwrap();
    let db = Db::new();
    *db.blockchain.write() = block;
    return db;
}
/// Creates a new database
pub fn create_database() -> Db {
    fs::create_dir_all("blocks").unwrap();
    fs::create_dir_all("users").unwrap();
    let (res, path) = last_block_exists();
    if res {
        return create_db_with_last_block(path);
    } else {
        return Db::new();
    }
}

/// A JWT Payload/Claims representation
///
/// https://tools.ietf.org/html/rfc7519#section-4.1
///
/// - `tha`: Transaction Hash, String (custom field)
/// - `iat`: Issued At, Unix Time, epoch
/// - `exp`: Expiration Time, epoch
#[derive(Debug, Serialize, Deserialize, PartialEq)]
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
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Block {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub transaction_list: Vec<PublicKeySignature>,
    pub nonce: u32,
    pub timestamp: NaiveDateTime,
    pub hash: String,
}

/// For prototyping and letting serde handle everything json
#[derive(Serialize, Deserialize, Debug, PartialEq)]
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
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct User {
    pub user_id: MetuId,
    pub public_key: String,
    pub balance: i32,
}

/// The values will be hard coded so MetuId::new() can accept/reject values based on that
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct MetuId {
    id: String,
    passwd: String,
}

/// The plaintext of the initial user authentication request
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct AuthRequest {
    pub student_id: String,
    pub passwd: String,
    pub public_key: String,
}

/// Ciphertext of the initial authentication request, or what we will receive
#[derive(Serialize, Deserialize, Debug)]
pub struct InitialAuthRequest {
    pub c: String,
    pub iv: [u8; 32],
    pub key: String,
}

lazy_static! {
    static ref OUR_STUDENTS: HashSet<(&'static str, &'static str)> = {
        [
            ("e254275", "DtNX1qk4YF4saRH"),
            ("e223687", "cvFEs4XLjuGBD1v"),
            ("e211024", "voQAcxiKJmEXYRT"),
            ("e209888", "O75dli6AQtz2tUi"),
            ("e223725", "xXuTD3Y4tyrv2Jz"),
            ("e209362", "N7wGm5XU5zVWOWu"),
            ("e209898", "aKBFfB8fZMq8pVn"),
            ("e230995", "TgcHGlqeFhQGx42"),
            ("e223743", "YVWVSWuIHplJk9C"),
            ("e223747", "8LAeHrsjnwXh59Q"),
            ("e223749", "HMFeJqVOzwCPHbc"),
            ("e223751", "NjMsxmtmy2VOwMW"),
            ("e188126", "QibuPdV2gXfsVJW"),
            ("e209913", "kMxJvl2vHSWCy4A"),
            ("e203608", "mfkkR0MWurk6Rp1"),
            ("e233013", "GCqHxdOaDj2pWXx"),
            ("e216982", "2Z0xmgCStnj5qg5"),
            ("e217185", "BcaZNlzlhPph7A3"),
            ("e223780", "2KvVxKUQaA9H4sn"),
            ("e194931", "hsC0Wb8PQ5vzwdQ"),
            ("e223783", "ETUJA3kt1QYvJai"),
            ("e254550", "rPRjX0A4NefvKWi"),
            ("e217203", "lN3IWhGyCrGfkk5"),
            ("e217477", "O9xlMaa7LanC82w"),
            ("e223786", "UxI6czykJfp9T9N"),
            ("e231060", "VJgziofQQPCoisH"),
            ("e223795", "pmcTCKox99NFsqp"),
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
    pub fn new(id: String, pwd: String) -> Option<Self> {
        if OUR_STUDENTS.contains(&(&*id, &*pwd)) {
            Some(MetuId { id: id, passwd: pwd })
        } else {
            None
        }
    }
}
