use crate::ppu::Ppu;

pub trait Mem {
    fn read(&mut self, addr: u16) -> u8;

    fn write(&mut self, addr: u16, value: u8);

    fn read16(&mut self, addr: u16) -> u16 {
        let lo = self.read(addr) as u16;
        let hi = self.read(addr + 1) as u16;
        hi << 8 | lo
    }

    fn read16_wrapped(&mut self, addr: u16) -> u16 {
        let wrapped = addr & 0xff00 | (addr + 1) & 0x00ff;
        let lo = self.read(addr) as u16;
        let hi = self.read(wrapped) as u16;
        hi << 8 | lo
    }
}

pub struct CpuMem {
    ram: [u8; 0x800],
    ppu: Ppu,
}

impl CpuMem {
    pub fn new() -> Self {
        CpuMem {
            ram: [0; 0x800],
            ppu: Ppu::new(),
        }
    }
}

impl Mem for CpuMem {
    fn read(&mut self, addr: u16) -> u8 {
        if addr < 0x2000 {
            self.ram[(addr % 0x800) as usize]
        }
        else if addr < 0x4000 {
            self.ppu.read_register(addr & 0x2007)
        }
        else {
            panic!("reading other CPU memory sections is not implemented yet!");
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        if addr < 0x2000 {
            self.ram[(addr % 0x800) as usize] = value;
        }
        else if addr < 0x4000 {
            self.ppu.write_register(addr & 0x2007, value);
        }
        else {
            panic!("writing other CPU memory sections is not implemented yet!");
        }
    }
}

pub struct PpuMem {
    pt: [u8; 0x2000],
    nt: [u8; 0x1000],
    palette: [u8; 0x20],
}

impl PpuMem {
    pub fn new() -> Self {
        PpuMem {
            pt: [0; 0x2000],
            nt: [0; 0x1000],
            palette: [0; 0x20],
        }
    }
}

impl Mem for PpuMem {
    fn read(&mut self, addr: u16) -> u8 {
        if addr < 0x2000 {
            self.pt[addr as usize]
        }
        else if addr < 0x3f00 {
            self.nt[(addr % 0x1000) as usize]
        }
        else if addr < 0x4000 {
            self.palette[(addr % 0x20) as usize]
        }
        else {
            panic!("reading other PPU memory sections is not implemented yet!");
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        if addr < 0x2000 {
            self.pt[addr as usize] = value;
        }
        else if addr < 0x3f00 {
            self.nt[(addr % 0x1000) as usize] = value;
        }
        else if addr < 0x4000 {
            self.palette[(addr % 0x20) as usize] = value;
        }
        else {
            panic!("reading other PPU memory sections is not implemented yet!");
        }
    }
}
