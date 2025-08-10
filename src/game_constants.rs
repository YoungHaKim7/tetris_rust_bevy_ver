pub const TITLE: &'static str = "Tetris in Rust";

pub const NUM_BLOCKS_X: usize = 10;
pub const NUM_BLOCKS_Y: usize = 18;


pub const TEXTURE_SIZE: u32 = 32;


pub const WIDTH: u32 = NUM_BLOCKS_X as u32 * TEXTURE_SIZE;
pub const HEIGHT: u32 = NUM_BLOCKS_Y as u32 * TEXTURE_SIZE;

pub const NUM_LEVELS: usize = 10;
pub const LEVEL_TIMES: [usize; NUM_LEVELS] = [3000, 850, 700, 600, 500, 400, 300, 250, 221, 190];

