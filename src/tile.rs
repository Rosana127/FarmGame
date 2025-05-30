use serde::{Serialize, Deserialize};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CropType {
    Wheat,
    Corn,
    Carrot,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]  // TileState 用 Copy 也可以，方便赋值
pub enum TileState {
    Empty,
    Planted { crop: CropType, timer: u32 },
    Mature { crop: CropType },
    Pest { crop: CropType, timer: u32 }, 
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Tile {
    pub state: TileState,
    pub is_unlocked: bool,  // 新增：地块是否已解锁
}

impl Default for Tile {
    fn default() -> Self {
        Self {
            state: TileState::Empty,
            is_unlocked: false,
        }
    }
}