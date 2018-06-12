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
    pc: Address,
    p: u8,
    cycle_count: usize,
    mem: Memory,
}

impl Access for Cpu {
    fn read(&self, at: Address) -> u8 {
        self.mem.read(at)
    }

    fn write(&mut self, at: Address, value: u8) {
        self.mem.write(at, value)
    }
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
    cpu: &'a mut Cpu,
}

impl<'a> AddressingMode for Immediate<'a> {
    fn read(&mut self) -> u8 {
        self.cpu.fetch()
    }
}

// zero page addressing
struct ZeroPage<'a> {
    at: Address,
    cpu: &'a mut Cpu,
}

impl<'a> AddressingMode for ZeroPage<'a> {
    fn read(&mut self) -> u8 {
        self.at = self.cpu.fetch() as Address;
        self.cpu.read(self.at)
    }

    fn write(&mut self, value: u8) {
        self.cpu.write(self.at, value);
    }
}

// zero page indexed addressing
struct ZeroPageIndexed<'a> {
    at: Address,
    index: u8,
    cpu: &'a mut Cpu,
}

impl<'a> AddressingMode for ZeroPageIndexed<'a> {
    fn read(&mut self) -> u8 {
        self.at = (self.cpu.fetch() as Address + self.index as Address) & 0x00ff;
        self.cpu.read(self.at)
    }

    fn write(&mut self, value: u8) {
        self.cpu.write(self.at, value);
    }
}

// absolute addressing
struct Absolute<'a> {
    at: Address,
    cpu: &'a mut Cpu,
}

impl<'a> AddressingMode for Absolute<'a> {
    fn read(&mut self) -> u8 {
        self.at = self.cpu.fetch_word();
        self.cpu.read(self.at)
    }

    fn write(&mut self, value: u8) {
        self.cpu.write(self.at, value);
    }
}

// absolute indexed addressing
struct AbsoluteIndexed<'a> {
    at: Address,
    index: u8,
    check_xpage: bool,
    cpu: &'a mut Cpu,
}

impl<'a> AddressingMode for AbsoluteIndexed<'a> {
    fn read(&mut self) -> u8 {
        let base = self.cpu.fetch_word();
        self.at = base.wrapping_add(self.index as Address);

        if self.check_xpage && does_x_page(base, self.at) {
            self.cpu.cycle_count += 1;
        }

        self.cpu.read(self.at)
    }

    fn write(&mut self, value: u8) {
        self.cpu.write(self.at, value);
    }
}

// indexed indirect addressing
struct IndexedIndirect<'a> {
    at: Address,
    cpu: &'a mut Cpu,
}

impl<'a> AddressingMode for IndexedIndirect<'a> {
    fn read(&mut self) -> u8 {
        let base = self.cpu.fetch();

        let lo = base.wrapping_add(self.cpu.x) as Address;
        let hi = (lo + 1) & 0x00ff;

        self.at = self.cpu.read(lo) as Address |
                  (self.cpu.read(hi) as Address) << 8;
        self.cpu.read(self.at)
    }

    fn write(&mut self, value: u8) {
        self.cpu.write(self.at, value);
    }
}

// indirect indexed addressing
struct IndirectIndexed<'a> {
    at: Address,
    cpu: &'a mut Cpu,
}

impl<'a> AddressingMode for IndirectIndexed<'a> {
    fn read(&mut self) -> u8 {
        let lo = self.cpu.fetch() as Address;
        let hi = (lo + 1) & 0x00ff;

        let base = self.cpu.read(lo) as Address |
                   (self.cpu.read(hi) as Address) << 8;
        self.at = base.wrapping_add(self.cpu.y as Address);

        if does_x_page(base, self.at) {
            self.cpu.cycle_count += 1;
        }

        self.cpu.read(self.at)
    }

    fn write(&mut self, value: u8) {
        self.cpu.write(self.at, value);
    }
}

// relative addressing
struct Relative<'a> {
    at: Address,
    cpu: &'a mut Cpu,
}

impl<'a> AddressingMode for Relative<'a> {
    fn read(&mut self) -> u8 {
        let offset = self.cpu.fetch() as i8 as i16;
        self.at = (self.cpu.pc as i16).wrapping_add(offset) as Address;
        0
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
            mem: Memory::new(),
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
    fn fetch(&mut self) -> u8 {
        let value = self.read(self.pc);
        self.pc += 1;
        value
    }

    #[inline(always)]
    fn fetch_word(&mut self) -> u16 {
        let value = self.read_word(self.pc);
        self.pc += 2;
        value
    }

    fn jump_on_condition(&mut self, at: Address, condition: bool) {
        if condition {
            self.cycle_count += 1;
            if does_x_page(self.pc, at) {
                self.cycle_count += 1;
            }
            self.pc = at;
        }
    }

    fn adc<M: AddressingMode>(&mut self, mut mode: M) -> u8 {
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
        result
    }

    fn and<M: AddressingMode>(&mut self, mut mode: M) -> u8 {
        let operand = mode.read();
        let result = operand & self.a;
        self.set_zn(result);
        self.a = result;
        result
    }

    fn asl<M: AddressingMode>(&mut self, mut mode: M) -> u8 {
        let operand = mode.read();
        self.set_flag(FLAG_C, operand & 0x80 != 0);
        let result = operand << 1;
        self.set_zn(result);
        result
    }

    fn bcc(&mut self, mut mode: Relative) {
        mode.read();
        let condition = !self.is_flag_set(FLAG_C);
        self.jump_on_condition(mode.at, condition);
    }
}
