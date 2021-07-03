use std::collections::HashMap;

struct BuyItem {
    shares: u32,
    price: f32,
    remaining_fee: f32
}

pub struct Fulfillment {
    // quantity, bought price, buying transaction fee
    pub(crate) items: Vec<(u32, f32, f32)>,
    pub(crate) net_profit: f32
}

pub struct Inventory {
    shares_map: HashMap<String, Vec<BuyItem>>,
    // financial year -> profit
    pub(crate) fy_profit_map: HashMap<u32, f32>
}

impl Inventory {
    pub fn new() -> Inventory {
        Inventory {
            shares_map: HashMap::new(),
            fy_profit_map: HashMap::new()
        }
    }

    pub fn buy(&mut self, code: &str, volume: u32, price: f32, fee: f32) {
        if !self.shares_map.contains_key(code) {
            self.shares_map.insert(String::from(code), Vec::new());
        }

        self.shares_map.get_mut(code).unwrap().push(BuyItem {
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
            if (first_stock.remaining_fee > 0.0) {
                buying_transaction_fee = first_stock.remaining_fee;
                first_stock.remaining_fee = 0.0;
                net_profit -= buying_transaction_fee;
            } else {
                buying_transaction_fee = 0.0;
            }
            items.push((q, first_stock.price, buying_transaction_fee));
            if (first_stock.shares == 0) {
                stocks.remove(0);
            }

            if (quantity_to_fulfill == 0) {
                break;
            }
        }

        self.record_fy_profit(fy, net_profit);

        Fulfillment {
            items,
            net_profit
        }
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
