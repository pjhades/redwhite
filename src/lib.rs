pub mod cpu;
pub mod mem;
pub mod ines;
pub mod error;

// Each tile is a 8x8 grid of pixels.
pub const TILESIZE: usize = 8;

// The color of each pixel is described with 4 bits.
pub type Tile = [u32;TILESIZE];
