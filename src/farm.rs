use crate::tile::{Tile, TileState, CropType};

pub struct Farm {
    pub grid: Vec<Vec<Tile>>,
}

impl Farm {
    pub fn new(rows: usize, cols: usize) -> Self {
        let row = vec![Tile { state: TileState::Empty }; cols];
        Self { grid: vec![row; rows] }
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
        if matches!(self.grid[row][col].state, TileState::Mature { .. }) {
            self.grid[row][col].state = TileState::Empty;
        }
    }
}