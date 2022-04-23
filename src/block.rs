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
use serde::{Deserialize, Serialize};

pub type Fingerprint = String;
pub type Id = String;

/// A block that was proposed with `transaction_list` and `nonce`
/// that made `hash` valid, 6 zeroes at the left hand side of the hash (24 bytes)
///
/// We are mining using blake2s algorithm, which produces 256 bit hashes.
/// Hash/second is roughly 20x10^3.
///
/// <https://serde.rs/container-attrs.html> might be valuable to normalize the
/// serialize/deserialize conventions as these will be hashed
///
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Block {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub transaction_list: Vec<Fingerprint>,
    pub nonce: u32,
    pub timestamp: NaiveDateTime,
    pub hash: String,
}

impl Default for Block {
    fn default() -> Self {
        Block {
            transaction_list: vec!["gradecoin_bank".to_owned()],
            nonce: 0,
            timestamp: NaiveDate::from_ymd(2022, 4, 11).and_hms(20, 45, 00),
            hash: String::from("not_actually_mined"),
        }
    }
}

/// For prototyping and letting serde handle everything json
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[allow(clippy::module_name_repetitions)]
pub struct NakedBlock {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub transaction_list: Vec<Fingerprint>,
    pub nonce: u32,
    pub timestamp: NaiveDateTime,
}

/// A transaction between `source` and `target` that moves `amount`
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Transaction {
    pub source: Fingerprint,
    pub target: Fingerprint,
    pub amount: u16,
    pub timestamp: NaiveDateTime,
}

/// A JWT Payload/Claims representation
///
/// <https://tools.ietf.org/html/rfc7519#section-4.1>
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
