use crate::tile::{Tile, TileState, CropType};
use crate::inventory::Inventory;

pub struct Farm {
    pub grid: Vec<Vec<Tile>>,
    pub inventory: Inventory,
    pub rows: usize,  // 添加 rows 字段
    pub cols: usize,  // 添加 cols 字段
}

impl Farm {
    pub fn new(rows: usize, cols: usize) -> Self {
        let row = vec![Tile { state: TileState::Empty }; cols];
        Self {
            grid: vec![row; rows],
            inventory: Inventory::new(),
            rows,  // 初始化 rows
            cols,  // 初始化 cols
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

    pub fn plant(&mut self, row: usize, col: usize, crop: CropType) -> bool {
        let crop_name = match crop {
            CropType::Wheat => "wheat",
            CropType::Corn => "corn",
            CropType::Carrot => "carrot",
        };

        // 检查位置是否有效
        if row >= self.rows || col >= self.cols {
            return false;
        }

        // 检查地块是否为空
        if self.grid[row][col].state != TileState::Empty {
            return false;
        }

        // 检查是否有足够的种子
        if !self.inventory.remove_seed(crop_name) {
            return false;
        }

        // 种植作物
        let timer = Self::get_growth_time(crop);
        self.grid[row][col].state = TileState::Planted { crop, timer };
        true
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

    // 修复后的 get_crop_info 方法
    pub fn get_crop_info(&self, row: usize, col: usize) -> String {
        if row >= self.rows || col >= self.cols {
            return serde_json::json!({
                "state": "invalid",
                "message": ""
            }).to_string();
        }

        let tile = &self.grid[row][col];
        match &tile.state {
            TileState::Empty => {
                serde_json::json!({
                    "state": "empty",
                    "message": "空地 - 可以种植作物",
                    "canPlant": true
                }).to_string()
            },
            TileState::Planted { crop, timer } => {
                let growth_time = Self::get_growth_time(*crop);
                let elapsed = growth_time - timer;
                let remaining = *timer;
                
                let crop_name = match crop {
                    CropType::Wheat => "小麦",
                    CropType::Corn => "玉米", 
                    CropType::Carrot => "胡萝卜",
                };
                
                let progress = (elapsed as f64 / growth_time as f64 * 100.0).min(100.0);
                
                serde_json::json!({
                    "state": "planted",
                    "crop": crop_name,
                    "message": format!(
                        "🌱 {} 幼苗\n⏱️ 成长进度: {:.1}%\n⏰ 剩余时间: {}秒\n💰 预期收益: {}金币", 
                        crop_name, 
                        progress,
                        remaining,
                        Self::get_crop_value(*crop)
                    ),
                    "progress": progress,
                    "remaining_time": remaining,
                    "expected_profit": Self::get_crop_value(*crop)
                }).to_string()
            },
            TileState::Mature { crop } => {
                let crop_name = match crop {
                    CropType::Wheat => "小麦",
                    CropType::Corn => "玉米",
                    CropType::Carrot => "胡萝卜", 
                };
                
                serde_json::json!({
                    "state": "mature",
                    "crop": crop_name,
                    "message": format!(
                        "✨ {} 已成熟！\n🎯 点击收获\n💰 价值: {}金币\n📊 生长周期: {}秒", 
                        crop_name,
                        Self::get_crop_value(*crop),
                        Self::get_growth_time(*crop)
                    ),
                    "sell_price": Self::get_crop_value(*crop),
                    "growth_time": Self::get_growth_time(*crop)
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