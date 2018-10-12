extern crate redwhite;

use redwhite::{Tile, TILESIZE, ines, error};
use error::{Error, ResultContext};

fn run() -> Result<(), Error> {
    let rom = ines::Ines::from_file("/Users/jing.peng/Downloads/super-mario-bro.nes")?;
    let mut tiles: Vec<Tile> = Vec::with_capacity(rom.chrrom.len() / 16);

    for chunk in rom.chrrom.chunks(16) {
        let mut tile = Tile::default();
        for i in 0..TILESIZE {
            for j in 0..TILESIZE {
                tile[i] |= (((((chunk[i + TILESIZE] >> j) & 1) << 1) |
                           ((chunk[i] >> j) & 1)) as u32) << (j * 4);
            }
        }
        tiles.push(tile);
    }

    Ok(())
}

fn main() {
    match run() {
        Ok(()) => (),
        Err(e) => println!("{}", e),
    }
}
