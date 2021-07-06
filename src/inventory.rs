use std::collections::HashMap;
use chrono::NaiveDateTime;
use crate::{TradingRecord, TradeType};

struct BuyItem {
    date: NaiveDateTime,
    shares: u32,
    price: f32,
    remaining_fee: f32
}

struct InventoryItem {
    date_acquired: NaiveDateTime,
    quantity: u32,
    price: f32,
    remaining_fee: f32
}

pub struct Fulfillment {
    // quantity, bought price, buying transaction fee
    pub(crate) items: Vec<(u32, f32, f32)>,
    pub(crate) net_profit: f32
}

// Output of Inventory is a list of ConsolidatedTransaction in chronological ordered
pub struct ConsolidatedTransaction {
    date: NaiveDateTime,
    trade_type: TradeType,
    code: String,
    quantity: u32,
    price: f32,
    fee: f32,
    amount_settled: f32, // for buy: quantity * price + fee, for sell: quantity * price - fee
    fulfillments: Option<Vec<SellingFulfillment>>,
    net_profit: f32
}

pub struct SellingFulfillment {
    date_purchased: NaiveDateTime,
    purchase_price: f32,
    quantity: u32,
    purchase_fee: f32,
    selling_fee: f32
}

pub struct Inventory {
    inventory_items: HashMap<String, Vec<InventoryItem>>,
    shares_map: HashMap<String, Vec<BuyItem>>,
    // financial year -> profit
    pub(crate) fy_profit_map: HashMap<u32, f32>
}

impl Inventory {
    pub fn new() -> Inventory {
        Inventory {
            inventory_items: HashMap::new(),
            shares_map: HashMap::new(),
            fy_profit_map: HashMap::new()
        }
    }

    // pub fn consolidated_transactions(&self) -> Vec<ConsolidatedTransaction> {
    //
    // }

    pub fn record_transaction(&mut self, trading_record: &TradingRecord) {
        let code = String::from(&trading_record.code);
        let inventory_items = self.inventory_items.entry(code).or_insert(Vec::new());

        match trading_record.buy_or_sell {
            TradeType::BUY => {
                Inventory::record_buy(inventory_items, trading_record);
            },
            TradeType::SELL => {

            },
            TradeType::DIV => {

            }
        }
    }

    fn record_buy(inventory_items: &mut Vec<InventoryItem>, trading_record: &TradingRecord) {
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
    }

    pub fn buy(&mut self, date: NaiveDateTime, code: &str, volume: u32, price: f32, fee: f32) {
        if !self.shares_map.contains_key(code) {
            self.shares_map.insert(String::from(code), Vec::new());
        }

        self.shares_map.get_mut(code).unwrap().push(BuyItem {
            date,
            shares: volume,
            price: price,
            remaining_fee: fee
        });
    }

    pub fn sell(&mut self, fy: u32, code: &str, volume: u32, price: f32, fee: f32) -> Fulfillment {
        let mut items = Vec::new();
        let stocks = self.shares_map.get_mut(code).unwrap();
        let mut quantity_to_fulfill = volume;
        let mut net_profit = -fee;
        loop {
            let first_stock = stocks.get_mut(0).unwrap();
            let q = if quantity_to_fulfill <= first_stock.shares {quantity_to_fulfill} else {first_stock.shares};
            net_profit += q as f32 * (price - first_stock.price);
            quantity_to_fulfill -= q;
            first_stock.shares -= q;

            let buying_transaction_fee;
            if first_stock.remaining_fee > 0.0 {
                buying_transaction_fee = first_stock.remaining_fee;
                first_stock.remaining_fee = 0.0;
                net_profit -= buying_transaction_fee;
            } else {
                buying_transaction_fee = 0.0;
            }
            items.push((q, first_stock.price, buying_transaction_fee));
            if first_stock.shares == 0 {
                stocks.remove(0);
            }

            if quantity_to_fulfill == 0 {
                break;
            }
        }

        self.record_fy_profit(fy, net_profit);

        Fulfillment {
            items,
            net_profit
        }
    }

    pub fn record_dividend(&mut self, fy: u32, _code: &str, volume: u32, amount: f32) {
        self.record_fy_profit(fy, volume as f32 * amount);
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
