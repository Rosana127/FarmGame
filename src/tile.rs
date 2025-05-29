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

