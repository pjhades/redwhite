extern crate sdl2;

pub mod cpu;
pub mod mem;
pub mod ines;
pub mod error;
pub mod palette;

pub const PIXEL_UNIT: usize = 4;
// Each tile is a 8x8 grid of pixels.
pub const TILESIZE: usize = 8;
// The color of each pixel is described with 4 bits.
pub type Tile = [u32;TILESIZE];
