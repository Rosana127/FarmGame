use crate::tile::{Tile, TileState, CropType};
use crate::inventory::Inventory;

pub struct Farm {
    pub grid: Vec<Vec<Tile>>,
    pub inventory: Inventory,
}

impl Farm {
    pub fn new(rows: usize, cols: usize) -> Self {
        let row = vec![Tile { state: TileState::Empty }; cols];
        Self {
            grid: vec![row; rows],
            inventory: Inventory::new(),
        }
    }

    pub fn tick(&mut self) {
        for row in &mut self.grid {
            for tile in row {
                if let TileState::Planted { crop, timer } = tile.state {
                    if timer == 0 {
                        tile.state = TileState::Mature { crop };
                    } else {
                        tile.state = TileState::Planted { crop, timer: timer - 1 };
                    }
                }
            }
        }
    }

    // 在 src/farm.rs 文件中
    pub fn plant(&mut self, row: usize, col: usize, crop: CropType) -> bool { // 返回 bool 表示成功与否
        let crop_name = match crop { //
            CropType::Wheat => "wheat", //
            CropType::Corn => "corn", //
            CropType::Carrot => "carrot", //
        };

        if self.grid[row][col].state == TileState::Empty { //
            if self.inventory.remove_seed(crop_name) { // 检查并移除种子
                let timer = Self::get_growth_time(crop); //
                self.grid[row][col].state = TileState::Planted { crop, timer }; //
                return true; // 种植成功
            } else {
                // 没有可用的种子
                return false;
            }
        }
        // 地块不为空或其他前置条件未满足
        false
    }

    pub fn harvest(&mut self, row: usize, col: usize) {
        if let TileState::Mature { crop } = self.grid[row][col].state {
            self.grid[row][col].state = TileState::Empty;
            let crop_name = match crop {
                CropType::Wheat => "wheat",
                CropType::Corn => "corn",
                CropType::Carrot => "carrot",
            };
            self.inventory.add_crop(crop_name);
        }
    }

    pub fn get_inventory(&self) -> (std::collections::HashMap<String, u32>, std::collections::HashMap<String, u32>) {
        self.inventory.get_items()
    }

    pub fn get_growth_time(crop: CropType) -> u32 {
        match crop {
            CropType::Carrot => 3,
            CropType::Corn => 10,
            CropType::Wheat => 5,
        }
    }
        pub fn get_crop_info(&self, row: usize, col: usize) -> String {
        let tile = &self.grid[row][col];
        
        match tile.state {
            TileState::Empty => {
                serde_json::json!({
                    "state": "empty",
                    "message": "空地 - 可以种植"
                }).to_string()
            }
            TileState::Planted { crop, timer } => {
                let crop_name = match crop {
                    CropType::Wheat => "小麦",
                    CropType::Corn => "玉米", 
                    CropType::Carrot => "胡萝卜",
                };
                
                let total_time = Self::get_growth_time(crop);
                let progress = total_time - timer;
                let expected_profit = Self::get_crop_value(crop);
                
                serde_json::json!({
                    "state": "growing",
                    "crop_name": crop_name,
                    "progress": progress,
                    "total_time": total_time,
                    "remaining_days": timer,
                    "expected_profit": expected_profit,
                    "message": format!("{} - 生长中 {}/{} 天，还需 {} 天成熟", 
                              crop_name, progress, total_time, timer)
                }).to_string()
            }
            TileState::Mature { crop } => {
                let crop_name = match crop {
                    CropType::Wheat => "小麦",
                    CropType::Corn => "玉米",
                    CropType::Carrot => "胡萝卜",
                };
                
                let profit = Self::get_crop_value(crop);
                
                serde_json::json!({
                    "state": "mature", 
                    "crop_name": crop_name,
                    "profit": profit,
                    "message": format!("{} - 已成熟，可收获 (价值: {}金币)", crop_name, profit)
                }).to_string()
            }
        }
    }

    // 添加作物价值计算方法
    fn get_crop_value(crop: CropType) -> u32 {
        match crop {
            CropType::Wheat => 15,   // 小麦卖15金币
            CropType::Corn => 30,    // 玉米卖30金币  
            CropType::Carrot => 20,  // 胡萝卜卖20金币
        }
    }

}
