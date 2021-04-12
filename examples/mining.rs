use chrono::NaiveDate;
use gradecoin::schema::NakedBlock;
use serde_json;
use std::time::Instant;

use blake2::{Blake2s, Digest};

pub fn main() {
    let mut b = NakedBlock {
        transaction_list: vec![
            "hash_value".to_owned(),
        ],
        nonce: 0,
        timestamp: NaiveDate::from_ymd(2021, 04, 08).and_hms(12, 30, 30),
    };

    let now = Instant::now();

    for nonce in 0..u32::MAX {
        b.nonce = nonce;

        let j = serde_json::to_vec(&b).unwrap();

        let result = Blake2s::digest(&j);

        let first_five = result[31] as i32 + result[30] as i32 + (result[29] << 4) as i32;

        if first_five == 0 {
            println!("{} - {:x}\n{:?}", nonce, result, b);
            break;
        }
    }

    println!("it took {} seconds", now.elapsed().as_secs());
}
