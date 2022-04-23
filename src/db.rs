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
use crate::config::{BotConfig, Config};
use crate::student::{MetuId, User, UserAtRest};
use log::info;
use parking_lot::RwLock;
use std::{collections::HashMap, fs, io, path::PathBuf, sync::Arc};

#[derive(Debug, Clone, Default)]
pub struct Db {
    pub blockchain: Arc<RwLock<Block>>,
    pub pending_transactions: Arc<RwLock<HashMap<Id, Transaction>>>,
    pub users: Arc<RwLock<HashMap<Fingerprint, User>>>,
    pub config: Config,
    preapproved_users: Vec<MetuId>,
}

impl Db {
    pub fn new(config: Config) -> Self {
        fs::create_dir_all(format!("blocks/{}", config.name)).unwrap();
        fs::create_dir_all(format!("users/{}", config.name)).unwrap();

        // Load bots
        let users: HashMap<Fingerprint, User> = get_bots(&config.bots);

        // Load the list of users who can register
        let preapproved_users = read_approved_users(&config.preapproved_users);

        let mut db = Db {
            blockchain: Arc::new(RwLock::new(Block::default())),
            pending_transactions: Arc::new(RwLock::new(HashMap::new())),
            users: Arc::new(RwLock::new(users)),
            config,
            preapproved_users,
        };

        // Load the latest block, continue from where we left off
        if let Some(block_path) = last_block_content(&db.config.name) {
            db.populate_with_last_block(block_path);
        }

        // Load the users that had registered themselves
        if let Ok(users_path) = read_users(&db.config.name) {
            db.populate_with_users(users_path);
        }

        db
    }

    fn populate_with_last_block(&mut self, path: String) {
        info!("Populating db with the latest block {}", path);
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

                info!("Populating db with user: {:?}", user_at_rest);
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

fn last_block_content(config_name: &str) -> Option<String> {
    let blocks = read_block_name(config_name).unwrap();

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

fn read_block_name(config_name: &str) -> io::Result<Vec<PathBuf>> {
    let path = format!("./blocks/{}", config_name);
    let entries = fs::read_dir(path)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;

    Ok(entries)
}

fn parse_block(path: &str) -> u64 {
    let start_pos = path.rfind('/').unwrap() + 1;
    let end_pos = path.find(".block").unwrap();
    let block_str = path[start_pos..end_pos].to_string();
    let block_u64: u64 = block_str.parse().unwrap();
    block_u64
}

fn read_users(config_name: &str) -> io::Result<Vec<PathBuf>> {
    let path = format!("./users/{}", config_name);
    let entries = fs::read_dir(path)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;

    Ok(entries)
}

/// Build bots from the given set of bot configurations.
fn get_bots(bot_configs: &HashMap<Fingerprint, BotConfig>) -> HashMap<Fingerprint, User> {
    let mut index = 0;

    bot_configs
        .iter()
        .map(|(fingerprint, config)| {
            index += 1;
            (
                fingerprint.to_string(),
                User {
                    user_id: MetuId::new(format!("friend_{}", index), "not_used".to_owned()),
                    public_key: "not_used".to_owned(),
                    balance: config.starting_balance,
                    is_bot: true,
                },
            )
        })
        .collect()
}

fn read_approved_users(filename: &str) -> Vec<MetuId> {
    let mut approved_students: Vec<MetuId> = Vec::new();
    let contents = fs::read_to_string(filename).unwrap_or_else(|_| {
        panic!(
            "{}",
            format!(
                "Expected {} in place to load preapproved students",
                filename
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
