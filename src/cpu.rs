use mem::{Address, Memory, Access};

const FLAG_N: u8 = 0b1000_0000;
const FLAG_V: u8 = 0b0100_0000;
const FLAG_X: u8 = 0b0010_0000;
const FLAG_B: u8 = 0b0001_0000;
const FLAG_D: u8 = 0b0000_1000;
const FLAG_I: u8 = 0b0000_0100;
const FLAG_Z: u8 = 0b0000_0010;
const FLAG_C: u8 = 0b0000_0001;

pub struct Cpu {
    a: u8,
    x: u8,
    y: u8,
    sp: u8,
    pc: u16,
    p: u8,

    cycle_count: usize,
}

// check if addresses `a1` and `a2` are on different pages
#[inline(always)]
fn does_x_page(a1: Address, a2: Address) -> bool {
    (a1 >> 8) == (a2 >> 8)
}

// addressing modes
trait AddressingMode {
    fn read(&mut self) -> u8;
    fn write(&mut self, _: u8) {
        panic!("write is not allowed in this addressing mode");
    }
}

// accumulator addressing
struct Accumulator<'a> {
    cpu: &'a mut Cpu,
}

impl<'a> AddressingMode for Accumulator<'a> {
    fn read(&mut self) -> u8 {
        self.cpu.a
    }

    fn write(&mut self, value: u8) {
        self.cpu.a = value;
    }
}

// immediate addressing
struct Immediate<'a> {
    mem: &'a Memory,
    cpu: &'a mut Cpu,
}

impl<'a> AddressingMode for Immediate<'a> {
    fn read(&mut self) -> u8 {
        self.cpu.fetch(self.mem)
    }
}

// zero page addressing
struct ZeroPage<'a> {
    at: Address,
    mem: &'a mut Memory,
    cpu: &'a mut Cpu,
}

impl<'a> AddressingMode for ZeroPage<'a> {
    fn read(&mut self) -> u8 {
        self.at = self.cpu.fetch(self.mem) as Address;
        self.mem.read(self.at)
    }

    fn write(&mut self, value: u8) {
        self.mem.write(self.at, value);
    }
}

// zero page indexed addressing
struct ZeroPageIndexed<'a> {
    at: Address,
    index: u8,
    mem: &'a mut Memory,
    cpu: &'a mut Cpu,
}

impl<'a> AddressingMode for ZeroPageIndexed<'a> {
    fn read(&mut self) -> u8 {
        self.at = (self.cpu.fetch(self.mem) as Address + self.index as Address) & 0x00ff;
        self.mem.read(self.at)
    }

    fn write(&mut self, value: u8) {
        self.mem.write(self.at, value);
    }
}

// absolute addressing
struct Absolute<'a> {
    at: Address,
    mem: &'a mut Memory,
    cpu: &'a mut Cpu,
}

impl<'a> AddressingMode for Absolute<'a> {
    fn read(&mut self) -> u8 {
        self.at = self.cpu.fetch_word(self.mem);
        self.mem.read(self.at)
    }

    fn write(&mut self, value: u8) {
        self.mem.write(self.at, value);
    }
}

// absolute indexed addressing
struct AbsoluteIndexed<'a> {
    at: Address,
    index: u8,
    check_xpage: bool,
    mem: &'a mut Memory,
    cpu: &'a mut Cpu,
}

impl<'a> AddressingMode for AbsoluteIndexed<'a> {
    fn read(&mut self) -> u8 {
        let base = self.cpu.fetch_word(self.mem);
        self.at = base.wrapping_add(self.index as Address);

        if self.check_xpage && does_x_page(base, self.at) {
            self.cpu.cycle_count += 1;
        }

        self.mem.read(self.at)
    }

    fn write(&mut self, value: u8) {
        self.mem.write(self.at, value);
    }
}

// indexed indirect addressing
struct IndexedIndirect<'a> {
    at: Address,
    mem: &'a mut Memory,
    cpu: &'a mut Cpu,
}

impl<'a> AddressingMode for IndexedIndirect<'a> {
    fn read(&mut self) -> u8 {
        let base = self.cpu.fetch(self.mem);

        let lo = base.wrapping_add(self.cpu.x) as Address;
        let hi = (lo + 1) & 0x00ff;

        self.at = self.mem.read(lo) as Address |
                  (self.mem.read(hi) as Address) << 8;
        self.mem.read(self.at)
    }

    fn write(&mut self, value: u8) {
        self.mem.write(self.at, value);
    }
}

// indirect indexed addressing
struct IndirectIndexed<'a> {
    at: Address,
    mem: &'a mut Memory,
    cpu: &'a mut Cpu,
}

impl<'a> AddressingMode for IndirectIndexed<'a> {
    fn read(&mut self) -> u8 {
        let lo = self.cpu.fetch(self.mem) as Address;
        let hi = (lo + 1) & 0x00ff;

        let base = self.mem.read(lo) as Address |
                   (self.mem.read(hi) as Address) << 8;
        self.at = base.wrapping_add(self.cpu.y as Address);

        if does_x_page(base, self.at) {
            self.cpu.cycle_count += 1;
        }

        self.mem.read(self.at)
    }

    fn write(&mut self, value: u8) {
        self.mem.write(self.at, value);
    }
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            a: 0,
            x: 0,
            y: 0,
            sp: 0,
            pc: 0,
            p: 0,

            cycle_count: 0,
        }
    }

    #[inline(always)]
    fn set_flag(&mut self, flag: u8, condition: bool) {
        if condition {
            self.p |= flag; 
        }
    }

    #[inline(always)]
    fn clear_flag(&mut self, flag: u8) {
        self.p &= !flag;
    }

    #[inline(always)]
    fn is_flag_set(&self, flag: u8) -> bool {
        self.p & flag != 0
    }

    #[inline(always)]
    fn set_zn(&mut self, value: u8) {
        self.set_flag(FLAG_Z, value == 0);
        self.set_flag(FLAG_N, value & 0x80 != 0);
    }

    #[inline(always)]
    fn fetch(&mut self, mem: &Memory) -> u8 {
        let value = mem.read(self.pc);
        self.pc += 1;
        value
    }

    #[inline(always)]
    fn fetch_word(&mut self, mem: &Memory) -> u16 {
        let value = mem.read_word(self.pc);
        self.pc += 2;
        value
    }

    fn adc(&mut self, mode: &mut AddressingMode) {
        let operand = mode.read();
        let mut result = operand as u16 + self.a as u16;
        if self.is_flag_set(FLAG_C) {
            result += 1;
        }

        self.set_flag(FLAG_C, result > 0xff);

        let result = result as u8;
        self.set_zn(result);

        let a = self.a;
        self.set_flag(FLAG_V, (a ^ operand) & 0x80 == 0 && (a ^ result) & 0x80 != 0);

        self.a = result;
    }
}
