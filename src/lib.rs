extern crate sdl2;

pub mod cpu;
pub mod ppu;
pub mod memory;
pub mod ines;
pub mod error;
pub mod palette;

pub const PX_SCALE: usize = 3;
// Each tile is a 8x8 grid of pixels.
pub const TILE_SIZE_PX: usize = 8;
// The color of each pixel is described by 4 bits.
pub type Tile = [u32;TILE_SIZE_PX];
// Pattern table size in tiles
pub const PATTAB_TILES: usize = 512;
