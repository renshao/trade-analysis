use std::error::Error;
use std::io;
use std::process;
use std::fs::File;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
enum TradeType {
    BUY,
    SELL,
}

#[derive(Debug, Deserialize)]
struct TradingRecord {
    date: String,
    #[serde(alias = "buy or sell")]
    buy_or_sell: TradeType,
    code: String,
    volume: i32,
    price: f32,
    fee: f32
}

fn read_transactions() -> Result<(), Box<dyn Error>> {
    // Build the CSV reader and iterate over each record.
    let mut rdr = csv::Reader::from_reader(io::BufReader::new(File::open("trades.csv")?));
    for result in rdr.deserialize() {
        // The iterator yields Result<StringRecord, Error>, so we check the
        // error here..
        let record: TradingRecord = result?;
        println!("{:?}", record);
    }
    Ok(())
}

fn main() {
    if let Err(err) = read_transactions() {
        println!("error running example: {}", err);
        process::exit(1);
    }
}