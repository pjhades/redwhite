extern crate redwhite;
extern crate sdl2;

use redwhite::{Tile, TILESIZE, PIXEL_UNIT};
use redwhite::error::Error;
use redwhite::ines::Ines;
use redwhite::palette::{self, Palette};
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Point;
use sdl2::render::WindowCanvas;

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
            print!("{:08b} {:08b}  ->  ", chunk[i], chunk[i + TILESIZE]);
            for j in (0..TILESIZE).rev() {
                let val = (tile[i] >> (j * 4)) & 3;
                if val == 0 {
                    print!(".");
                }
                else {
                    print!("{:1x}", val);
                }
            }
            println!("");
        }
        println!("");
        tiles.push(tile);
    }

    tiles
}

fn draw_tile_at(tile: &Tile, x: i32, y: i32, canvas: &mut WindowCanvas, pal: &Palette) -> Result<(), Error> {
    for r in 0..TILESIZE as i32 {
        let row = tile[r as usize];
        for c in (0..TILESIZE as i32).rev() {
            match (row >> (c * 4)) & 0xf {
                0 => canvas.set_draw_color(Color::RGB(255, 255, 255)),
                idx => canvas.set_draw_color(pal[idx as usize]),
            }
            let mut points = vec![];
            for px in 0..PIXEL_UNIT {
                for py in 0..PIXEL_UNIT {
                    points.push(Point::new(x + (TILESIZE as i32 - c - 1) * PIXEL_UNIT as i32 + px as i32,
                                y + r * PIXEL_UNIT as i32 + py as i32));
                }
            }
            canvas.draw_points(&points[..]).map_err(|s| Error::new(s));

        }
    }
    Ok(())
}

fn dump(tiles: &Vec<Tile>) -> Result<(), Error> {
    let sdl_ctx = sdl2::init().map_err(|s| Error::new(s))?;
    let video = sdl_ctx.video().map_err(|s| Error::new(s))?;
    let pixel_height = NTILE * (TILESIZE as u32) * (PIXEL_UNIT as u32);
    let pixel_width = pixel_height * 2;
    let window = video.window("ptdump", pixel_width as u32, pixel_height as u32)
        .position_centered().build()?;
    let mut canvas = window.into_canvas().build()?;
    let mut event_pump = sdl_ctx.event_pump().unwrap();

    canvas.set_draw_color(Color::RGB(255, 255, 255));
    canvas.clear();

    let pals = palette::palette_from_file("./palette.pal")?;
    // fix palette here
    let pal = [pals[1][6], pals[2][7],pals[3][7],pals[1][8],
               pals[1][6], pals[1][6],pals[1][6],pals[1][6],
               pals[1][6], pals[1][6],pals[1][6],pals[1][6],
               pals[1][6], pals[1][6],pals[1][6],pals[1][6]];


    for chunk in tiles.chunks(512) {
        for row in 0..16 {
            for col in 0..16 {
                let x = (col * TILESIZE * PIXEL_UNIT) as i32;
                let y = (row * TILESIZE * PIXEL_UNIT) as i32;
                draw_tile_at(&chunk[row * 16 + col], x, y, &mut canvas, &pal)?;
            }
        }
        for row in 0..16 {
            for col in 16..32 {
                let x = (col * TILESIZE * PIXEL_UNIT) as i32;
                let y = (row * TILESIZE * PIXEL_UNIT) as i32;
                draw_tile_at(&chunk[256 + row * 16 + col - 16], x, y, &mut canvas, &pal)?;
            }
        }
    }

    canvas.present();
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
    let rom = Ines::from_file("/Users/pjhades/code/test/super-mario-bro.nes")?;
    let tiles = merge_tiles(&rom);
    dump(&tiles)
}

fn main() {
    match run() {
        Ok(()) => (),
        Err(e) => println!("{}", e),
    }
}
