use super::tile::{CropType, Tile, TileState, FertilizerType};
use super::inventory::Inventory;
use serde::{Serialize, Deserialize};
use rand::Rng;

// 表示一个农场，包含瓦片网格和库存
#[derive(Serialize, Deserialize)]
pub struct Farm {
    pub grid: Vec<Vec<Tile>>,  // 农场网格，每个瓦片包含状态和作物信息
    pub inventory: Inventory,  // 库存，包含种子、肥料和作物
}

impl Farm {
    // 创建一个新的农场，初始化网格和库存
    pub fn new(width: usize, height: usize) -> Self {
        // 创建一个宽为 width，高为 height 的网格，每个瓦片初始化为空状态
        let grid = vec![vec![Tile::new(); width]; height];
        // 创建一个新的库存，用于管理种子、肥料和作物
        Self {
            grid,
            inventory: Inventory::new(),
        }
    }
    // 处理农场中的时间流逝，不考虑虫害
    pub fn tick_without_infestation(&mut self) {
        // 遍历网格中的每一行
        for row in self.grid.iter_mut() {
            // 遍历当前行中的每个瓦片
            for tile in row.iter_mut() {
                // 如果当前瓦片处于种植状态，则增加计时器
                match &mut tile.state {
                    TileState::Planted { timer, .. } => {
                        *timer += 1;
                        // 如果计时器达到生长时间，则将作物状态改为成熟
                        if *timer >= 5 {
                            if let TileState::Planted { crop, .. } = std::mem::replace(&mut tile.state, TileState::Empty) {
                                // 将作物状态改为成熟
                                tile.state = TileState::Mature { crop };
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // 处理农场中的时间流逝，考虑虫害
    pub fn tick(&mut self) {
        // 遍历网格中的每一行
        for row in self.grid.iter_mut() {
            // 遍历当前行中的每个瓦片
            for tile in row.iter_mut() {
                // 如果当前瓦片处于种植状态，则增加计时器
                if let TileState::Planted { crop, timer, fertilizer } = &mut tile.state {
                    *timer += 1;
                    // 使用 CropType 中定义的统一方法
                    let adjusted_time = crop.growth_time_with_fertilizer(*fertilizer);
                    if *timer >= adjusted_time {
                        tile.state = TileState::Mature { crop: *crop };
                    }
                }
            }
        }
        self.random_infest(); 
    }

    // 种植作物，如果成功种植则返回 true
    pub fn plant(&mut self, row: usize, col: usize, crop: CropType, seed_key: String) -> bool {
        // 检查坐标是否在网格范围内
        if row < self.grid.len() && col < self.grid[0].len() {
            // 获取指定位置的瓦片
            let tile = &mut self.grid[row][col];
            // 检查瓦片是否可以种植
            if tile.can_plant() && self.inventory.remove_seed(&seed_key) {
                tile.state = TileState::Planted {
                    crop,
                    timer: 0,
                    fertilizer: FertilizerType::None,
                };
                return true;
            }
        }
        false
    }

    // 收获作物，如果成功收获则返回 true
    pub fn harvest(&mut self, row: usize, col: usize) {
        // 检查坐标是否在网格范围内
        if row < self.grid.len() && col < self.grid[0].len() {
            // 获取指定位置的瓦片
            let tile = &mut self.grid[row][col];
            // 如果当前瓦片处于成熟状态，则收获作物
            if let TileState::Mature { crop } = tile.state {
                let crop_str = match crop {
                    CropType::Wheat => "wheat",
                    CropType::PremiumWheat => "premium_wheat",
                    CropType::GoldenWheat => "golden_wheat",
                    CropType::Corn => "corn",
                    CropType::PremiumCorn => "premium_corn",
                    CropType::GoldenCorn => "golden_corn",
                    CropType::Carrot => "carrot",
                    CropType::PremiumCarrot => "premium_carrot",
                    CropType::GoldenCarrot => "golden_carrot",
                };
                self.inventory.add_crop(crop_str);
                tile.state = TileState::Empty;
            }
        }
    }

    // 随机产生虫害，每帧 2% 概率变成虫害
    pub fn random_infest(&mut self) {
        let mut rng = rand::thread_rng();
        // 遍历网格中的每一行
        for row in &mut self.grid {
            // 遍历当前行中的每个瓦片
            for tile in row {
                // 如果当前瓦片处于种植状态，则产生虫害
                if let TileState::Planted { crop, .. } = tile.state {
                    // 随机值 0.0 ~ 1.0
                    let chance: f32 = rng.gen();
                    // 每帧 2% 概率变成虫害
                    if chance < 0.02 {
                        // 将作物状态改为虫害
                        tile.state = TileState::Infested { crop };
                        crate::utils::show_message("⚠️ 有作物遭遇虫害了！");
                    }
                }
            }
        }
    }

    // 施肥，如果成功施肥则返回 true
    pub fn fertilize(&mut self, row: usize, col: usize, fertilizer_type: &str) -> bool {
        // 检查坐标是否在网格范围内
        if row < self.grid.len() && col < self.grid[0].len() {
            // 获取指定位置的瓦片
            let tile = &mut self.grid[row][col];
            // 获取肥料类型
            let fertilizer = match fertilizer_type {
                "basic_fertilizer" => FertilizerType::Basic,
                "premium_fertilizer" => FertilizerType::Premium,
                "super_fertilizer" => FertilizerType::Super,
                _ => FertilizerType::None,
            };
            // 检查瓦片是否可以施肥
            if tile.can_fertilize() && self.inventory.remove_fertilizer(fertilizer_type) {
                // 施肥
                return tile.apply_fertilizer(fertilizer);
            }
        }
        false
    }

    // 获取作物信息，如果坐标无效则返回错误信息
    pub fn get_crop_info(&self, row: usize, col: usize) -> String {
        // 检查坐标是否在网格范围内
        if row < self.grid.len() && col < self.grid[0].len() {
            // 获取指定位置的瓦片
            let tile = &self.grid[row][col];
            // 直接返回 tile 的信息文本
            tile.get_crop_info()
        } else {
            "无效位置".to_string()
        }
    }

    // 获取完整库存，返回种子、肥料和作物
    pub fn get_full_inventory(&self) -> (std::collections::HashMap<String, u32>, std::collections::HashMap<String, u32>, std::collections::HashMap<String, u32>) {
        self.inventory.get_all_items()
    }

    // 获取库存，返回种子和肥料
    pub fn get_inventory(&self) -> (std::collections::HashMap<String, u32>, std::collections::HashMap<String, u32>) {
        // 获取库存
        self.inventory.get_items()
    }
}