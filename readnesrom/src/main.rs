extern crate clap;
extern crate redwhite;
extern crate sdl2;

use clap::{App, ArgGroup};
use redwhite::{Tile, TILESIZE, PIXEL_UNIT};
use redwhite::error::Error;
use redwhite::ines::Ines;
use redwhite::palette;
use sdl2::EventPump;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Point;
use sdl2::render::WindowCanvas;

const NTILE: u32 = 16;

fn make_tiles(rom: &Ines) -> Vec<Tile> {
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

fn draw_tile_at(tile: &Tile, x: i32, y: i32, canvas: &mut WindowCanvas, pal: &[Color;4]) -> Result<(), Error> {
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
            canvas.draw_points(&points[..]).map_err(|s| Error::new(s))?;

        }
    }
    Ok(())
}

fn dump_tiles(tiles: &Vec<Tile>) -> Result<(), Error> {
    let height = NTILE * (TILESIZE as u32) * (PIXEL_UNIT as u32);
    let width = height * 2;

    let (mut canvas, mut event_pump) = init_window("pattern table", width, height)?;

    canvas.set_draw_color(Color::RGB(255, 255, 255));
    canvas.clear();

    // Since there is no attribute table, only the 2 bits
    // from the pattern table, we fix the colors used here.
    let pal = [
        Color::RGB(255, 255, 255),
        Color::RGB(255, 0, 0),
        Color::RGB(0, 255, 0),
        Color::RGB(0, 0, 255),
    ];

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


fn dump_pattern_table(file: &str) -> Result<(), Error> {
    let rom = Ines::from_file(file)?;
    let tiles = make_tiles(&rom);
    dump_tiles(&tiles)
}

fn init_window(title: &str, width: u32, height: u32) -> Result<(WindowCanvas, EventPump), Error> {
    let sdl_ctx = sdl2::init().map_err(|s| Error::new(s))?;
    let video = sdl_ctx.video().map_err(|s| Error::new(s))?;
    let window = video.window(title, width, height).position_centered().build()?;

    let canvas = window.into_canvas().build()?;
    let event_pump = sdl_ctx.event_pump().unwrap();

    Ok((canvas, event_pump))
}

fn dump_palette(file: &str) -> Result<(), Error> {
    let side = (TILESIZE as u32) * (PIXEL_UNIT as u32);
    let height = 4 * side as u32;
    let width = 16 * side as u32;

    let (mut canvas, mut event_pump) = init_window("palette", width, height)?;

    canvas.set_draw_color(Color::RGB(255, 255, 255));
    canvas.clear();

    let pals = palette::palette_from_file(file)?;

    for (r, pal) in pals.iter().enumerate() {
        for (c, color) in pal.iter().enumerate() {
            canvas.set_draw_color(*color);
            let mut points = vec![];

            for px in c as u32 * side..c as u32 * side + side {
                for py in r as u32 * side..r as u32 * side + side {
                    points.push(Point::new(px as i32, py as i32))
                }
            }

            canvas.draw_points(&points[..]).map_err(|s| Error::new(s))?;
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

fn main() -> Result<(), Error> {
    let args = App::new("readnesrom")
                .version("0.1")
                .about("Read iNES rom files")
                .args_from_usage("-p, --palette 'Dump palette file'
                                  -r, --rom     'Dump iNES rom file'
                                  <FILE>        'Target file'")
                .group(ArgGroup::with_name("action")
                       .args(&["palette", "rom"])
                       .required(true))
                .get_matches();

    let file = args.value_of("FILE").unwrap();

    if args.is_present("palette") {
        dump_palette(file)
    }
    else {
        dump_pattern_table(file)
    }
}
