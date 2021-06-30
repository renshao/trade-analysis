use std::collections::HashMap;

struct BuyItem {
    shares: u32,
    price: f32,
    remaining_fee: f32
}

pub struct Inventory {
    shares_map: HashMap<String, Vec<BuyItem>>
}

impl Inventory {
    pub fn new() -> Inventory {
        Inventory {
            shares_map: HashMap::new()
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
}
