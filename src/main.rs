#[macro_use] extern crate prettytable;
use std::error::Error;
use std::{io, fmt};
use std::process;
use std::fs::File;
use serde::Deserialize;
use prettytable::{Table, Row, Cell};
use prettytable::format;
use prettytable::format::Alignment;


#[derive(Debug, Deserialize)]
enum TradeType {
    BUY,
    SELL,
}
impl fmt::Display for TradeType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TradeType::BUY => write!(f, "BUY"),
            TradeType::SELL => write!(f, "SELL"),
        }
    }
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

fn calculateTotal(price: f32, volume: i32, fee: f32) -> f32 {
    price * (volume as f32) + fee
}

fn read_transactions() -> Result<(), Box<dyn Error>> {
    let mut table = Table::new();
    table.set_titles(row!["Date", "Trade", "Code", "Volume", "Price", "Fee", "Total"]);

    let mut rdr = csv::Reader::from_reader(io::BufReader::new(File::open("trades.csv")?));
    for result in rdr.deserialize() {
        let record: TradingRecord = result?;
        let mut row = row![record.date, record.buy_or_sell, record.code];
        row.add_cell(Cell::new_align(&record.volume.to_string(), Alignment::RIGHT));
        row.add_cell(Cell::new_align(&format!("{:.3}", &record.price), Alignment::RIGHT));
        row.add_cell(Cell::new_align(&format!("{:.2}", record.fee), Alignment::RIGHT));
        let mut totalString = format!("{:.2}", calculateTotal(record.price, record.volume, record.fee));
        match record.buy_or_sell {
            TradeType::BUY => totalString.insert_str(0, "- "),
            _ => (),
        }
        row.add_cell(Cell::new_align(&totalString, Alignment::RIGHT));
        table.add_row(row);
    }

    table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
    table.printstd();
    Ok(())
}

fn main() {
    if let Err(err) = read_transactions() {
        println!("error running example: {}", err);
        process::exit(1);
    }
}