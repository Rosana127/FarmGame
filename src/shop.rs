use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Shop {
    pub seeds: HashMap<String, u32>,
    pub fertilizers: HashMap<String, u32>,
    pub balance: u32,
}

impl Shop {
    pub fn new() -> Self {
        let mut seeds = HashMap::new();
        seeds.insert("wheat".to_string(), 10);
        seeds.insert("corn".to_string(), 20);
        seeds.insert("carrot".to_string(), 15);
        seeds.insert("premium_wheat".to_string(), 25);
        seeds.insert("premium_corn".to_string(), 35);
        seeds.insert("premium_carrot".to_string(), 30);
        seeds.insert("golden_wheat".to_string(), 50);
        seeds.insert("golden_corn".to_string(), 60);
        seeds.insert("golden_carrot".to_string(), 55);

        let mut fertilizers = HashMap::new();
        fertilizers.insert("basic_fertilizer".to_string(), 25);
        fertilizers.insert("premium_fertilizer".to_string(), 50);
        fertilizers.insert("super_fertilizer".to_string(), 80);

        Self { seeds, fertilizers, balance: 100 }
    }

    pub fn buy_fertilizer(&mut self, fertilizer_type: &str) -> bool {
        if let Some(&price) = self.fertilizers.get(fertilizer_type) {
            if self.balance >= price {
                self.balance -= price;
                return true;
            }
        }
        false
    }

    pub fn get_fertilizer_price(&self, fertilizer_type: &str) -> Option<u32> {
        self.fertilizers.get(fertilizer_type).copied()
    }

    pub fn buy_seed(&mut self, seed_type: &str) -> bool {
        if let Some(&price) = self.seeds.get(seed_type) {
            if self.balance >= price {
                self.balance -= price;
                return true;
            }
        }
        false
    }

    pub fn sell_crop(&mut self, crop_type: &str) {
        let price = match crop_type {
            "wheat" => 15,
            "corn" => 25,
            "carrot" => 20,
            "premium_wheat" => 30,
            "premium_corn" => 50,
            "premium_carrot" => 40,
            "golden_wheat" => 75,
            "golden_corn" => 90,
            "golden_carrot" => 80,
            _ => 0,
        };
        self.balance += price;
    }

    pub fn get_crop_price(&self, crop_type: &str) -> Option<u32> {
        match crop_type {
            "wheat" => Some(15),
            "corn" => Some(25),
            "carrot" => Some(20),
            "premium_wheat" => Some(30),
            "premium_corn" => Some(50),
            "premium_carrot" => Some(40),
            "golden_wheat" => Some(75),
            "golden_corn" => Some(90),
            "golden_carrot" => Some(80),
            _ => None,
        }
    }

    pub fn get_balance(&self) -> u32 {
        self.balance
    }
}