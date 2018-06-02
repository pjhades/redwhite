use std::ops::{Deref, DerefMut};

pub type Address = u16;

pub trait Access {
    fn read(&self, at: Address) -> u8;
    fn write(&mut self, at: Address, val: u8);

    fn read_word(&self, at: Address) -> u16 {
        self.read(at) as u16 |
        (self.read(at + 1) as u16) << 8
    }

    fn read_word_wrap(&self, at: Address) -> u16 {
        let wrap = at & 0xff00 | (at + 1) & 0x00ff;
        self.read(at) as u16 |
        (self.read(wrap) as u16) << 8
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
    fn read(&self, at: Address) -> u8 {
        self[at as usize & 0x07ff]
    }

    fn write(&mut self, at: Address, val: u8) {
        self[at as usize & 0x07ff] = val;
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
    fn read(&self, at: Address) -> u8 {
        if at < 0x2000 {
            self.ram.read(at)
        }
        else {
            panic!("reading other memory sections is not implemented yet!");
        }
    }

    fn write(&mut self, at: Address, val: u8) {
        if at < 0x2000 {
            self.ram.write(at, val)
        }
        else {
            panic!("writing other memory sections is not implemented yet!");
        }
    }
}
