use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Shop {
    pub seeds: HashMap<String, u32>, // 种子价格
    pub balance: u32, // 玩家金币
}

impl Shop {
    pub fn new() -> Self {
        let mut seeds = HashMap::new();
        seeds.insert("wheat".to_string(), 10);
        seeds.insert("corn".to_string(), 20);
        seeds.insert("carrot".to_string(), 15);
        
        Self {
            seeds,
            balance: 100, // 初始金币
        }
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
            "corn" => 30,
            "carrot" => 25,
            _ => 0,
        };
        self.balance += price;
    }

    pub fn get_balance(&self) -> u32 {
        self.balance
    }

    pub fn get_seed_price(&self, seed_type: &str) -> Option<u32> {
        self.seeds.get(seed_type).copied()
    }
    pub fn get_crop_price(&self, crop_type: &str) -> Option<u32> { //
        match crop_type {
            "wheat" => Some(15), //
            "corn" => Some(30), //
            "carrot" => Some(25), //
            _ => None,
        }
    }
} 