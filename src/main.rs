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
use prettytable::{color};
use chrono::{NaiveDateTime};
use Alignment::RIGHT;
use term::Attr::ForegroundColor;

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

fn read_transactions() -> Result<(), Box<dyn Error>> {
    let mut inventory = Inventory::new();

    let mut table = Table::new();
    table.set_titles(row!["Date", "Trade", "Code", "Volume", "Price", "Fee", "Total", "Net Profit", "Fulfillment"]);

    let mut rdr = csv::Reader::from_reader(io::BufReader::new(File::open("trades.csv")?));
    for result in rdr.deserialize() {
        let record: TradingRecord = result?;
        inventory.record_transaction(&record);
    }

    for t in inventory.consolidated_transactions().iter() {
        let mut row = row![t.date.format("%Y-%m-%d"), t.trade_type, t.code];
        row.add_cell(Cell::new_align(&t.quantity.to_string(), RIGHT));
        row.add_cell(Cell::new_align(&format!("{:.3}", t.price), RIGHT));

        match t.trade_type {
            TradeType::BUY => {
                row.add_cell(Cell::new_align(&format!("{:.2}", t.fee), RIGHT));
                row.add_cell(Cell::new_align(&format!("-{:.2}", t.amount_settled), RIGHT));
                row.add_cell(Cell::new(""));
                table.add_row(row);
            }
            TradeType::SELL => {
                row.add_cell(Cell::new_align(&format!("{:.2}", t.fee), RIGHT));
                row.add_cell(Cell::new_align(&format!("{:.2}", t.amount_settled), RIGHT));
                let mut cell = Cell::new_align(&format!("{:.2}", t.net_profit), RIGHT);
                if t.net_profit >= 0.0 {
                    cell = cell.with_style(ForegroundColor(color::GREEN));
                } else  {
                    cell = cell.with_style(ForegroundColor(color::RED));
                }
                row.add_cell(cell);

                let mut fulfill_table = Table::new();
                fulfill_table.set_titles(row!["Qty", "Purchase Price", "Diff %", "Fees", "Net Profit", "Acquired Duration"]);
                let fulfill_table_format = format::FormatBuilder::from(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR)
                    .separators(&[format::LinePosition::Bottom], format::LineSeparator::new('=', '=', '+', '+')).build();
                fulfill_table.set_format(fulfill_table_format);

                for f in t.fulfillments.as_ref().unwrap() {
                    let mut row = row![];
                    row.add_cell(Cell::new_align(&f.quantity.to_string(), RIGHT));
                    row.add_cell(Cell::new_align(&format!("{:.3}", f.purchase_price), RIGHT));
                    row.add_cell(Cell::new_align(&format!("{:.1}", (t.price - f.purchase_price) / f.purchase_price * 100.0), RIGHT));
                    row.add_cell(Cell::new_align(&format!("{:.2}", f.purchase_fee + f.selling_fee), RIGHT));
                    row.add_cell(Cell::new_align(&format!("{:.2}", f.profit), RIGHT));
                    row.add_cell(Cell::new_align(&format!("{} days", f.acquired_duration.num_days()), RIGHT));
                    fulfill_table.add_row(row);
                }

                row.add_cell(Cell::new(&fulfill_table.to_string()));
                table.add_row(row);
            }
            TradeType::DIV => {
                row.add_cell(Cell::new(""));
                row.add_cell(Cell::new_align(&format!("{:.2}", t.amount_settled), RIGHT));
                row.add_cell(Cell::new_align(&format!("{:.2}", t.net_profit), RIGHT).with_style(ForegroundColor(color::GREEN)));
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