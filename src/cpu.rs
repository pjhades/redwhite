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
    fn read(&mut self, cpu: &mut Cpu) -> u8;
    fn write(&mut self, cpu: &mut Cpu, at: Address, value: u8) {
        cpu.write(at, value);
    }
}
// accumulator addressing
struct Accumulator;

impl AddressingMode for Accumulator {
    fn read(&mut self, cpu: &mut Cpu) -> u8 {
        cpu.a
    }

    fn write(&mut self, cpu: &mut Cpu, _: Address, value: u8) {
        cpu.a = value;
    }
}

// immediate addressing
struct Immediate;

impl AddressingMode for Immediate {
    fn read(&mut self, cpu: &mut Cpu) -> u8 {
        cpu.fetch()
    }

    fn write(&mut self, _: &mut Cpu, _: Address, _: u8) {
        panic!("write is not allowed in this addressing mode");
    }
}

// zero page addressing
struct ZeroPage {
    at: Address,
}

impl AddressingMode for ZeroPage {
    fn read(&mut self, cpu: &mut Cpu) -> u8 {
        self.at = cpu.fetch() as Address;
        cpu.read(self.at)
    }
}

// zero page indexed addressing
struct ZeroPageIndexed {
    at: Address,
    index: u8,
}

impl AddressingMode for ZeroPageIndexed {
    fn read(&mut self, cpu: &mut Cpu) -> u8 {
        self.at = (cpu.fetch() as Address + self.index as Address) & 0x00ff;
        cpu.read(self.at)
    }
}

// absolute addressing
struct Absolute {
    at: Address,
}

impl AddressingMode for Absolute {
    fn read(&mut self, cpu: &mut Cpu) -> u8 {
        self.at = cpu.fetch_word();
        cpu.read(self.at)
    }
}

// absolute indexed addressing
struct AbsoluteIndexed {
    at: Address,
    index: u8,
    check_xpage: bool,
}

impl AddressingMode for AbsoluteIndexed {
    fn read(&mut self, cpu: &mut Cpu) -> u8 {
        let base = cpu.fetch_word();
        self.at = base.wrapping_add(self.index as Address);

        if self.check_xpage && does_x_page(base, self.at) {
            cpu.cycle_count += 1;
        }

        cpu.read(self.at)
    }
}

// indexed indirect addressing
struct IndexedIndirect {
    at: Address,
}

impl AddressingMode for IndexedIndirect {
    fn read(&mut self, cpu: &mut Cpu) -> u8 {
        let base = cpu.fetch();

        let lo = base.wrapping_add(cpu.x) as Address;
        let hi = (lo + 1) & 0x00ff;

        self.at = cpu.read(lo) as Address |
                  (cpu.read(hi) as Address) << 8;
        cpu.read(self.at)
    }
}

// indirect indexed addressing
struct IndirectIndexed {
    at: Address,
}

impl AddressingMode for IndirectIndexed {
    fn read(&mut self, cpu: &mut Cpu) -> u8 {
        let lo = cpu.fetch() as Address;
        let hi = (lo + 1) & 0x00ff;

        let base = cpu.read(lo) as Address |
                   (cpu.read(hi) as Address) << 8;
        self.at = base.wrapping_add(cpu.y as Address);

        if does_x_page(base, self.at) {
            cpu.cycle_count += 1;
        }

        cpu.read(self.at)
    }
}

// relative addressing
struct Relative {
    at: Address,
}

impl AddressingMode for Relative {
    fn read(&mut self, cpu: &mut Cpu) -> u8 {
        let offset = cpu.fetch() as i8 as i16;
        self.at = (cpu.pc as i16).wrapping_add(offset) as Address;
        0
    }

    fn write(&mut self, _: &mut Cpu, _: Address, _: u8) {
        panic!("write is not allowed in this addressing mode");
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
    fn set_flag(&mut self, flag: u8) {
        self.p |= flag;
    }

    #[inline(always)]
    fn set_flag_if(&mut self, flag: u8, condition: bool) {
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
        self.set_flag_if(FLAG_Z, value == 0);
        self.set_flag_if(FLAG_N, value & 0x80 != 0);
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

    #[inline(always)]
    fn push(&mut self, value: u8) {
        let at = self.sp as Address + 0x0100;
        self.write(at, value);
        self.sp = self.sp.wrapping_sub(1);
    }

    #[inline(always)]
    fn pop(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        self.read(self.sp as Address + 0x0100)
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

    fn adc(&mut self, operand: u8) -> u8 {
        let mut result = operand as u16 + self.a as u16;
        if self.is_flag_set(FLAG_C) {
            result += 1;
        }

        self.set_flag_if(FLAG_C, result > 0xff);

        let result = result as u8;
        self.set_zn(result);

        let a = self.a;
        self.set_flag_if(FLAG_V, (a ^ operand) & 0x80 == 0 && (a ^ result) & 0x80 != 0);

        self.a = result;
        result
    }

    fn and(&mut self, operand: u8) -> u8 {
        let result = operand & self.a;
        self.set_zn(result);
        self.a = result;
        result
    }

    fn asl(&mut self, operand: u8) -> u8 {
        self.set_flag_if(FLAG_C, operand & 0x80 != 0);
        let result = operand << 1;
        self.set_zn(result);
        result
    }

    fn bcc(&mut self, at: Address) {
        let cond = !self.is_flag_set(FLAG_C);
        self.jump_on_condition(at, cond);
    }

    fn bcs(&mut self, at: Address) {
        let cond = self.is_flag_set(FLAG_C);
        self.jump_on_condition(at, cond);
    }

    fn beq(&mut self, at: Address) {
        let cond = self.is_flag_set(FLAG_Z);
        self.jump_on_condition(at, cond);
    }

    fn bmi(&mut self, at: Address) {
        let cond = self.is_flag_set(FLAG_N);
        self.jump_on_condition(at, cond);
    }

    fn bne(&mut self, at: Address) {
        let cond = !self.is_flag_set(FLAG_Z);
        self.jump_on_condition(at, cond);
    }

    fn bpl(&mut self, at: Address) {
        let cond = !self.is_flag_set(FLAG_N);
        self.jump_on_condition(at, cond);
    }

    fn bvc(&mut self, at: Address) {
        let cond = !self.is_flag_set(FLAG_V);
        self.jump_on_condition(at, cond);
    }

    fn bvs(&mut self, at: Address) {
        let cond = self.is_flag_set(FLAG_V);
        self.jump_on_condition(at, cond);
    }

    fn bit(&mut self, operand: u8) {
        self.set_flag_if(FLAG_N, operand & 0x80 != 0);
        self.set_flag_if(FLAG_V, operand & 0x40 != 0);
        if operand & self.a == 0 {
            self.set_flag(FLAG_Z);
        }
        else {
            self.clear_flag(FLAG_Z);
        }
    }

    #[inline(always)]
    fn clc(&mut self, _: u8) {
        self.clear_flag(FLAG_C);
    }

    #[inline(always)]
    fn cld(&mut self, _: u8) {
        self.clear_flag(FLAG_D);
    }

    #[inline(always)]
    fn cli(&mut self, _: u8) {
        self.clear_flag(FLAG_I);
    }

    #[inline(always)]
    fn clv(&mut self, _: u8) {
        self.clear_flag(FLAG_V);
    }

    // cmp, cpx, cpy
    fn cmp_with_reg(&mut self, reg: u8, operand: u8) {
        let result = reg as i8 - operand as i8;
        self.set_flag_if(FLAG_C, result >= 0);
        self.set_zn(result as u8);
    }
}
