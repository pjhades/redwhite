use std::fs::File;
use std::path::Path;
use std::io::Read;
use sdl2::pixels::Color;
use crate::error::Error;

const NCOLOR: usize = 16;
const NPALETTE: usize = 4;
const PALETTE_SIZE: usize = 48;

pub type Palette = [Color;NCOLOR];
pub type PaletteSet = [Palette;NPALETTE];

pub fn palette_from_file<P: AsRef<Path>>(path: P) -> Result<PaletteSet, Error> {
    let mut file = File::open(path)?;
    let mut pal = [[Color::RGB(255, 255, 255); NCOLOR]; NPALETTE];

    for i in 0..NPALETTE {
        let mut bytes = [0u8;PALETTE_SIZE];
        file.read_exact(&mut bytes)?;
        for (j, chunk) in bytes.chunks(3).enumerate() {
            pal[i][j] = Color::RGB(chunk[0], chunk[1], chunk[2]);
        }
    }

    Ok(pal)
}
