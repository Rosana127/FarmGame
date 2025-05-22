use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Inventory {
    pub items: HashMap<String, u32>,
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
        }
    }

    pub fn add(&mut self, item: &str) {
        *self.items.entry(item.to_string()).or_insert(0) += 1;
    }

    pub fn get_items(&self) -> HashMap<String, u32> {
        self.items.clone()
    }
}