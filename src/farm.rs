use super::tile::{CropType, Tile, TileState, FertilizerType};
use super::inventory::Inventory;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Farm {
    pub grid: Vec<Vec<Tile>>,
    pub inventory: Inventory,
}

impl Farm {
    pub fn new(width: usize, height: usize) -> Self {
        let grid = vec![vec![Tile::new(); width]; height];
        Self {
            grid,
            inventory: Inventory::new(),
        }
    }

    pub fn tick(&mut self) {
        for row in self.grid.iter_mut() {
            for tile in row.iter_mut() {
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
    }

    pub fn plant(&mut self, row: usize, col: usize, crop: CropType) -> bool {
        if row < self.grid.len() && col < self.grid[0].len() {
            let tile = &mut self.grid[row][col];
            let crop_str = match crop {
                CropType::Wheat => "wheat",
                CropType::Corn => "corn",
                CropType::Carrot => "carrot",
            };
            if tile.can_plant() && self.inventory.remove_seed(crop_str) {
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

    pub fn harvest(&mut self, row: usize, col: usize) {
        if row < self.grid.len() && col < self.grid[0].len() {
            let tile = &mut self.grid[row][col];
            if let TileState::Mature { crop } = tile.state {
                let crop_str = match crop {
                    CropType::Wheat => "wheat",
                    CropType::Corn => "corn",
                    CropType::Carrot => "carrot",
                };
                self.inventory.add_crop(crop_str);
                tile.state = TileState::Empty;
            }
        }
    }

    pub fn fertilize(&mut self, row: usize, col: usize, fertilizer_type: &str) -> bool {
        if row < self.grid.len() && col < self.grid[0].len() {
            let tile = &mut self.grid[row][col];
            let fertilizer = match fertilizer_type {
                "basic_fertilizer" => FertilizerType::Basic,
                "premium_fertilizer" => FertilizerType::Premium,
                "super_fertilizer" => FertilizerType::Super,
                _ => FertilizerType::None,
            };
            if tile.can_fertilize() && self.inventory.remove_fertilizer(fertilizer_type) {
                return tile.apply_fertilizer(fertilizer);
            }
        }
        false
    }

    pub fn get_crop_info(&self, row: usize, col: usize) -> String {
        if row < self.grid.len() && col < self.grid[0].len() {
            let tile = &self.grid[row][col];
            let message = tile.get_crop_info();
            let state = match tile.state {
                TileState::Empty => "empty",
                TileState::Planted { .. } => "planted",
                TileState::Mature { .. } => "mature",
            };
            serde_json::to_string(&serde_json::json!({
                "message": message,
                "state": state
            }))
            .unwrap_or_default()
        } else {
            "{}".to_string()
        }
    }

    pub fn get_full_inventory(&self) -> (std::collections::HashMap<String, u32>, std::collections::HashMap<String, u32>, std::collections::HashMap<String, u32>) {
        self.inventory.get_all_items()
    }

    pub fn get_inventory(&self) -> (std::collections::HashMap<String, u32>, std::collections::HashMap<String, u32>) {
        self.inventory.get_items()
    }
}