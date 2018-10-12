extern crate redwhite;
extern crate sdl2;

use redwhite::{Tile, TILESIZE};
use redwhite::error::{Error, ResultContext};
use redwhite::ines::Ines;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

const NTILE: u32 = 16;

fn merge_tiles(rom: &Ines) -> Vec<Tile> {
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

    tiles
}

fn dump(tiles: &Vec<Tile>) -> Result<(), Error> {
    let sdl_ctx = sdl2::init().map_err(|s| Error::new(s))?;
    let video = sdl_ctx.video().map_err(|s| Error::new(s))?;

    let pixel_height = NTILE * TILESIZE as u32;
    let pixel_width = pixel_height * 2;

    let window = video.window("ptdump", pixel_width, pixel_height).position_centered().build()?;
    let mut canvas = window.into_canvas().build()?;
    canvas.set_draw_color(Color::RGB(255, 255, 255));
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_ctx.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'running,
                _ => {}
            }
        }
    }

    Ok(())
}

fn run() -> Result<(), Error> {
    let rom = Ines::from_file("/Users/jing.peng/Downloads/super-mario-bro.nes")?;
    let tiles = merge_tiles(&rom);
    dump(&tiles)
}

fn main() {
    match run() {
        Ok(()) => (),
        Err(e) => println!("{}", e),
    }
}
