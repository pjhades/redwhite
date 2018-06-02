use mem::{Address, Memory, Access};

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
    sp: u8,
    pc: u16,
    p: Flags,

    cycle_count: usize,
}

// check if addresses `a1` and `a2` are on different pages
#[inline]
fn does_x_page(a1: Address, a2: Address) -> bool {
    (a1 >> 8) == (a2 >> 8)
}

// addressing modes
trait AddressingMode {
    fn read(&mut self, cpu: &mut Cpu, mem: &Memory) -> u8;
    fn write(&mut self, cpu: &mut Cpu, mem: &mut Memory, val: u8);
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

    fn write(&mut self, _cpu: &mut Cpu, _mem: &mut Memory, _val: u8) {
        panic!("write back in an instruction with immediate addressing");
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
            p: Flags::default(),

            cycle_count: 0,
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

    pub fn fetch(&mut self, mem: &Memory) -> u8 {
        let val = mem.read(self.pc);
        self.pc += 1;
        val
    }

    pub fn fetch_word(&mut self, mem: &Memory) -> u16 {
        let val = mem.read_word(self.pc);
        self.pc += 2;
        val
    }
}
