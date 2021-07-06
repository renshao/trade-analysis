mod inventory;

#[macro_use] extern crate prettytable;
use std::error::Error;
use std::{io, fmt};
use std::process;
use std::fs::File;
use serde::Deserialize;
use prettytable::{Table, Cell};
use prettytable::format;
use prettytable::format::Alignment;
use crate::inventory::Inventory;
use prettytable::{Attr, color};
use chrono::{NaiveDateTime, Datelike};
use Alignment::RIGHT;
use core::mem;

#[derive(Debug, Clone, Deserialize)]
pub enum TradeType {
    BUY,
    SELL,
    DIV
}
impl fmt::Display for TradeType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TradeType::BUY => write!(f, "BUY"),
            TradeType::SELL => write!(f, "SELL"),
            TradeType::DIV => write!(f, "DIV"),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct TradingRecord {
    date: NaiveDateTime,
    #[serde(alias = "buy or sell")]
    buy_or_sell: TradeType,
    code: String,
    volume: u32,
    price: f32,
    fee: f32
}

fn calculate_buy_total(price: f32, volume: u32, fee: f32) -> f32 {
    price * (volume as f32) + fee
}
fn calculate_sell_total(price: f32, volume: u32, fee: f32) -> f32 { price * (volume as f32) - fee }

fn read_transactions() -> Result<(), Box<dyn Error>> {
    let mut inventory = Inventory::new();

    let mut table = Table::new();
    table.set_titles(row!["Date", "Trade", "Code", "Volume", "Price", "Fee", "Total"]);

    let mut rdr = csv::Reader::from_reader(io::BufReader::new(File::open("trades.csv")?));
    for result in rdr.deserialize() {
        let record: TradingRecord = result?;
        let record2 = record.clone();
        inventory.record_transaction(&record2);

        //let datetime = NaiveDateTime::parse_from_str(&record.date, "%Y-%m-%d %H:%M:%S").unwrap();
        let mut row = row![record.date.format("%Y-%m-%d"), record.buy_or_sell, record.code];
        row.add_cell(Cell::new_align(&record.volume.to_string(), RIGHT));
        row.add_cell(Cell::new_align(&format!("{:.3}", &record.price), RIGHT));
        row.add_cell(Cell::new_align(&format!("{:.2}", record.fee), RIGHT));

        let financial_year = if record.date.date().month() < 7 {record.date.date().year()} else {record.date.date().year() + 1} as u32;

        match record.buy_or_sell {
            TradeType::BUY => {
                let total_string = format!("-{:.2}", calculate_buy_total(record.price, record.volume, record.fee));
                inventory.buy(record.date, &record.code, record.volume, record.price, record.fee);
                row.add_cell(Cell::new_align(&total_string, RIGHT));
                table.add_row(row);
            },
            TradeType::SELL => {
                let total_string = format!("-{:.2}", calculate_sell_total(record.price, record.volume, record.fee));
                let mut row = row![record.date.format("%Y-%m-%d"), record.buy_or_sell, record.code];
                row.add_cell(Cell::new_align(&record.volume.to_string(), RIGHT));
                row.add_cell(Cell::new_align(&format!("{:.2}", &record.price), RIGHT));
                row.add_cell(Cell::new_align(&format!("{:.2}", record.fee), RIGHT));
                row.add_cell(Cell::new_align(&total_string, RIGHT));
                row.add_cell(Cell::new_align("Fulfill", RIGHT).with_style(Attr::ForegroundColor(color::CYAN)));
                table.add_row(row);

                let fulfillment = inventory.sell(financial_year, &record.code, record.volume, record.price, record.fee);
                let mut fulfillment_table = Table::new();
                fulfillment_table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);

                for (quantity, bought_price, _bought_fee) in fulfillment.items {
                    let mut row = row!["", "", "", "", "", "", ""];
                    row.add_cell(Cell::new_align(&format!("{} x {:.2}", quantity, bought_price), RIGHT));
                    table.add_row(row);
                }
                let mut row = row!["", "", "", "", "", "", ""];
                let style = if fulfillment.net_profit < 0.0 { Attr::ForegroundColor(color::RED) } else { Attr::ForegroundColor(color::GREEN) };
                let profit_or_loss = if fulfillment.net_profit < 0.0 { "Loss" } else { "Profit" };
                row.add_cell(Cell::new_align(&format!("{}: {:.2}", profit_or_loss, fulfillment.net_profit), RIGHT).with_style(style));
                table.add_row(row);
            },
            TradeType::DIV => { // dividend
                inventory.record_dividend(financial_year, &record.code, record.volume, record.price);
                let mut row = row![record.date.format("%Y-%m-%d"), TradeType::DIV, record.code];
                row.add_cell(Cell::new_align(&record.volume.to_string(), RIGHT));
                row.add_cell(Cell::new_align(&format!("{:.3}", &record.price), RIGHT));
                row.add_cell(Cell::new(""));
                row.add_cell(Cell::new(""));
                let total_string = format!("{:.2}", calculate_buy_total(record.price, record.volume, record.fee));
                row.add_cell(Cell::new_align(&total_string, RIGHT).with_style(Attr::ForegroundColor(color::GREEN)));
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
    println!("{}", mem::size_of::<NaiveDateTime>());
}