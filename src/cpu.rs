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
    fn read(&mut self, cpu: &mut Cpu, mem: &Memory) -> u8;
    fn write(&mut self, _cpu: &mut Cpu, _mem: &mut Memory, _val: u8) {
        panic!("write back in an instruction with immediate addressing");
    }
}

// accumulator addressing
struct Accumulator;

impl AddressingMode for Accumulator {
    fn read(&mut self, cpu: &mut Cpu, mem: &Memory) -> u8 {
        cpu.a
    }

    fn write(&mut self, cpu: &mut Cpu, mem: &mut Memory, val: u8) {
        cpu.a = val;
    }
}

// immediate addressing
struct Immediate;

impl AddressingMode for Immediate {
    fn read(&mut self, cpu: &mut Cpu, mem: &Memory) -> u8 {
        cpu.fetch(mem)
    }
}

// zero page addressing
struct ZeroPage {
    at: Address
}

impl AddressingMode for ZeroPage {
    fn read(&mut self, cpu: &mut Cpu, mem: &Memory) -> u8 {
        self.at = cpu.fetch(mem) as Address;
        mem.read(self.at)
    }

    fn write(&mut self, _cpu: &mut Cpu, mem: &mut Memory, val: u8) {
        mem.write(self.at, val);
    }
}

// zero page indexed addressing
struct ZeroPageIndexed {
    at: Address,
    index: u8,
}

impl AddressingMode for ZeroPageIndexed {
    fn read(&mut self, cpu: &mut Cpu, mem: &Memory) -> u8 {
        self.at = (cpu.fetch(mem) as Address + self.index as Address) & 0x00ff;
        mem.read(self.at)
    }

    fn write(&mut self, _cpu: &mut Cpu, mem: &mut Memory, val: u8) {
        mem.write(self.at, val);
    }
}

// absolute addressing
struct Absolute {
    at: Address
}

impl AddressingMode for Absolute {
    fn read(&mut self, cpu: &mut Cpu, mem: &Memory) -> u8 {
        self.at = cpu.fetch_word(mem);
        mem.read(self.at)
    }

    fn write(&mut self, _cpu: &mut Cpu, mem: &mut Memory, val: u8) {
        mem.write(self.at, val);
    }
}

// absolute indexed addressing
struct AbsoluteIndexed {
    at: Address,
    index: u8,
    check_xpage: bool,
}

impl AddressingMode for AbsoluteIndexed {
    fn read(&mut self, cpu: &mut Cpu, mem: &Memory) -> u8 {
        let base = cpu.fetch_word(mem);
        self.at = base.wrapping_add(self.index as Address);
        if self.check_xpage && does_x_page(base, self.at) {
            cpu.cycle_count += 1;
        }
        mem.read(self.at)
    }

    fn write(&mut self, _cpu: &mut Cpu, mem: &mut Memory, val: u8) {
        mem.write(self.at, val);
    }
}

// indexed indirect addressing
struct IndexedIndirect {
    at: Address
}

impl AddressingMode for IndexedIndirect {
    fn read(&mut self, cpu: &mut Cpu, mem: &Memory) -> u8 {
        let base = cpu.fetch(mem);
        let lo = base.wrapping_add(cpu.x) as Address;
        let hi = (lo + 1) & 0x00ff;
        self.at = mem.read(lo) as Address |
                  (mem.read(hi) as Address) << 8;
        mem.read(self.at)
    }

    fn write(&mut self, _cpu: &mut Cpu, mem: &mut Memory, val: u8) {
        mem.write(self.at, val);
    }
}

// indirect indexed addressing
struct IndirectIndexed {
    at: Address
}

impl AddressingMode for IndirectIndexed {
    fn read(&mut self, cpu: &mut Cpu, mem: &Memory) -> u8 {
        let lo = cpu.fetch(mem) as Address;
        let hi = (lo + 1) & 0x00ff;
        let base = mem.read(lo) as Address |
                   (mem.read(hi) as Address) << 8;
        self.at = base.wrapping_add(cpu.y as Address);
        if does_x_page(base, self.at) {
            cpu.cycle_count += 1;
        }
        mem.read(self.at)
    }

    fn write(&mut self, _cpu: &mut Cpu, mem: &mut Memory, val: u8) {
        mem.write(self.at, val);
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
    fn fetch(&mut self, mem: &Memory) -> u8 {
        let val = mem.read(self.pc);
        self.pc += 1;
        val
    }

    #[inline(always)]
    fn fetch_word(&mut self, mem: &Memory) -> u16 {
        let val = mem.read_word(self.pc);
        self.pc += 2;
        val
    }
}
