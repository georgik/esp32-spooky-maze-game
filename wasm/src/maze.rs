
pub struct Maze {
    pub width: u32,
    pub height: u32,
    pub visible_width: u32,
    pub visible_height: u32,
    pub data: [u8; 64*64],
    // Tile map should have small border top line and left column
    pub offset: u32,
    pub tile_width: u32,
    pub tile_height: u32,
}

impl Maze {
    pub fn new(width: u32, height: u32) -> Maze {
        Maze {
            width,
            height,
            visible_width: 20,
            visible_height: 16,
            data: [1; 64*64],
            offset: width+1,
            tile_width: 16,
            tile_height: 16,
        }
    }
}
