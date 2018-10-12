extern crate redwhite;

use redwhite::{TILESIZE, ines, error};
use error::{Error, ResultContext};

type Tile = [[u8;TILESIZE];TILESIZE];

fn run() -> Result<(), Error> {
    let rom = ines::Ines::from_file("/Users/jing.peng/Downloads/super-mario-bro.nes")?;
    let mut tiles: Vec<Tile> = Vec::with_capacity(rom.chrrom.len() / 16);

    for chunk in rom.chrrom.chunks(16) {
        let mut tile = Tile::default();
        for i in (0..TILESIZE) {
            for j in (0..TILESIZE) {
                tile[i][TILESIZE - j - 1] = ((((chunk[i + 8] >> j)) & 0x1) << 1) | ((chunk[i] >> j) & 0x1);
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
