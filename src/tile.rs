use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CropType {
    Wheat,
    Corn,
    Carrot,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]  // TileState 用 Copy 也可以，方便赋值
pub enum TileState {
    Empty,
    Planted { crop: CropType, timer: u32 },
    Mature { crop: CropType },
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Tile {
    pub state: TileState,
}

impl CropType {
    pub fn sell_price(&self) -> u32 {
        match self {
            CropType::Wheat => 15,
            CropType::Corn => 25, 
            CropType::Carrot => 20,
        }
    }
    
    pub fn growth_time(&self) -> u64 {
        match self {
            CropType::Wheat => 10,
            CropType::Corn => 15,
            CropType::Carrot => 12,
        }
    }
}

