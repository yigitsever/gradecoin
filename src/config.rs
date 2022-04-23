//! # Configuration
//!
//! This module holds the data structures for network configuration.
use serde::{Deserialize, Serialize};
use log::{error, info};

/// Configuration for a single network
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Config {
    // Valid blocks should have this many transactions
    pub block_transaction_count: u8,
    // Inital registration bonus
    pub register_bonus: u16,
    // Coinbase reward
    pub block_reward: u16,
    // Transaction amount limit
    pub tx_upper_limit: u16,
    pub tx_lower_limit: u16,
    // Transaction traffic reward
    pub tx_traffic_reward: u16,
}

impl Config {
    pub fn read(filename: &str) -> Option<Self> {
        let file = match std::fs::File::open(filename) {
            Ok(f) => f,
            Err(e) => {
                error!("Cannot read config file: {}", filename);
                error!("Error: {:?}", e);
                return None;
            },
        };
        let config : Config = match serde_yaml::from_reader(file) {
            Ok(c) => c,
            Err(e) => {
                error!("Cannot parse config file: {}", filename);
                error!("Error: {:?}", e);
                return None;
            },
        };
        // File closes automatically when it goes out of scope.
        info!("Config file read successfully: {}", filename);
        Some(config)
    }
}
