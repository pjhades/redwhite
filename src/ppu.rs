use mem::Access;

const PPUCTRL:   u16 = 0x2000;
const PPUMASK:   u16 = 0x2001;
const PPUSTATUS: u16 = 0x2002;
const OAMADDR:   u16 = 0x2003;
const OAMDATA:   u16 = 0x2004;
const PPUSCROLL: u16 = 0x2005;
const PPUADDR:   u16 = 0x2006;
const PPUDATA:   u16 = 0x2007;
const OAMDMA:    u16 = 0x4014;

pub struct Ppu {
    vaddr: u16,         // current VRAM address
    taddr: u16,         // temporary VRAM address
    finex: u8,          // fine X scroll
    write_toggle: bool, // PPUSCROLL, PPUADDR write toggle
    // http://wiki.nesdev.com/w/index.php/PPU_registers
    regs: [u8; 8],      // registers
    data_bus: u8,        // internal data bus/latch
}

impl Ppu {
    pub fn new() -> Self {
        // PPU power-up state
        // http://wiki.nesdev.com/w/index.php/PPU_power_up_state
        Ppu {
            vaddr: 0,
            taddr: 0,
            finex: 0,
            write_toggle: false,
            regs: [0; 8],  // TODO are you sure?
            data_bus: 0,
        }
    }

    pub fn read_reg(&self, addr: u16) -> u8 {
        if addr == PPUSTATUS || addr == OAMDATA || addr == PPUDATA {
            self.data_bus = self.regs[(addr - 0x2000) as usize];
        }
        self.data_bus
    }

    pub fn write_reg(&mut self, addr: u16, value: u8) {
        self.data_db = value;
        self.regs[(addr - 0x2000) as usize] = value;
    }
}

//pub struct
//
//impl Access for Ppu {
//}
