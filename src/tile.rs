#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CropType {
    Wheat,
    Corn,
    Carrot,
}

#[derive(Clone, Copy, PartialEq, Eq)]  // TileState 用 Copy 也可以，方便赋值
pub enum TileState {
    Empty,
    Planted { crop: CropType, timer: u32 },
    Mature { crop: CropType },
}

#[derive(Clone)]
pub struct Tile {
    pub state: TileState,
}
