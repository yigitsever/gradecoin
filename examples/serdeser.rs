use gradecoin::schema::Transaction;
use serde_json;

pub fn main() {

    let tx = Transaction {
        source: "fingerprint_of_some_guy".to_owned(),
        target: "31415926535897932384626433832795028841971693993751058209749445923".to_owned(),
        amount: 2,
        timestamp: chrono::NaiveDate::from_ymd(2021, 04, 13).and_hms(20, 55, 30),
    };

    let tx_string = serde_json::to_string(&tx).unwrap();

    println!("{}", &tx_string);
}
