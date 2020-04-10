use memory::{Access, PpuMem};

struct Regs {
    ppuctrl: u8,   // $2000, write
    ppumask: u8,   // $2001, write
    ppustatus: u8, // $2002, read
    oamaddr: u8,   // $2003, write
    oamdata: u8,   // $2004, read/write
    ppuscroll: u8, // $2005, write x2
    ppuaddr: u8,   // $2006, write x2
    ppudata: u8,   // $2007, read/write
    oamdma: u8,    // $4014, write
}

impl Regs {
    fn new() -> Self {
        Regs {
            ppuctrl: 0,
            ppumask: 0,
            ppustatus: 0,
            oamaddr: 0,
            oamdata: 0,
            ppuscroll: 0,
            ppuaddr: 0,
            ppudata: 0,
            oamdma: 0,
        }
    }

    #[inline(always)]
    fn base_nt(&self) -> u16 {
        0x2000 | ((self.ppuctrl as u16 & 0x3) << 10)
    }

    #[inline(always)]
    fn vram_inc(&self) -> u16 {
        if self.ppuctrl & 0x4 == 0 { 1 } else { 32 }
    }

    #[inline(always)]
    fn spr_pt(&self) -> u16 {
        self.ppuctrl as u16 & 0x8
    }

    #[inline(always)]
    fn bg_pt(&self) -> u16 {
        (self.ppuctrl as u16 & 0x10) >> 1
    }

    #[inline(always)]
    fn spr_8x8(&self) -> bool {
        self.ppuctrl & 0x20 == 0
    }

    #[inline(always)]
    fn slave(&self) -> bool {
        self.ppuctrl & 0x40 != 0
    }

    #[inline(always)]
    fn gen_nmi(&self) -> bool {
        self.ppuctrl & 0x80 != 0
    }

    #[inline(always)]
    fn greyscale(&self) -> bool {
        self.ppumask & 0x1 != 0
    }

    #[inline(always)]
    fn show_bg_left_8px(&self) -> bool {
        self.ppumask & 0x2 != 0
    }

    #[inline(always)]
    fn show_spr_left_8px(&self) -> bool {
        self.ppumask & 0x4 != 0
    }

    #[inline(always)]
    fn show_bg(&self) -> bool {
        self.ppumask & 0x8 != 0
    }

    #[inline(always)]
    fn show_spr(&self) -> bool {
        self.ppumask & 0x10 != 0
    }

    #[inline(always)]
    fn emphasize_r(&self) -> bool {
        self.ppumask & 0x20 != 0
    }

    #[inline(always)]
    fn emphasize_g(&self) -> bool {
        self.ppumask & 0x40 != 0
    }

    #[inline(always)]
    fn emphasize_b(&self) -> bool {
        self.ppumask & 0x80 != 0
    }

    #[inline(always)]
    fn spr_overflow(&self) -> bool {
        self.ppustatus & 0x20 != 0
    }

    #[inline(always)]
    fn spr_0hit(&self) -> bool {
        self.ppustatus & 0x40 != 0
    }

    #[inline(always)]
    fn vblank(&self) -> bool {
        self.ppustatus & 0x80 != 0
    }
}

pub struct Ppu {
    v: u16,        // current VRAM address
    t: u16,        // temporary VRAM address
    x: u8,         // fine X scroll
    w: bool,       // PPUSCROLL, PPUADDR write latch
    regs: Regs,
    mem: PpuMem,
    oam: [u8; 256],
    cycles: usize,
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
            regs: Regs::new(),
            mem: PpuMem::new(),
            oam: [0; 256],
            cycles: 0,
        }
    }

    pub fn read_register(&mut self, addr: u16) -> u8 {
        match addr {
            0x2002 => {
                let value = self.regs.ppustatus;
                self.w = false;
                self.regs.ppustatus &= 0x7f;
                value
            }
            0x2004 => self.oam[self.regs.oamaddr as usize],
            0x2007 => {
                let value = self.mem.read(self.v);
                self.v += self.regs.vram_inc();
                value
            }
            _ => 0,
        }
    }

    pub fn write_register(&mut self, addr: u16, value: u8) {
        match addr {
            0x2000 => self.regs.ppuctrl = value,
            0x2001 => self.regs.ppumask = value,
            0x2003 => self.regs.oamaddr = value,
            0x2004 => {
                self.oam[self.regs.oamaddr as usize] = value;
                self.regs.oamaddr += 1;
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
                self.v += self.regs.vram_inc();
            }
            _ => ()
        }
    }
}
