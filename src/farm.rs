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

    pub fn plant(&mut self, row: usize, col: usize, crop: CropType) {
        if self.grid[row][col].state == TileState::Empty {
            self.grid[row][col].state = TileState::Planted { crop, timer: 5 };
        }
    }

    pub fn harvest(&mut self, row: usize, col: usize) {
        if let TileState::Mature { crop } = self.grid[row][col].state {
            self.grid[row][col].state = TileState::Empty;
            let crop_name = match crop {
                CropType::Wheat => "wheat",
                CropType::Corn => "corn",
                CropType::Carrot => "carrot",
            };
            self.inventory.add(crop_name);
        }
    }

    pub fn get_inventory(&self) -> std::collections::HashMap<String, u32> {
        self.inventory.get_items()
    }
}
