bitflags! {
    #[derive(Default)]
    pub struct Flags: u8 {
        const N = 0b1000_0000;
        const V = 0b0100_0000;
        const X = 0b0010_0000;
        const B = 0b0001_0000;
        const D = 0b0000_1000;
        const I = 0b0000_0100;
        const Z = 0b0000_0010;
        const C = 0b0000_0001;
    }
}

pub struct Cpu {
    a: u8,
    x: u8,
    y: u8,
    sp: u16,
    pc: u16,
    p: Flags,
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            a: 0,
            x: 0,
            y: 0,
            sp: 0,
            pc: 0,
            p: Flags::default(),
        }
    }

    pub fn set_flag(&mut self, flag: Flags) {
        self.p.insert(flag);
    }

    pub fn clear_flag(&mut self, flag: Flags) {
        self.p.remove(flag);
    }

    pub fn is_flag_set(&self, flag: Flags) {
        self.p.contains(flag);
    }

    pub fn flags_as_byte(&self) -> u8 {
        self.p.bits()
    }
}
