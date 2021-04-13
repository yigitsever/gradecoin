use chrono::NaiveDate;
use gradecoin::schema::NakedBlock;
use serde_json;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

use blake2::{Blake2s, Digest};

const N: usize = 4;

pub fn main() {
    let counter = Arc::new(Mutex::new(0));

    let now = Instant::now();

    let mut threads = Vec::with_capacity(N);

    (0..N).for_each(|_| {
        let counter = Arc::clone(&counter);
        threads.push(thread::spawn(move || {
            let mut b = NakedBlock {
                transaction_list: vec!["e254275".to_owned()],
                nonce: 0,
                timestamp: NaiveDate::from_ymd(2021, 04, 13).and_hms(23, 38, 00),
            };

            let start: u32;
            let end: u32;
            {
                let mut num = counter.lock().unwrap();

                println!("Starting with 2 over {}", num);

                start = 0 + (1073741824 * *num);
                end = 1073741820 * (*num + 1);
                *num += 1;
            }

            println!("here {} - {}", start, end);

            for nonce in start..end {
                b.nonce = nonce;

                let j = serde_json::to_vec(&b).unwrap();

                let result = Blake2s::digest(&j);

                let first_six = result[0] as i32 + result[1] as i32 + (result[2]) as i32;

                if first_six == 0 {
                    println!("{} - {:x}\n{:?}", nonce, result, b);
                    break;
                }
            }
        }));
    });

    threads.into_iter().for_each(|thread| {
        thread
            .join()
            .expect("The thread creating or execution failed !")
    });

    println!("it took {} seconds", now.elapsed().as_secs());
}
