extern crate clap;
extern crate redwhite;
extern crate sdl2;

use std::process;
use clap::{App, ArgGroup};
use redwhite::{Tile, TILE_SIZE_PX, PX_SCALE, PATTAB_TILES};
use redwhite::error::Error;
use redwhite::ines::Ines;
use redwhite::palette;
use sdl2::EventPump;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;

fn make_tiles(rom: &Ines) -> Vec<Tile> {
    let mut tiles: Vec<Tile> = Vec::with_capacity(rom.chrrom.len() / 16);

    for chunk in rom.chrrom.chunks(16) {
        let mut tile = Tile::default();
        for i in 0..TILE_SIZE_PX {
            for j in 0..TILE_SIZE_PX {
                let hi_bit = (chunk[i + TILE_SIZE_PX] >> j) & 1;
                let lo_bit = (chunk[i] >> j) & 1;
                tile[i] |= (((hi_bit << 1) | lo_bit) as u32) << (j * 4);
            }
        }
        tiles.push(tile);
    }

    tiles
}

fn draw_tile_at(tile: &Tile, x: i32, y: i32, canvas: &mut WindowCanvas, pal: &[Color;4]) -> Result<(), Error> {
    for row in 0..TILE_SIZE_PX {
        for col in (0..TILE_SIZE_PX).rev() {
            let color_idx = (tile[row] >> (col * 4)) & 0xf;
            canvas.set_draw_color(pal[color_idx as usize]);

            let dotx = x + (TILE_SIZE_PX - col - 1) as i32 * PX_SCALE as i32;
            let doty = y + row as i32 * PX_SCALE as i32;
            canvas.fill_rect(Rect::new(dotx, doty, PX_SCALE as u32, PX_SCALE as u32))
                  .map_err(|s| Error::new(s))?;
        }
    }
    Ok(())
}

// Draw each pattern table in a 16 tile x 32 tile grid.
fn dump_table(tiles: &Vec<Tile>, table_id: usize, canvas: &mut WindowCanvas, pal: &[Color;4]) -> Result<(), Error> {
    let table = tiles.chunks(PATTAB_TILES).nth(table_id)
                .ok_or(Error::new("invalid pattern table".to_string()))?;
    let rightx = (16 * TILE_SIZE_PX * PX_SCALE) as i32;
    let mut y = 0;

    println!("dump pattern table {}", table_id);

    for row in 0..16 {
        let mut x = 0;
        for col in 0..16 {
            let tile = &table[row * 16 + col];
            draw_tile_at(tile, x, y, canvas, pal)?;
            let tile = &table[PATTAB_TILES / 2 + row * 16 + col];
            draw_tile_at(tile, x + rightx, y, canvas, pal)?;
            x += (TILE_SIZE_PX * PX_SCALE) as i32;
        }
        y += (TILE_SIZE_PX * PX_SCALE) as i32;
    }

    Ok(())
}


fn dump_pattern_table(file: &str) -> Result<(), Error> {
    let rom = Ines::from_file(file)?;
    let tiles = make_tiles(&rom);
    let win_height = (16 * TILE_SIZE_PX * PX_SCALE) as u32;
    let win_width = win_height * 2;

    // Since there is no attribute table, only the 2 bits
    // from the pattern table, we fix the colors used here.
    let greyscale = [
        Color::RGB(200, 200, 200),
        Color::RGB(0, 0, 0),
        Color::RGB(60, 60, 60),
        Color::RGB(120, 120, 120),
    ];

    let (mut canvas, mut event_pump) = init_window("pattern table", win_width, win_height)?;

    canvas.set_draw_color(Color::RGB(255, 255, 255));
    canvas.clear();

    let mut table_id = 0;
    dump_table(&tiles, table_id, &mut canvas, &greyscale)?;
    canvas.present();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } =>
                    break 'running,
                Event::KeyDown { keycode: Some(Keycode::Right), .. } |
                Event::KeyDown { keycode: Some(Keycode::Up), .. } => {
                    if table_id == rom.header.n_chrrom as usize - 1 {
                        table_id = 0;
                    }
                    else {
                        table_id += 1;
                    }
                    dump_table(&tiles, table_id, &mut canvas, &greyscale)?;
                    canvas.present();
                }
                Event::KeyDown { keycode: Some(Keycode::Left), .. } |
                Event::KeyDown { keycode: Some(Keycode::Down), .. } => {
                    if table_id == 0 {
                        table_id = rom.header.n_chrrom as usize - 1;
                    }
                    else {
                        table_id -= 1;
                    }

                    dump_table(&tiles, table_id, &mut canvas, &greyscale)?;
                    canvas.present();
                }
                _ => {}
            }
        }
    }

    Ok(())
}

fn init_window(title: &str, width: u32, height: u32) -> Result<(WindowCanvas, EventPump), Error> {
    let sdl_ctx = sdl2::init().map_err(|s| Error::new(s))?;
    let video = sdl_ctx.video().map_err(|s| Error::new(s))?;
    let window = video.window(title, width, height).position_centered().build()?;

    let canvas = window.into_canvas().build()?;
    let event_pump = sdl_ctx.event_pump().map_err(|s| Error::new(s))?;

    Ok((canvas, event_pump))
}

fn dump_palette(file: &str) -> Result<(), Error> {
    let side = TILE_SIZE_PX * PX_SCALE;
    let height = 4 * side as u32;
    let width = 16 * side as u32;

    let (mut canvas, mut event_pump) = init_window("palette", width, height)?;

    canvas.set_draw_color(Color::RGB(255, 255, 255));
    canvas.clear();

    let pals = palette::palette_from_file(file)?;

    for (r, pal) in pals.iter().enumerate() {
        for (c, color) in pal.iter().enumerate() {
            canvas.set_draw_color(*color);
            let x = (c * side) as i32;
            let y = (r * side) as i32;
            canvas.fill_rect(Rect::new(x, y, side as u32, side as u32))
                  .map_err(|s| Error::new(s))?;
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
        dump_palette(file)?;
        return Ok(());
    }

    if args.is_present("rom") {
        dump_pattern_table(file)?;
        return Ok(());
    }

    Ok(())
}

fn main() {
    match run() {
        Ok(()) => (),
        Err(e) => {
            eprintln!("error: {}", e);
            process::exit(1);
        }
    }
}
