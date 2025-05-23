use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Inventory {
    pub seeds: HashMap<String, u32>,
    pub crops: HashMap<String, u32>,
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            seeds: HashMap::new(),
            crops: HashMap::new(),
        }
    }

    pub fn add_seed(&mut self, seed: &str) {
        *self.seeds.entry(seed.to_string()).or_insert(0) += 1;
    }

    pub fn add_crop(&mut self, crop: &str) {
        *self.crops.entry(crop.to_string()).or_insert(0) += 1;
    }

    pub fn remove_seed(&mut self, seed: &str) -> bool {
        if let Some(count) = self.seeds.get_mut(seed) {
            if *count > 0 {
                *count -= 1;
                return true;
            }
        }
        false
    }

    pub fn get_items(&self) -> (HashMap<String, u32>, HashMap<String, u32>) {
        (self.seeds.clone(), self.crops.clone())
    }
}