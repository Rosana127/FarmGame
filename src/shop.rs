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
        // 基础种子
        seeds.insert("wheat".to_string(), 10);
        seeds.insert("corn".to_string(), 20);
        seeds.insert("carrot".to_string(), 15);
        
        // 高级种子
        seeds.insert("premium_wheat".to_string(), 25);
        seeds.insert("premium_corn".to_string(), 35);
        seeds.insert("premium_carrot".to_string(), 30);
        
        // 特殊种子
        seeds.insert("golden_wheat".to_string(), 50);
        seeds.insert("golden_corn".to_string(), 60);
        seeds.insert("golden_carrot".to_string(), 55);
        
        Self {
            seeds,
            balance: 100, // 初始金币
        }
    }

    pub fn buy_seed(&mut self, seed_type: &str) -> bool {
        // 将基础种子类型映射到实际种子类型
        let actual_seed_type = match seed_type {
            "wheat" => "wheat",
            "corn" => "corn",
            "carrot" => "carrot",
            _ => seed_type,
        };

        if let Some(&price) = self.seeds.get(actual_seed_type) {
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
            "premium_wheat" => 40,
            "premium_corn" => 60,
            "premium_carrot" => 50,
            "golden_wheat" => 80,
            "golden_corn" => 100,
            "golden_carrot" => 90,
            _ => 0,
        };
        self.balance += price;
    }

    pub fn get_balance(&self) -> u32 {
        self.balance
    }

    pub fn get_seed_price(&self, seed_type: &str) -> Option<u32> {
        // 将基础种子类型映射到实际种子类型
        let actual_seed_type = match seed_type {
            "wheat" => "wheat",
            "corn" => "corn",
            "carrot" => "carrot",
            _ => seed_type,
        };
        self.seeds.get(actual_seed_type).copied()
    }

    pub fn get_crop_price(&self, crop_type: &str) -> Option<u32> {
        match crop_type {
            "wheat" => Some(15),
            "corn" => Some(30),
            "carrot" => Some(25),
            "premium_wheat" => Some(40),
            "premium_corn" => Some(60),
            "premium_carrot" => Some(50),
            "golden_wheat" => Some(80),
            "golden_corn" => Some(100),
            "golden_carrot" => Some(90),
            _ => None,
        }
    }
} 