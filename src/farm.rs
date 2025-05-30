use crate::tile::{Tile, TileState, CropType};
use crate::inventory::Inventory;
use std::collections::HashMap;
use rand::{thread_rng, Rng};

use wasm_bindgen_futures;
use crate::draw_canvas_once;

pub struct Farm {
    pub grid: Vec<Vec<Tile>>,
    pub inventory: Inventory,
}

impl Farm {
    pub fn new(rows: usize, cols: usize) -> Self {
        let mut grid = Vec::with_capacity(rows);
        for row in 0..rows {
            let mut tiles = Vec::with_capacity(cols);
            for col in 0..cols {
                // 初始解锁前 4x4 区域
                let is_unlocked = row < 4 && col < 4;
                tiles.push(Tile {
                    state: TileState::Empty,
                    is_unlocked,
                });
            }
            grid.push(tiles);
        }
        Self {
            grid,
            inventory: Inventory::new(),
        }
    }

    pub fn tick(&mut self) {
        for row in &mut self.grid {
            for tile in row {
                match tile.state {
                    TileState::Planted { crop, timer } => {
                        if timer == 0 {
                            tile.state = TileState::Mature { crop };
                        } else {
                            // 有概率进入虫害状态
                            if thread_rng().gen_bool(0.01) {
                                tile.state = TileState::Pest {
                                    crop,
                                    timer: 10,
                                };
                            } else {
                                tile.state = TileState::Planted {
                                    crop,
                                    timer: timer - 1,
                                };
                            }
                        }
                    }

                    TileState::Mature { .. } => {
                        // 成熟状态不再进入虫害
                    }

                    TileState::Pest { crop, timer } => {
                        if timer <= 1 {
                            tile.state = TileState::Empty;
                        } else {
                            tile.state = TileState::Pest {
                                crop,
                                timer: timer - 1,
                            };
                        }
                    }

                    _ => {}
                }
            }
        }
    }

    // 使用杀虫剂：将 Pest 地块恢复为 Planted
    pub fn use_pesticide(&mut self, row: usize, col: usize) -> bool {
        let tile = &mut self.grid[row][col];
        if let TileState::Pest { crop, .. } = tile.state {
            if self.inventory.pesticide > 0 {
                self.inventory.pesticide -= 1;
    
                let remaining_time = Self::get_growth_time(crop);
                tile.state = TileState::Planted {
                    crop,
                    timer: remaining_time,
                };
    
                //wasm_bindgen_futures::spawn_local(async move {
                //    crate::draw_canvas_once(); // ✅ 刷新画布
                //});
                //crate::draw_canvas_once(); // 同步刷新，立即执行

    
                return true;
            }
        }
        false
    }
    
    
    

    // 种植作物
    pub fn plant(&mut self, row: usize, col: usize, crop: CropType) -> bool {
        if !self.grid[row][col].is_unlocked {
            return false;
        }

        let crop_name = match crop {
            CropType::Wheat => "wheat",
            CropType::Corn => "corn",
            CropType::Carrot => "carrot",
        };

        if self.grid[row][col].state == TileState::Empty {
            if self.inventory.remove_seed(crop_name) {
                let timer = Self::get_growth_time(crop);
                self.grid[row][col].state = TileState::Planted { crop, timer };
                return true;
            } else {
                return false;
            }
        }
        false
    }

    // 收获成熟作物
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

    // 获取背包内容（种子、作物、杀虫剂）
    pub fn get_inventory(&self) -> (
        HashMap<String, u32>,
        HashMap<String, u32>,
        u32,
    ) {
        self.inventory.get_items()
    }

    // 各作物的生长时间
    fn get_growth_time(crop: CropType) -> u32 {
        match crop {
            CropType::Carrot => 30,
            CropType::Corn => 60,
            CropType::Wheat => 15,
        }
    }

    // 使用铲子清除地块
    pub fn clear_tile(&mut self, row: usize, col: usize) {
        if row < self.grid.len() && col < self.grid[0].len() {
            self.grid[row][col].state = TileState::Empty;
        }
    }

    // 解锁地块（花费金币）
    pub fn unlock_tile(&mut self, row: usize, col: usize) -> bool {
        let mut unlocked = false;
        crate::SHOP.with(|shop_cell| {
            let mut shop = shop_cell.borrow_mut();
            if shop.balance >= 5 {
                shop.balance -= 5;
                self.grid[row][col].is_unlocked = true;
                unlocked = true;
            }
        });
        unlocked
    }
}
