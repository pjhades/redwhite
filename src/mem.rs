use std::ops::{Deref, DerefMut};

pub trait Access {
    // read a single byte
    fn read(&self, addr: u16) -> u8;

    // write a single byte
    fn write(&mut self, addr: u16, value: u8);

    // read 2 bytes starting from `addr`
    fn read16(&self, addr: u16) -> u16 {
        self.read(addr) as u16 | (self.read(addr + 1) as u16) << 8
    }

    // read 2 bytes but with lower address wrapped around
    // http://nesdev.com/6502_cpu.txt
    fn read16_wrapped(&self, addr: u16) -> u16 {
        let wrapped = addr & 0xff00 | (addr + 1) & 0x00ff;
        self.read(addr) as u16 | (self.read(wrapped) as u16) << 8
    }
}

// RAM
struct Ram {
    data: [u8; 0x800]
}

impl Deref for Ram {
    type Target = [u8; 0x800];

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for Ram {
    fn deref_mut(&mut self) -> &mut [u8; 0x800] {
        &mut self.data
    }
}

impl Access for Ram {
    fn read(&self, addr: u16) -> u8 {
        self[addr as usize & 0x07ff]
    }

    fn write(&mut self, addr: u16, value: u8) {
        self[addr as usize & 0x07ff] = value;
    }
}

// main memory
pub struct Memory {
    ram: Ram,
}

impl Memory {
    pub fn new() -> Self {
        Memory {
            ram: Ram { data: [0; 0x800] },
        }
    }
}

impl Access for Memory {
    fn read(&self, addr: u16) -> u8 {
        if addr < 0x2000 {
            self.ram.read(addr)
        }
        else {
            panic!("reading other memory sections is not implemented yet!");
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        if addr < 0x2000 {
            self.ram.write(addr, value)
        }
        else {
            panic!("writing other memory sections is not implemented yet!");
        }
    }
}
