use std::collections::HashMap;
use chrono::{NaiveDateTime, Datelike, Duration};
use crate::{TradingRecord, TradeType};
use crate::TradeType::{BUY, DIV, SELL};

struct InventoryItem {
    date_acquired: NaiveDateTime,
    quantity: u32,
    price: f32,
    remaining_fee: f32
}

// Output of Inventory is a list of ConsolidatedTransaction in chronological ordered
pub struct ConsolidatedTransaction {
    pub date: NaiveDateTime,
    pub trade_type: TradeType,
    pub code: String,
    pub quantity: u32,
    pub price: f32,
    pub fee: f32,
    pub amount_settled: f32, // for buy: quantity * price + fee, for sell: quantity * price - fee
    pub fulfillments: Option<Vec<SellingFulfillment>>,
    pub net_profit: f32
}

pub struct SellingFulfillment {
    pub date_purchased: NaiveDateTime,
    pub purchase_price: f32,
    pub quantity: u32,
    pub purchase_fee: f32,
    pub selling_fee: f32,
    pub acquired_duration: Duration,
    pub profit: f32
}

pub struct Inventory {
    inventory_items: HashMap<String, Vec<InventoryItem>>,
    // financial year -> profit
    pub(crate) fy_profit_map: HashMap<u32, f32>,
    consolidated_transactions: Vec<ConsolidatedTransaction>
}

impl Inventory {
    pub fn new() -> Inventory {
        Inventory {
            inventory_items: HashMap::new(),
            fy_profit_map: HashMap::new(),
            consolidated_transactions: vec![]
        }
    }

    pub fn consolidated_transactions(&self) -> &Vec<ConsolidatedTransaction> {
        &self.consolidated_transactions
    }

    pub fn record_transaction(&mut self, trading_record: &TradingRecord) {
        match trading_record.buy_or_sell {
            TradeType::BUY => {
                self.record_buy(trading_record);
            },
            TradeType::SELL => {
                self.record_sell(trading_record);
            },
            TradeType::DIV => {
                self.record_dividend(trading_record);
            }
        }
    }

    fn record_buy(&mut self, trading_record: &TradingRecord) {
        let inventory_items = self.inventory_items.entry(String::from(&trading_record.code)).or_insert(Vec::new());

        let mut i: usize = 0;
        for item in inventory_items.iter() {
            if trading_record.date < item.date_acquired {
                break;
            }
            i += 1;
        }
        inventory_items.insert(i, InventoryItem {
            date_acquired: trading_record.date,
            quantity: trading_record.volume,
            price: trading_record.price,
            remaining_fee: trading_record.fee
        });

        self.consolidated_transactions.push(ConsolidatedTransaction {
            date: trading_record.date,
            trade_type: BUY,
            code: String::from(&trading_record.code),
            quantity: trading_record.volume,
            price: trading_record.price,
            fee: trading_record.fee,
            amount_settled: trading_record.price * trading_record.volume as f32 + trading_record.fee,
            fulfillments: None,
            net_profit: 0.0
        });
    }

    fn record_sell(&mut self, trading_record: &TradingRecord) {
        let inventory_items = self.inventory_items.get_mut(&trading_record.code).unwrap();
        let mut quantity_to_fulfill = trading_record.volume;
        let mut net_profit = 0.0;
        let mut fulfillments: Vec<SellingFulfillment> = vec![];

        let mut i = 0;
        loop {
            let earliest_item = inventory_items.get_mut(0).unwrap();
            let current_round_quantity = if quantity_to_fulfill <= earliest_item.quantity { quantity_to_fulfill } else { earliest_item.quantity };
            quantity_to_fulfill -= current_round_quantity;
            earliest_item.quantity -= current_round_quantity;

            let purchase_fee;
            if earliest_item.remaining_fee > 0.0 {
                purchase_fee = earliest_item.remaining_fee;
                earliest_item.remaining_fee = 0.0;
            } else {
                purchase_fee = 0.0;
            }

            let selling_fee = if i == 0 { trading_record.fee } else { 0.0 };

            let fulfillment_profit = (trading_record.price - earliest_item.price) * current_round_quantity as f32 - purchase_fee - selling_fee;
            fulfillments.push(SellingFulfillment {
                date_purchased: earliest_item.date_acquired,
                purchase_price: earliest_item.price,
                quantity: current_round_quantity,
                purchase_fee,
                selling_fee,
                acquired_duration: trading_record.date - earliest_item.date_acquired,
                profit: fulfillment_profit
            });

            net_profit += fulfillment_profit;

            if earliest_item.quantity == 0 {
                inventory_items.remove(0);
            }

            if quantity_to_fulfill == 0 {
                break;
            }

            i += 1;
        }

        self.consolidated_transactions.push(ConsolidatedTransaction {
            date: trading_record.date,
            trade_type: SELL,
            code: String::from(&trading_record.code),
            quantity: trading_record.volume,
            price: trading_record.price,
            fee: trading_record.fee,
            amount_settled: trading_record.price * trading_record.volume as f32 - trading_record.fee,
            fulfillments: Some(fulfillments),
            net_profit
        });

        let financial_year = if trading_record.date.month() < 7 {trading_record.date.year()} else {trading_record.date.year() + 1} as u32;
        self.record_fy_profit(financial_year, net_profit);
    }

    pub fn record_dividend(&mut self, trading_record: &TradingRecord) {
        let financial_year = if trading_record.date.month() < 7 {trading_record.date.year()} else {trading_record.date.year() + 1} as u32;
        self.record_fy_profit(financial_year, trading_record.price * trading_record.volume as f32);

        let amount_settled = trading_record.price * trading_record.volume as f32;
        self.consolidated_transactions.push(ConsolidatedTransaction {
            date: trading_record.date,
            trade_type: DIV,
            code: String::from(&trading_record.code),
            quantity: trading_record.volume,
            price: trading_record.price,
            fee: 0.0,
            amount_settled,
            fulfillments: None,
            net_profit: amount_settled
        });
    }

    fn record_fy_profit(&mut self, fy: u32, profit: f32) {
        if !self.fy_profit_map.contains_key(&fy) {
            self.fy_profit_map.insert(fy, profit);
        } else {
            let new_profit = self.fy_profit_map.get(&fy).unwrap() + profit;
            self.fy_profit_map.insert(fy, new_profit);
        }
    }
}
