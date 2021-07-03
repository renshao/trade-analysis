mod inventory;

#[macro_use] extern crate prettytable;
use std::error::Error;
use std::{io, fmt};
use std::process;
use std::fs::File;
use serde::Deserialize;
use prettytable::{Table, Row, Cell};
use prettytable::format;
use prettytable::format::Alignment;
use crate::inventory::Inventory;
use std::panic::resume_unwind;
use prettytable::{Attr, color};
use std::sync::atomic::Ordering::AcqRel;
use chrono::{NaiveDateTime, Datelike};


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
    volume: u32,
    price: f32,
    fee: f32
}

fn calculateTotal(price: f32, volume: u32, fee: f32) -> f32 {
    price * (volume as f32) + fee
}

fn read_transactions() -> Result<(), Box<dyn Error>> {
    let mut inventory = Inventory::new();

    let mut table = Table::new();
    table.set_titles(row!["Date", "Trade", "Code", "Volume", "Price", "Fee", "Total"]);

    let mut rdr = csv::Reader::from_reader(io::BufReader::new(File::open("trades.csv")?));
    for result in rdr.deserialize() {
        let record: TradingRecord = result?;

        let datetime = NaiveDateTime::parse_from_str(&record.date, "%Y-%m-%d %H:%M:%S").unwrap();
        let mut row = row![datetime.format("%Y-%m-%d"), record.buy_or_sell, record.code];
        row.add_cell(Cell::new_align(&record.volume.to_string(), Alignment::RIGHT));
        row.add_cell(Cell::new_align(&format!("{:.3}", &record.price), Alignment::RIGHT));
        row.add_cell(Cell::new_align(&format!("{:.2}", record.fee), Alignment::RIGHT));
        let mut totalString = format!("{:.2}", calculateTotal(record.price, record.volume, record.fee));

        match record.buy_or_sell {
            TradeType::BUY => {
                totalString.insert_str(0, "- ");
                inventory.buy(&record.code, record.volume, record.price, record.fee);
                row.add_cell(Cell::new_align(&totalString, Alignment::RIGHT));
                table.add_row(row);
            },
            TradeType::SELL => {
                let financial_year = if datetime.date().month() < 7 {datetime.date().year()} else {datetime.date().year() + 1};

                let fulfullment = inventory.sell(financial_year as u32, &record.code, record.volume, record.price, record.fee);
                let mut fulfillment_table = Table::new();
                fulfillment_table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);

                let mut i = 0;
                for (quantity, bought_price, bought_fee) in fulfullment.items {
                    if i == 0 {
                        let mut row = row![datetime.format("%Y-%m-%d"), record.buy_or_sell, record.code];
                        row.add_cell(Cell::new_align(&record.volume.to_string(), Alignment::RIGHT));
                        row.add_cell(Cell::new_align(&format!("{:.2}", &record.price), Alignment::RIGHT));
                        row.add_cell(Cell::new_align(&format!("{:.2}", record.fee), Alignment::RIGHT));
                        row.add_cell(Cell::new_align(&format!("{} x {:.2}", quantity, bought_price), Alignment::RIGHT));
                        table.add_row(row);
                    } else {
                        let mut row = row!["", "", "", "", "", ""];
                        row.add_cell(Cell::new_align(&format!("{} x {:.2}", quantity, bought_price), Alignment::RIGHT));
                        table.add_row(row);
                    }
                    i += 1;
                }
                let mut row = row!["", "", "", "", "", ""];
                let style = if fulfullment.net_profit < 0.0 { Attr::ForegroundColor(color::RED) } else { Attr::ForegroundColor(color::GREEN) };
                row.add_cell(Cell::new_align(&format!("net: {:.2}", fulfullment.net_profit), Alignment::RIGHT).with_style(style));
                table.add_row(row);
            }
        }


    }

    table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
    table.printstd();

    for (fy, profit) in inventory.fy_profit_map {
        println!("{}: {:.2}", fy, profit);
    }

    Ok(())
}

fn main() {
    if let Err(err) = read_transactions() {
        println!("error running example: {}", err);
        process::exit(1);
    }
}