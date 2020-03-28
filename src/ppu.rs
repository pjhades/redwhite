use mem::Access;

pub struct Ppu {
    vaddr: u16, // current VRAM address
    taddr: u16, // temporary VRAM address
    finex: u8, // fine X scroll
    write_toggle: bool, // PPUSCROLL, PPUADDR write toggle
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
        }
    }

    pub fn read_reg(&self, addr: u16) -> u8 {
        0x23
    }

    pub fn write_reg(&mut self, addr: u16, value: u8) {
    }
}

//pub struct
//
//impl Access for Ppu {
//}
