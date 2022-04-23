//! # Configuration
//!
//! This module holds the data structures for network configuration.
use crate::block::Fingerprint;
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration struct for a single bot
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct BotConfig {
    /// The initial balance of this bot.
    pub starting_balance: u16,
}

/// Configuration for a single network
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Config {
    /// Name of the network
    pub name: String,

    /// URL prefix for this network, can be empty
    ///
    /// For example, if url_prefix is `example`, register at
    /// `gradecoin.xyz/example/register`
    pub url_prefix: String,

    /// CSV file that contains the list of users who can register
    ///
    /// Format of CSV file:
    /// ```
    /// User ID, Password
    /// e123456,register_password
    /// e123456,register_password
    /// ```
    /// First line is ignored.
    pub preapproved_users: String,

    /// Valid blocks should have this many transactions
    pub block_transaction_count: u8,

    /// How many zero hexadecimal characters should a correct hash start with?
    pub hash_zeros: u8,

    /// Inital registration bonus
    pub register_bonus: u16,

    /// Coinbase reward
    pub block_reward: u16,

    /// Transaction amount upper limit
    pub tx_upper_limit: u16,

    /// Transaction amount lower limit
    pub tx_lower_limit: u16,

    /// Transaction traffic reward
    pub tx_traffic_reward: u16,

    /// The configuration of the bots in this network.
    /// Maps bot fingerprints to their configurations.
    pub bots: HashMap<Fingerprint, BotConfig>,
}

impl Config {
    /// Read the configuration from a given `.yaml` file.
    pub fn read(filename: &str) -> Option<Self> {
        let file = match std::fs::File::open(filename) {
            Ok(f) => f,
            Err(e) => {
                error!("Cannot read config file: {}", filename);
                error!("Error: {:?}", e);
                return None;
            }
        };
        let config: Config = match serde_yaml::from_reader(file) {
            Ok(c) => c,
            Err(e) => {
                error!("Cannot parse config file: {}", filename);
                error!("Error: {:?}", e);
                return None;
            }
        };
        // File closes automatically when it goes out of scope.
        info!("Config file read successfully: {}", filename);
        Some(config)
    }
}
