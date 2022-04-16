//! # Global Database representation
//!
//! [`Db::blockchain`] is just the last block that was mined.
//! All the blocks are written to disk as text files whenever they are accepted.
//!
//! [`Db::pending_transactions`] is the in memory representation of the waiting transactions.
//! Every user can have only one outstanding transaction at any given time.
//!
//! [`Db::users`] is the in memory representation of the users,
//! with their public keys, `metu_ids` and gradecoin balances.
use crate::block::{Block, Fingerprint, Id, Transaction};
use crate::student::{MetuId, User, UserAtRest};
use log::debug;
use parking_lot::RwLock;
use std::{collections::HashMap, fs, io, path::PathBuf, sync::Arc};

const PREAPPROVED_STU_FILENAME: &str = "students.csv";

#[derive(Debug, Clone, Default)]
pub struct Db {
    pub blockchain: Arc<RwLock<Block>>,
    pub pending_transactions: Arc<RwLock<HashMap<Id, Transaction>>>,
    pub users: Arc<RwLock<HashMap<Fingerprint, User>>>,
    preapproved_users: Vec<MetuId>,
}

impl Db {
    pub fn new() -> Self {
        fs::create_dir_all("blocks").unwrap();
        fs::create_dir_all("users").unwrap();
        let mut db = Db::default();
        if let Some(block_path) = last_block_content() {
            db.populate_with_last_block(block_path);
        }

        if let Ok(users_path) = read_users() {
            db.populate_with_users(users_path);
        }

        let users: HashMap<Fingerprint, User> = get_friendly_users();
        let preapproved_users = read_approved_users();

        Db {
            blockchain: Arc::new(RwLock::new(Block::default())),
            pending_transactions: Arc::new(RwLock::new(HashMap::new())),
            users: Arc::new(RwLock::new(users)),
            preapproved_users,
        }
    }

    fn populate_with_last_block(&mut self, path: String) {
        debug!("Populating db with the latest block {}", path);
        let file = fs::read(path).unwrap();
        let json = std::str::from_utf8(&file).unwrap();
        let block: Block = serde_json::from_str(json).unwrap();
        *self.blockchain.write() = block;
    }

    fn populate_with_users(&mut self, files: Vec<PathBuf>) {
        for fs in files {
            if let Ok(file_content) = fs::read(fs) {
                let json =
                    String::from_utf8(file_content).expect("we have written a malformed user file");
                let user_at_rest: UserAtRest = serde_json::from_str(&json).unwrap();

                debug!("Populating db with user: {:?}", user_at_rest);
                self.users
                    .write()
                    .insert(user_at_rest.fingerprint, user_at_rest.user);
            }
        }
    }

    pub fn is_user_preapproved(&self, id: &Id, passwd: &String) -> bool {
        for user in &self.preapproved_users {
            if *user.get_id() == *id && *user.get_passwd() == *passwd {
                return true;
            }
        }

        false
    }
}

fn last_block_content() -> Option<String> {
    let blocks = read_block_name().unwrap();

    if blocks.is_empty() {
        return None;
    }

    let last_block = blocks[0].to_str().unwrap();
    let mut last_block = parse_block(last_block);
    let mut last_block_index = 0;

    for (index, block) in blocks.iter().enumerate() {
        let block = block.to_str().unwrap();
        let block = parse_block(block);
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

fn parse_block(path: &str) -> u64 {
    let end_pos = path.find(".block").unwrap();
    let block_str = path[9..end_pos].to_string();
    let block_u64: u64 = block_str.parse().unwrap();
    block_u64
}

fn read_users() -> io::Result<Vec<PathBuf>> {
    let entries = fs::read_dir("./users")?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;

    Ok(entries)
}

fn get_friendly_users() -> HashMap<Fingerprint, User> {
    let mut users: HashMap<Fingerprint, User> = HashMap::new();

    users.insert(
        "cde48537ca2c28084ff560826d0e6388b7c57a51497a6cb56f397289e52ff41b".to_owned(),
        User {
            user_id: MetuId::new("friend_1".to_owned(), "not_used".to_owned()),
            public_key: "not_used".to_owned(),
            balance: 70,
            is_bot: true,
        },
    );

    users.insert(
        "a1a38b5bae5866d7d998a9834229ec2f9db7a4fc8fb6f58b1115a96a446875ff".to_owned(),
        User {
            user_id: MetuId::new("friend_2".to_owned(), "not_used".to_owned()),
            public_key: "not_used".to_owned(),
            balance: 20,
            is_bot: true,
        },
    );

    users.insert(
        "4e048fd2a62f1307866086e803e9be43f78a702d5df10831fbf434e7663ae0e7".to_owned(),
        User {
            user_id: MetuId::new("friend_4".to_owned(), "not_used".to_owned()),
            public_key: "not_used".to_owned(),
            balance: 120,
            is_bot: true,
        },
    );

    users.insert(
        "60e77101e76950a9b1830fa107fd2f8fc545255b3e0f14b6a7797cf9ee005f07".to_owned(),
        User {
            user_id: MetuId::new("friend_4".to_owned(), "not_used".to_owned()),
            public_key: "not_used".to_owned(),
            balance: 40,
            is_bot: true,
        },
    );
    users
}

fn read_approved_users() -> Vec<MetuId> {
    let mut approved_students: Vec<MetuId> = Vec::new();
    let contents = fs::read_to_string(PREAPPROVED_STU_FILENAME).unwrap_or_else(|_| {
        panic!(
            "{}",
            format!(
                "Expected {} in place to load preapproved students",
                PREAPPROVED_STU_FILENAME
            )
        )
    });
    let mut reader = csv::Reader::from_reader(contents.as_bytes());
    for student in reader.records() {
        let student = student.unwrap();
        approved_students.push(MetuId::new(student[0].to_owned(), student[1].to_owned()));
    }
    approved_students
}
