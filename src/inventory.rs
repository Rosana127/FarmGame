use std::collections::HashMap;
use serde::{Serialize, Deserialize};

// 表示一个库存，包含种子、作物和肥料
#[derive(Serialize, Deserialize, Clone)]
pub struct Inventory {
    pub seeds: HashMap<String, u32>,  // 种子，键为种子名称，值为数量
    pub crops: HashMap<String, u32>,  // 作物，键为作物名称，值为数量
    pub fertilizers: HashMap<String, u32>,
}

impl Inventory {
    // 创建一个新的库存，初始化种子、作物和肥料
    pub fn new() -> Self {
        Self {
            seeds: HashMap::new(),
            crops: HashMap::new(),
            fertilizers: HashMap::new(),
        }
    }

    // 添加种子，如果种子不存在则创建
    pub fn add_seed(&mut self, seed: &str) {
        *self.seeds.entry(seed.to_string()).or_insert(0) += 1;
    }

    // 添加作物，如果作物不存在则创建
    pub fn add_crop(&mut self, crop: &str) {
        *self.crops.entry(crop.to_string()).or_insert(0) += 1;  
    }

    // 添加肥料，如果肥料不存在则创建
    pub fn add_fertilizer(&mut self, fertilizer: &str) {
        *self.fertilizers.entry(fertilizer.to_string()).or_insert(0) += 1;
    }

    // 移除种子，如果种子不存在则返回 false
    pub fn remove_seed(&mut self, seed: &str) -> bool {
        if let Some(count) = self.seeds.get_mut(seed) {
            if *count > 0 {
                *count -= 1;
                if *count == 0 {
                    self.seeds.remove(seed);
                }
                return true;
            }
        }
        false
    }

    // 移除作物，如果作物不存在则返回 false
    pub fn remove_crop(&mut self, crop: &str) -> bool {
        if let Some(count) = self.crops.get_mut(crop) {
            if *count > 0 {
                *count -= 1;
                if *count == 0 {
                    self.crops.remove(crop);
                }
                return true;
            }
        }
        false
    }

    // 移除肥料，如果肥料不存在则返回 false
    pub fn remove_fertilizer(&mut self, fertilizer: &str) -> bool {
        if let Some(count) = self.fertilizers.get_mut(fertilizer) {
            if *count > 0 {
                *count -= 1;
                if *count == 0 {
                    self.fertilizers.remove(fertilizer);
                }
                return true;
            }
        }
        false
    }

    // 获取库存，返回种子和作物
    pub fn get_items(&self) -> (HashMap<String, u32>, HashMap<String, u32>) {
        (self.seeds.clone(), self.crops.clone())
    }

    // 获取完整库存，返回种子、作物和肥料
    pub fn get_all_items(&self) -> (HashMap<String, u32>, HashMap<String, u32>, HashMap<String, u32>) {
        (self.seeds.clone(), self.crops.clone(), self.fertilizers.clone())
    }
}