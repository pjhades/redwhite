use std::ops::{Deref, DerefMut};
use memory::{Mem, PpuMem};

#[derive(Default)]
struct PpuCtrl {
    value: u8
}

impl Deref for PpuCtrl {
    type Target = u8;

    fn deref(&self) -> &u8 {
        &self.value
    }
}

impl DerefMut for PpuCtrl {
    fn deref_mut(&mut self) -> &mut u8 {
        &mut self.value
    }
}

impl PpuCtrl {
    #[inline(always)]
    fn base_name_table(&self) -> u16 {
        match **self & 0x3 {
            0 => 0x2000,
            1 => 0x2400,
            2 => 0x2800,
            _ => 0x2c00,
        }
    }

    #[inline(always)]
    fn vram_increment(&self) -> u16 {
        if **self & 0x4 == 0 {
            1
        }
        else {
            32
        }
    }

    #[inline(always)]
    fn spr_pattern_table(&self) -> u16 {
        if **self & 0x8 == 0 {
            0x0000
        }
        else {
            0x1000
        }
    }

    #[inline(always)]
    fn bg_pattern_table(&self) -> u16 {
        if **self & 0x10 == 0 {
            0x0000
        }
        else {
            0x1000
        }
    }

    #[inline(always)]
    fn spr_8x16(&self) -> bool {
        **self & 0x20 != 0
    }

    #[inline(always)]
    fn slave(&self) -> bool {
        **self & 0x40 != 0
    }

    #[inline(always)]
    fn gen_vblank_nmi(&self) -> bool {
        **self & 0x80 != 0
    }
}

#[derive(Default)]
struct PpuStatus {
    pub spr_overflow: bool,
    pub spr_0_hit: bool,
    pub vblank: bool,
}

impl PpuStatus {
    fn as_byte(&self) -> u8 {
        (self.spr_overflow as u8) << 5 |
        (self.spr_0_hit as u8)    << 6 |
        (self.vblank as u8)       << 7
    }
}

//struct PpuMask {
//    pub greyscale: bool,
//    pub show_bg_left_8px: bool,
//    pub show_spr_left_8px: bool,
//    pub show_bg: bool,
//    pub show_spr: bool,
//    pub emphasize_r: bool,
//    pub emphasize_g: bool,
//    pub emphasize_b: bool,
//}
//
//impl PpuMask {
//    fn new(value: u8) -> Self {
//        PpuMask {
//            greyscale:         value & 0x01 != 0,
//            show_bg_left_8px:  value & 0x02 != 0,
//            show_spr_left_8px: value & 0x04 != 0,
//            show_bg:           value & 0x08 != 0,
//            show_spr:          value & 0x10 != 0,
//            emphasize_r:       value & 0x20 != 0,
//            emphasize_g:       value & 0x40 != 0,
//            emphasize_b:       value & 0x80 != 0,
//        }
//    }
//
//    fn as_byte(&self) -> u8 {
//        (self.greyscale as u8)                    |
//        (self.greyscshow_bg_left_8px as u8)  << 1 |
//        (self.greyscshow_spr_left_8px as u8) << 2 |
//        (self.greyscshow_bg as u8)           << 3 |
//        (self.greyscshow_spr as u8)          << 4 |
//        (self.greyscemphasize_r as u8)       << 5 |
//        (self.greyscemphasize_g as u8)       << 6 |
//        (self.greyscemphasize_b as u8)       << 7
//    }
//}

//#[derive(Copy, Clone, Default)]
//struct Sprite {
//    y: u8,
//    index: u8,
//    attr: u8,
//    x: u8,
//}
//
//impl Sprite {
//    #[inline(always)]
//    fn behind_bg(&self) -> bool {
//        self.attr & 0x20 != 0
//    }
//
//    #[inline(always)]
//    fn flip_hori(&self) -> bool {
//        self.attr & 0x40 != 0
//    }
//
//    #[inline(always)]
//    fn flip_vert(&self) -> bool {
//        self.attr & 0x80 != 0
//    }
//}

//struct Oam {
//    data: [Sprite; 64],
//}
//
//impl Oam {
//    fn new() -> Self {
//        Oam {
//            data: [Sprite::default(); 64]
//        }
//    }
//}
//
//impl Deref for Oam {
//    type Target = [Sprite; 64];
//
//    fn deref(&self) -> &Self::Target {
//        &self.data
//    }
//}

pub struct Ppu {
    v: u16,        // current VRAM address
    t: u16,        // temporary VRAM address
    x: u8,         // fine X scroll
    w: bool,       // PPUSCROLL, PPUADDR write latch
    mem: PpuMem,
    oam: [u8; 256],
    secondary_oam: [u8; 32],
    cycles: usize,
    nmi_occured: bool,
    nmi_output: bool,
    frame: usize,
    tick: usize,
    scanline: usize,
    ppuctrl: PpuCtrl,     // $2000, write
    ppumask: u8,          // $2001, write
    ppustatus: PpuStatus, // $2002, read
    oamaddr: u8,          // $2003, write
    oamdata: u8,          // $2004, read/write
    ppuscroll: u8,        // $2005, write x2
    ppuaddr: u8,          // $2006, write x2
    ppudata: u8,          // $2007, read/write
    oamdma: u8,           // $4014, write
}

impl Ppu {
    pub fn new() -> Self {
        // PPU power-up state
        // http://wiki.nesdev.com/w/index.php/PPU_power_up_state
        Ppu {
            v: 0,
            t: 0,
            x: 0,
            w: false,
            mem: PpuMem::new(),
            oam: [0; 256],
            secondary_oam: [0; 32],
            cycles: 0,
            nmi_occured: false,
            nmi_output: false,
            frame: 0,
            tick: 0,
            scanline: 261,
            ppuctrl: PpuCtrl::default(),
            ppumask: 0,
            ppustatus: PpuStatus::default(),
            oamaddr: 0,
            oamdata: 0,
            ppuscroll: 0,
            ppuaddr: 0,
            ppudata: 0,
            oamdma: 0,
        }
    }

    pub fn read_register(&mut self, addr: u16) -> u8 {
        match addr {
            0x2002 => {
                let value = (self.ppustatus.as_byte() & 0x7f) | ((self.nmi_occured as u8) << 7);
                self.nmi_occured = false;
                self.w = false;
                self.ppustatus.vblank = false;
                value
            }
            0x2004 => self.oam[self.oamaddr as usize],
            0x2007 => {
                let value = self.mem.read(self.v);
                self.v += self.ppuctrl.vram_increment();
                value
            }
            _ => 0,
        }
    }

    pub fn write_register(&mut self, addr: u16, value: u8) {
        match addr {
            0x2000 => {
                *self.ppuctrl = value;
                self.nmi_output = value & 0x80 != 0;
            }
            0x2001 => self.ppumask = value,
            0x2003 => self.oamaddr = value,
            0x2004 => {
                self.oam[self.oamaddr as usize] = value;
                self.oamaddr += 1;
            }
            0x2005 => {
                if !self.w {
                    // first write
                    self.t = ((value as u16) >> 3) | (self.t & 0xffe0);
                    self.x = value & 0x7;
                    self.w = true;
                }
                else {
                    // second write
                    self.t = (((value as u16) & 0x7) << 12) |
                             (((value as u16) & 0xf8) << 2) |
                             (self.t & 0x8c1f);
                    self.w = false;
                }
            }
            0x2006 => {
                if !self.w {
                    // first write
                    self.t = ((value as u16) & 0x3f) | (self.t & 0xc0ff);
                    self.t &= 0xbfff;
                    self.w = true;
                }
                else {
                    // second write
                    self.t = (value as u16) | (self.t & 0xff00);
                    self.v = self.t;
                    self.w = false;
                }
            }
            0x2007 => {
                self.mem.write(self.v, value);
                self.v += self.ppuctrl.vram_increment();
            }
            _ => ()
        }
    }

    #[inline(always)]
    pub fn nmi_assert_low(&self) -> bool {
        self.nmi_occured && self.nmi_output
    }

    fn run(&mut self, upto: usize) {
        let start = self.cycles;

        while self.cycles - start < upto {
            match self.scanline {
                261 => { // pre-render
                }

                0..=239 => { // visible
                }
                240 => { // post-render
                }
                241..=260 => { // vblank
                }
                _ => panic!("invalid scanline"),
            }
        }
    }
}
