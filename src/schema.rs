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
use log::debug;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::string::String;
use std::sync::Arc;
use std::vec::Vec;

pub type Fingerprint = String;
pub type Id = String;

fn block_parser(path: String) -> u64 {
    let end_pos = path.find(".block").unwrap();
    let block_str = path[9..end_pos].to_string();
    let block_u64: u64 = block_str.parse().unwrap();
    block_u64
}

fn last_block_content() -> Option<String> {
    let blocks = read_block_name().unwrap();

    if blocks.is_empty() {
        return None;
    }

    let last_block = blocks[0].to_str().unwrap();
    let mut last_block = block_parser(last_block.to_string());
    let mut last_block_index = 0;

    for (index, block) in blocks.iter().enumerate() {
        let block = block.to_str().unwrap();
        let block = block_parser(block.to_string());
        if block > last_block {
            last_block = block;
            last_block_index = index;
        }
    }
    return Some(blocks[last_block_index].to_str().unwrap().parse().unwrap());
}

fn read_block_name() -> io::Result<Vec<PathBuf>> {
    let entries = fs::read_dir("./blocks")?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;

    Ok(entries)
}

fn read_users() -> io::Result<Vec<PathBuf>> {
    let entries = fs::read_dir("./users")?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;

    Ok(entries)
}

fn populate_db_with_last_block(db: &mut Db, path: String) -> &mut Db {
    debug!("Populating db with last block {}", path);
    let file = fs::read(path).unwrap();
    let json = std::str::from_utf8(&file).unwrap();
    let block: Block = serde_json::from_str(json).unwrap();
    *db.blockchain.write() = block;

    db
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct UserAtRest {
    pub fingerprint: Fingerprint,
    pub user: User,
}

fn populate_db_with_users(db: &mut Db, files: Vec<PathBuf>) -> &mut Db {
    for fs in files {
        if let Ok(file_content) = fs::read(fs) {
            let json =
                String::from_utf8(file_content).expect("we have written a malformed user file");
            let user_at_rest: UserAtRest = serde_json::from_str(&json).unwrap();

            debug!("Populating db with user: {:?}", user_at_rest);
            db.users
                .write()
                .insert(user_at_rest.fingerprint, user_at_rest.user);
        }
    }

    db
}

/// Creates a new database, uses the previous last block if one exists and attempts the populate
/// the users
pub fn create_database() -> Db {
    fs::create_dir_all("blocks").unwrap();
    fs::create_dir_all("users").unwrap();
    let mut db = Db::new();
    if let Some(block_path) = last_block_content() {
        populate_db_with_last_block(&mut db, block_path);
    }

    if let Ok(users_path) = read_users() {
        populate_db_with_users(&mut db, users_path);
    }

    db
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
/// drocoin balances.
#[derive(Debug, Clone)]
pub struct Db {
    pub blockchain: Arc<RwLock<Block>>,
    pub pending_transactions: Arc<RwLock<HashMap<Id, Transaction>>>,
    pub users: Arc<RwLock<HashMap<Fingerprint, User>>>,
}

impl Db {
    pub fn new() -> Self {
        let mut users: HashMap<Fingerprint, User> = HashMap::new();

        let friendly_1 = MetuId::new("friend_1".to_owned(), "not_used".to_owned()).unwrap();

        users.insert(
            "cde48537ca2c28084ff560826d0e6388b7c57a51497a6cb56f397289e52ff41b".to_owned(),
            User {
                user_id: friendly_1,
                public_key: "not_used".to_owned(),
                balance: 70,
                is_bot: true,
            },
        );

        let friendly_2 = MetuId::new("friend_2".to_owned(), "not_used".to_owned()).unwrap();

        users.insert(
            "a1a38b5bae5866d7d998a9834229ec2f9db7a4fc8fb6f58b1115a96a446875ff".to_owned(),
            User {
                user_id: friendly_2,
                public_key: "not_used".to_owned(),
                balance: 20,
                is_bot: true,
            },
        );

        let friendly_3 = MetuId::new("friend_4".to_owned(), "not_used".to_owned()).unwrap();

        users.insert(
            "4e048fd2a62f1307866086e803e9be43f78a702d5df10831fbf434e7663ae0e7".to_owned(),
            User {
                user_id: friendly_3,
                public_key: "not_used".to_owned(),
                balance: 120,
                is_bot: true,
            },
        );

        let friendly_4 = MetuId::new("friend_4".to_owned(), "not_used".to_owned()).unwrap();

        users.insert(
            "60e77101e76950a9b1830fa107fd2f8fc545255b3e0f14b6a7797cf9ee005f07".to_owned(),
            User {
                user_id: friendly_4,
                public_key: "not_used".to_owned(),
                balance: 40,
                is_bot: true,
            },
        );

        Db {
            blockchain: Arc::new(RwLock::new(Block::new())),
            pending_transactions: Arc::new(RwLock::new(HashMap::new())),
            users: Arc::new(RwLock::new(users)),
        }
    }
}

impl Default for Db {
    fn default() -> Self {
        Self::new()
    }
}

/// A transaction between `source` and `target` that moves `amount`
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Transaction {
    pub source: Fingerprint,
    pub target: Fingerprint,
    pub amount: u16,
    pub timestamp: NaiveDateTime,
}

/// A block that was proposed with `transaction_list` and `nonce` that made `hash` valid, 6 zeroes
/// at the left hand side of the hash (24 bytes)
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
    pub transaction_list: Vec<Fingerprint>,
    pub nonce: u32,
    pub timestamp: NaiveDateTime,
    pub hash: String,
}

/// For prototyping and letting serde handle everything json
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct NakedBlock {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub transaction_list: Vec<Fingerprint>,
    pub nonce: u32,
    pub timestamp: NaiveDateTime,
}

impl Block {
    /// Genesis block
    pub fn new() -> Block {
        Block {
            transaction_list: vec!["drocoin_bank".to_owned()],
            nonce: 0,
            timestamp: NaiveDate::from_ymd(2021, 4, 11).and_hms(20, 45, 00),
            hash: String::from("not_actually_mined"),
        }
    }
}

impl Default for Block {
    fn default() -> Self {
        Self::new()
    }
}

/// A Student
///
/// * [`user_id`]: Can only be one of the repopulated
/// * [`public_key`]: A PEM format public key "---- BEGIN" and all
/// * [`balance`]: User's current Drocoin amount
///
/// This should ideally include the fingerprint as well?
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct User {
    pub user_id: MetuId,
    pub public_key: String,
    pub balance: u16,
    #[serde(skip, default = "bool::default")]
    pub is_bot: bool,
}

/// The values are hard coded in [`OUR_STUDENTS`] so MetuId::new() can accept/reject values based on that
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct MetuId {
    id: String,
    passwd: String,
}

impl MetuId {
    pub fn quick_equal(&self, other: &str) -> bool {
        self.id == other
    }
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
    pub iv: String,
    pub key: String,
}

// Students who are authorized to have Drocoin accounts
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
            ("e223715", "1H5QuOYI1b2r9ET"),
            ("e181932", "THANKYOUHAVEFUN"),
            ("bank", "P7oxDm30g1jeIId"),
            ("friend_1", "not_used"),
            ("friend_2", "not_used"),
            ("friend_3", "not_used"),
            ("friend_4", "not_used"),
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
            Some(MetuId { id, passwd: pwd })
        } else {
            None
        }
    }
}
