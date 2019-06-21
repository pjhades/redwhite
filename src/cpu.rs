use mem::{Memory, Access};

const CYCLES: [usize;256] = [
    //       0  1  2  3  4  5  6  7  8  9  a  b  c  d  e  f
    /* 0 */  7, 6, 0, 0, 0, 3, 5, 0, 3, 2, 2, 0, 0, 4, 6, 0,
    /* 1 */  2, 5, 0, 0, 0, 4, 6, 0, 2, 4, 0, 0, 0, 4, 7, 0,
    /* 2 */  6, 6, 0, 0, 3, 3, 5, 0, 4, 2, 2, 0, 4, 4, 6, 0,
    /* 3 */  2, 5, 0, 0, 0, 4, 6, 0, 2, 4, 0, 0, 0, 4, 7, 0,
    /* 4 */  6, 6, 0, 0, 0, 3, 5, 0, 3, 2, 2, 0, 3, 4, 6, 0,
    /* 5 */  2, 5, 0, 0, 0, 4, 6, 0, 2, 4, 0, 0, 0, 4, 7, 0,
    /* 6 */  6, 6, 0, 0, 0, 3, 5, 0, 4, 2, 2, 0, 5, 4, 6, 0,
    /* 7 */  2, 5, 0, 0, 0, 4, 6, 0, 2, 4, 0, 0, 0, 4, 7, 0,
    /* 8 */  0, 6, 0, 0, 3, 3, 3, 0, 2, 0, 2, 0, 4, 4, 4, 0,
    /* 9 */  2, 6, 0, 0, 4, 4, 4, 0, 2, 5, 2, 0, 0, 5, 0, 0,
    /* a */  2, 6, 2, 0, 3, 3, 3, 0, 2, 2, 2, 0, 4, 4, 4, 0,
    /* b */  2, 5, 0, 0, 4, 4, 4, 0, 2, 4, 2, 0, 4, 4, 4, 0,
    /* c */  2, 6, 0, 0, 3, 3, 5, 0, 2, 2, 2, 0, 4, 4, 6, 0,
    /* d */  2, 5, 0, 0, 0, 4, 6, 0, 2, 4, 0, 0, 0, 4, 7, 0,
    /* e */  2, 6, 0, 0, 3, 3, 5, 0, 2, 2, 2, 0, 4, 4, 6, 0,
    /* f */  2, 5, 0, 0, 0, 4, 6, 0, 2, 4, 0, 0, 0, 4, 7, 0,
];

const XPAGE_CYCLES: [usize;256] = [
    //       0  1  2  3  4  5  6  7  8  9  a  b  c  d  e  f
    /* 0 */  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    /* 1 */  1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0,
    /* 2 */  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    /* 3 */  1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0,
    /* 4 */  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    /* 5 */  1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0,
    /* 6 */  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    /* 7 */  1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0,
    /* 8 */  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    /* 9 */  1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    /* a */  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    /* b */  1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 1, 1, 0,
    /* c */  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    /* d */  1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0,
    /* e */  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    /* f */  1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0,
];

// status flags
const NEGATIVE:  u8 = 0b1000_0000;
const OVERFLOW:  u8 = 0b0100_0000;
const XFLAG:     u8 = 0b0010_0000;
const BREAK:     u8 = 0b0001_0000;
const DECIMAL:   u8 = 0b0000_1000;
const INTERRUPT: u8 = 0b0000_0100;
const ZERO:      u8 = 0b0000_0010;
const CARRY:     u8 = 0b0000_0001;

pub struct Cpu {
    a:  u8,
    x:  u8,
    y:  u8,
    sp: u8,
    p:  u8,
    pc: u16,
    mem: Memory,
    cycles: usize,
    check_xpage: bool,
}

impl Access for Cpu {
    fn read(&self, addr: u16) -> u8 {
        self.mem.read(addr)
    }

    fn write(&mut self, addr: u16, value: u8) {
        self.mem.write(addr, value)
    }
}

trait Addressing {
    fn address(&self, cpu: &mut Cpu) -> u8;
    fn writeback(&self, _cpu: &mut Cpu, _value: u8) {}
}

struct Immediate;
struct Accumulator;
struct FromMemory { addr: u16 }

impl Addressing for Immediate {
    fn address(&self, cpu: &mut Cpu) -> u8 {
        cpu.read_at_pc()
    }
}

impl Addressing for Accumulator {
    fn address(&self, cpu: &mut Cpu) -> u8 {
        cpu.a
    }

    fn writeback(&self, cpu: &mut Cpu, value: u8) {
        cpu.a = value;
    }
}

impl Addressing for FromMemory {
    fn address(&self, cpu: &mut Cpu) -> u8 {
        cpu.read(self.addr)
    }

    fn writeback(&self, cpu: &mut Cpu, value: u8) {
        cpu.write(self.addr, value);
    }
}

macro_rules! I {
    ($cpu:ident, $inst_fn:ident, $mode_fn:ident) => {{
        let mode = $cpu.$mode_fn();
        $cpu.$inst_fn(mode);
    }}
}

impl Cpu {
    pub fn new() -> Self {
        // Set power-up state
        // Ref: https://wiki.nesdev.com/w/index.php/CPU_power_up_state
        Cpu {
            a:  0,
            x:  0,
            y:  0,
            sp: 0xfd,
            pc: 0,
            p:  0x34,
            mem: Memory::new(),
            cycles: 0,
            check_xpage: false,
        }
    }

    #[inline(always)]
    fn set_flag(&mut self, flag: u8) {
        self.p |= flag;
    }

    #[inline(always)]
    fn clear_flag(&mut self, flag: u8) {
        self.p &= !flag;
    }

    #[inline(always)]
    fn update_flag(&mut self, flag: u8, cond: bool) {
        self.set_flag(if cond { flag } else { !flag });
    }

    #[inline(always)]
    fn update_zero_negative(&mut self, value: u8) {
        self.update_flag(ZERO, value == 0);
        self.update_flag(NEGATIVE, value & 0x80 != 0);
    }

    #[inline(always)]
    fn flag_on(&self, flag: u8) -> bool {
        self.p & flag != 0
    }

    fn read_at_pc(&mut self) -> u8 {
        let value = self.read(self.pc);
        self.pc += 1;
        value
    }

    fn read16_at_pc(&mut self) -> u16 {
        let value = self.read16(self.pc);
        self.pc += 2;
        value
    }

    fn push(&mut self, value: u8) {
        let addr = self.sp as u16 + 0x0100;
        self.write(addr, value);
        self.sp -= 1;
    }

    fn push16(&mut self, value: u16) {
        self.push(((value & 0xff00) >> 8) as u8);
        self.push(value as u8);
    }

    fn pop(&mut self) -> u8 {
        self.sp += 1;
        self.read(self.sp as u16 + 0x0100)
    }

    fn pop16(&mut self) -> u16 {
        let value = self.pop() as u16;
        let value = value | (self.pop() as u16) << 8;
        value
    }

    fn page_crossed(&self, addr1: u16, addr2: u16) -> bool {
        addr1 & 0xff00 != addr2 & 0xff00
    }

    fn jump_on_cond(&mut self, addr: u16, condition: bool) {
        if condition {
            // +1 cycle if branch is taken
            self.cycles += 1;
            // +1 cycle if branching across page boundary
            if self.page_crossed(self.pc, addr) {
                self.cycles += 1;
            }
            self.pc = addr;
        }
    }

    // addressing modes: http://obelisk.me.uk/6502/addressing.html
    fn immediate(&self) -> Immediate {
        Immediate {}
    }

    fn accumulator(&self) -> Accumulator {
        Accumulator {}
    }

    fn zeropage(&mut self) -> FromMemory {
        FromMemory { addr: self.read_at_pc() as u16 }
    }

    fn zeropage_x(&mut self) -> FromMemory {
        FromMemory { addr: self.read_at_pc().wrapping_add(self.x) as u16 }
    }

    fn zeropage_y(&mut self) -> FromMemory {
        FromMemory { addr: self.read_at_pc().wrapping_add(self.y) as u16 }
    }

    fn absolute(&mut self) -> FromMemory {
        FromMemory { addr: self.read16_at_pc() }
    }

    fn absolute_x(&mut self) -> FromMemory {
        let pc = self.pc;
        let addr = self.read16_at_pc() + self.x as u16;
        self.check_xpage = self.page_crossed(pc, addr);
        FromMemory { addr }
    }

    fn absolute_y(&mut self) -> FromMemory {
        let pc = self.pc;
        let addr = self.read16_at_pc() + self.y as u16;
        self.check_xpage = self.page_crossed(pc, addr);
        FromMemory { addr }
    }

    fn indirect(&mut self) -> FromMemory {
        let a = self.read16_at_pc();
        FromMemory { addr: self.read16_wrapped(a) }
    }

    fn indexed_indirect(&mut self) -> FromMemory {
        let a = self.read_at_pc().wrapping_add(self.x);
        FromMemory { addr: self.read16_wrapped(a as u16) }
    }

    fn indirect_indexed(&mut self) -> FromMemory {
        let a = self.read_at_pc();
        let v = self.read16_wrapped(a as u16);
        let addr = v + self.y as u16;
        self.check_xpage = self.page_crossed(v, addr);
        FromMemory { addr }
    }

    fn relative(&mut self) -> FromMemory {
        let offset = self.read_at_pc();
        FromMemory { addr: ((self.pc as i16) + (offset as i8 as i16)) as u16 }
    }

    // instructions
    fn adc<T: Addressing>(&mut self, mode: T) {
        let operand = mode.address(self);
        let mut sum = operand as u16 +
                      self.a as u16 +
                      if self.flag_on(CARRY) { 1 } else { 0 };
        self.update_flag(CARRY, sum > 0xff);
        let result = sum as u8;
        self.update_zero_negative(result);
        let a = self.a;
        let cond = (a ^ operand) & 0x80 == 0 && (a ^ result) & 0x80 != 0;
        self.update_flag(OVERFLOW, cond);
        self.a = result;
    }

    fn and<T: Addressing>(&mut self, mode: T) {
        let operand = mode.address(self);
        let result = operand & self.a;
        self.update_zero_negative(result);
        self.a = result;
    }

    fn asl<T: Addressing>(&mut self, mode: T) {
        let operand = mode.address(self);
        self.update_flag(CARRY, operand & 0x80 != 0);
        let result = operand << 1;
        self.update_zero_negative(result);
        mode.writeback(self, result);
    }

    fn bcc(&mut self, mode: FromMemory) {
        let cond = !self.flag_on(CARRY);
        self.jump_on_cond(mode.addr, cond);
    }

    fn bcs(&mut self, mode: FromMemory) {
        let cond = self.flag_on(CARRY);
        self.jump_on_cond(mode.addr, cond);
    }

    fn beq(&mut self, mode: FromMemory) {
        let cond = self.flag_on(ZERO);
        self.jump_on_cond(mode.addr, cond);
    }

    fn bmi(&mut self, mode: FromMemory) {
        let cond = self.flag_on(NEGATIVE);
        self.jump_on_cond(mode.addr, cond);
    }

    fn bne(&mut self, mode: FromMemory) {
        let cond = !self.flag_on(ZERO);
        self.jump_on_cond(mode.addr, cond);
    }

    fn bpl(&mut self, mode: FromMemory) {
        let cond = !self.flag_on(NEGATIVE);
        self.jump_on_cond(mode.addr, cond);
    }

    fn bvc(&mut self, mode: FromMemory) {
        let cond = !self.flag_on(OVERFLOW);
        self.jump_on_cond(mode.addr, cond);
    }

    fn bvs(&mut self, mode: FromMemory) {
        let cond = self.flag_on(OVERFLOW);
        self.jump_on_cond(mode.addr, cond);
    }

    fn bit<T: Addressing>(&mut self, mode: T) {
        let operand = mode.address(self);
        self.update_flag(NEGATIVE, operand & 0x80 != 0);
        self.update_flag(OVERFLOW, operand & 0x40 != 0);
        let cond = operand & self.a == 0;
        self.update_flag(ZERO, cond);
    }

    fn cmp<T: Addressing>(&mut self, mode: T) {
        let operand = mode.address(self);
        let result = self.a as i8 - operand as i8;
        self.update_flag(CARRY, result >= 0);
        self.update_zero_negative(result as u8);
    }

    fn cpx<T: Addressing>(&mut self, mode: T) {
        let operand = mode.address(self);
        let result = self.x as i8 - operand as i8;
        self.update_flag(CARRY, result >= 0);
        self.update_zero_negative(result as u8);
    }

    fn cpy<T: Addressing>(&mut self, mode: T) {
        let operand = mode.address(self);
        let result = self.y as i8 - operand as i8;
        self.update_flag(CARRY, result >= 0);
        self.update_zero_negative(result as u8);
    }

    fn dec<T: Addressing>(&mut self, mode: T) {
        let operand = mode.address(self);
        let result = operand.wrapping_sub(1);
        self.update_zero_negative(result);
        mode.writeback(self, result);
    }

    fn dex(&mut self) {
        let result = self.x.wrapping_sub(1);
        self.update_zero_negative(result);
        self.x = result;
    }

    fn dey(&mut self) {
        let result = self.y.wrapping_sub(1);
        self.update_zero_negative(result);
        self.y = result;
    }

    fn eor<T: Addressing>(&mut self, mode: T) {
        let operand = mode.address(self);
        let result = self.a ^ operand;
        self.update_zero_negative(result);
        self.a = result;
    }

    fn inc<T: Addressing>(&mut self, mode: T) {
        let operand = mode.address(self);
        let result = operand.wrapping_add(1);
        self.update_zero_negative(result);
        mode.writeback(self, result);
    }

    fn inx(&mut self) {
        let result = self.x.wrapping_add(1);
        self.update_zero_negative(result);
        self.x = result;
    }

    fn iny(&mut self) {
        let result = self.y.wrapping_add(1);
        self.update_zero_negative(result);
        self.y = result;
    }

    #[inline(always)]
    fn jmp(&mut self, mode: FromMemory) {
        self.pc = mode.addr;
    }

    fn jsr(&mut self, mode: FromMemory) {
        let ret = self.pc - 1;
        self.push16(ret);
        self.pc = mode.addr;
    }

    fn lda<T: Addressing>(&mut self, mode: T) {
        let operand = mode.address(self);
        self.update_zero_negative(operand);
        self.a = operand;
    }

    fn ldx<T: Addressing>(&mut self, mode: T) {
        let operand = mode.address(self);
        self.update_zero_negative(operand);
        self.x = operand;
    }

    fn ldy<T: Addressing>(&mut self, mode: T) {
        let operand = mode.address(self);
        self.update_zero_negative(operand);
        self.y = operand;
    }

    fn lsr<T: Addressing>(&mut self, mode: T) {
        let operand = mode.address(self);
        self.update_flag(CARRY, operand & 0x1 != 0);
        let result = operand >> 1;
        self.update_zero_negative(result);
        mode.writeback(self, result);
    }

    fn ora<T: Addressing>(&mut self, mode: T) {
        let operand = mode.address(self);
        let result = operand | self.a;
        self.update_zero_negative(result);
        self.a = result;
    }

    fn rol<T: Addressing>(&mut self, mode: T) {
        let operand = mode.address(self);
        let mut shift = (operand as u16) << 1;
        if self.flag_on(CARRY) {
            shift |= 0x1;
        }
        self.update_flag(CARRY, shift > 0xff);
        let result = shift as u8;
        self.update_zero_negative(result);
        mode.writeback(self, result);
    }

    fn ror<T: Addressing>(&mut self, mode: T) {
        let operand = mode.address(self);
        let mut shift = operand as u16;
        if self.flag_on(CARRY) {
            shift |= 0x0100;
        }
        self.update_flag(CARRY, shift & 0x1 != 0);
        let result = (shift >> 1) as u8;
        self.update_zero_negative(result);
        mode.writeback(self, result);
    }

    fn rts(&mut self) {
        self.pc = self.pop16() + 1;
    }

    fn sbc<T: Addressing>(&mut self, mode: T) {
        let operand = mode.address(self);
        let diff = self.a as u16 -
                   operand as u16 -
                   if self.flag_on(CARRY) { 0 } else { 1 };
        self.update_flag(CARRY, diff < 0x100);
        let result = diff as u8;
        self.update_zero_negative(result);
        let a = self.a;
        let cond = (a ^ result) & 0x80 != 0 && (a ^ operand) & 0x80 != 0;
        self.update_flag(OVERFLOW, cond);
        self.a = result;
    }

    #[inline(always)]
    fn sta<T: Addressing>(&mut self, mode: T) {
        let a = self.a;
        mode.writeback(self, a);
    }

    #[inline(always)]
    fn stx<T: Addressing>(&mut self, mode: T) {
        let x = self.x;
        mode.writeback(self, x);
    }

    #[inline(always)]
    fn sty<T: Addressing>(&mut self, mode: T) {
        let y = self.y;
        mode.writeback(self, y);
    }

    fn tax(&mut self) {
        let a = self.a;
        self.update_zero_negative(a);
        self.x = a;
    }

    fn tay(&mut self) {
        let a = self.a;
        self.update_zero_negative(a);
        self.y = a;
    }

    fn tsx(&mut self) {
        let sp = self.sp;
        self.update_zero_negative(sp);
        self.x = sp;
    }

    fn tsy(&mut self) {
        let sp = self.sp;
        self.update_zero_negative(sp);
        self.y = sp;
    }

    fn txa(&mut self) {
        let x = self.x;
        self.update_zero_negative(x);
        self.a = x;
    }

    fn txs(&mut self) {
        let x = self.x;
        self.update_zero_negative(x);
        self.sp = x;
    }

    fn tya(&mut self) {
        let y = self.y;
        self.update_zero_negative(y);
        self.a = y;
    }

    /*
    fn rti(&mut self) {
        cpu.p = cpu.pop();
        cpu.pc = cpu.pop_word() as u16;
    }
    */

    fn dispatch(&mut self) {
        let opcode = self.read_at_pc();
        match opcode {
            0x69 => I!(self, adc, immediate),
            0x65 => I!(self, adc, zeropage),
            0x75 => I!(self, adc, zeropage_x),
            0x6d => I!(self, adc, absolute),
            0x7d => I!(self, adc, absolute_x),
            0x79 => I!(self, adc, absolute_y),
            0x61 => I!(self, adc, indexed_indirect),
            0x71 => I!(self, adc, indirect_indexed),

            0x29 => I!(self, and, immediate),
            0x25 => I!(self, and, zeropage),
            0x35 => I!(self, and, zeropage_x),
            0x2d => I!(self, and, absolute),
            0x3d => I!(self, and, absolute_x),
            0x39 => I!(self, and, absolute_y),
            0x21 => I!(self, and, indexed_indirect),
            0x31 => I!(self, and, indirect_indexed),

            0x0a => I!(self, asl, accumulator),
            0x06 => I!(self, asl, zeropage),
            0x16 => I!(self, asl, zeropage_x),
            0x0e => I!(self, asl, absolute),
            0x1e => I!(self, asl, absolute_x),

            0x90 => I!(self, bcc, relative),
            0xb0 => I!(self, bcs, relative),
            0xf0 => I!(self, beq, relative),
            0x30 => I!(self, bmi, relative),
            0xd0 => I!(self, bne, relative),
            0x10 => I!(self, bpl, relative),
            0x50 => I!(self, bvc, relative),
            0x70 => I!(self, bvs, relative),

            0x24 => I!(self, bit, zeropage),
            0x2c => I!(self, bit, absolute),

            0x18 => self.clear_flag(CARRY),     // clc
            0xd8 => self.clear_flag(DECIMAL),   // cld
            0x58 => self.clear_flag(INTERRUPT), // cli
            0xb8 => self.clear_flag(OVERFLOW),  // clv

            0xc9 => I!(self, cmp, immediate),
            0xc5 => I!(self, cmp, zeropage),
            0xd5 => I!(self, cmp, zeropage_x),
            0xcd => I!(self, cmp, absolute),
            0xdd => I!(self, cmp, absolute_x),
            0xd9 => I!(self, cmp, absolute_y),
            0xc1 => I!(self, cmp, indexed_indirect),
            0xd1 => I!(self, cmp, indirect_indexed),

            0xe0 => I!(self, cpx, immediate),
            0xe4 => I!(self, cpx, zeropage),
            0xec => I!(self, cpx, absolute),

            0xc0 => I!(self, cpy, immediate),
            0xc4 => I!(self, cpy, zeropage),
            0xcc => I!(self, cpy, absolute),

            0xc6 => I!(self, dec, zeropage),
            0xd6 => I!(self, dec, zeropage_x),
            0xce => I!(self, dec, absolute),
            0xde => I!(self, dec, absolute_x),

            0xca => self.dex(),
            0x88 => self.dey(),

            0x49 => I!(self, eor, immediate),
            0x45 => I!(self, eor, zeropage),
            0x55 => I!(self, eor, zeropage_x),
            0x4d => I!(self, eor, absolute),
            0x5d => I!(self, eor, absolute_x),
            0x59 => I!(self, eor, absolute_y),
            0x41 => I!(self, eor, indexed_indirect),
            0x51 => I!(self, eor, indirect_indexed),

            0xe6 => I!(self, inc, zeropage),
            0xf6 => I!(self, inc, zeropage_x),
            0xee => I!(self, inc, absolute),
            0xfe => I!(self, inc, absolute_x),

            0xe8 => self.inx(),
            0xc8 => self.iny(),

            0x4c => I!(self, jmp, absolute),
            0x6c => I!(self, jmp, indirect),
    
            0x20 => I!(self, jsr, absolute),
    
            0xa9 => I!(self, lda, immediate),
            0xa5 => I!(self, lda, zeropage),
            0xb5 => I!(self, lda, zeropage_x),
            0xad => I!(self, lda, absolute),
            0xbd => I!(self, lda, absolute_x),
            0xb9 => I!(self, lda, absolute_y),
            0xa1 => I!(self, lda, indexed_indirect),
            0xb1 => I!(self, lda, indirect_indexed),

            0xa2 => I!(self, ldx, immediate),
            0xa6 => I!(self, ldx, zeropage),
            0xb6 => I!(self, ldx, zeropage_y),
            0xae => I!(self, ldx, absolute),
            0xbe => I!(self, ldx, absolute_y),

            0xa0 => I!(self, ldy, immediate),
            0xa4 => I!(self, ldy, zeropage),
            0xb4 => I!(self, ldy, zeropage_x),
            0xac => I!(self, ldy, absolute),
            0xbc => I!(self, ldy, absolute_x),

            0x4a => I!(self, lsr, accumulator),
            0x46 => I!(self, lsr, zeropage),
            0x56 => I!(self, lsr, zeropage_x),
            0x4e => I!(self, lsr, absolute),
            0x5e => I!(self, lsr, absolute_x),

            0xea => (), // nop

            0x09 => I!(self, ora, immediate),
            0x05 => I!(self, ora, zeropage),
            0x15 => I!(self, ora, zeropage_x),
            0x0d => I!(self, ora, absolute),
            0x1d => I!(self, ora, absolute_x),
            0x19 => I!(self, ora, absolute_y),
            0x01 => I!(self, ora, indexed_indirect),
            0x11 => I!(self, ora, indirect_indexed),

            0x48 => { // pha
                let a = self.a;
                self.push(a);
            }

            0x08 => { // php
                let p = self.p;
                // PHP always pushes Break flag as 1
                self.push(p | BREAK);
            }

            0x68 => self.a = self.pop(), // pla
            0x28 => self.p = self.pop(), // plp

            0x2a => I!(self, rol, accumulator),
            0x26 => I!(self, rol, zeropage),
            0x36 => I!(self, rol, zeropage_x),
            0x2e => I!(self, rol, absolute),
            0x3e => I!(self, rol, absolute_x),

            0x6a => I!(self, ror, accumulator),
            0x66 => I!(self, ror, zeropage),
            0x76 => I!(self, ror, zeropage_x),
            0x6e => I!(self, ror, absolute),
            0x7e => I!(self, ror, absolute_x),

            0x60 => self.rts(),

            0xe9 => I!(self, sbc, immediate),
            0xe5 => I!(self, sbc, zeropage),
            0xf5 => I!(self, sbc, zeropage_x),
            0xed => I!(self, sbc, absolute),
            0xfd => I!(self, sbc, absolute_x),
            0xf9 => I!(self, sbc, absolute_y),
            0xe1 => I!(self, sbc, indexed_indirect),
            0xf1 => I!(self, sbc, indirect_indexed),

            0x38 => self.set_flag(CARRY),     // sec
            0xf8 => self.set_flag(DECIMAL),   // sed
            0x78 => self.set_flag(INTERRUPT), // sei

            0x85 => I!(self, sta, zeropage),
            0x95 => I!(self, sta, zeropage_x),
            0x8d => I!(self, sta, absolute),
            0x9d => I!(self, sta, absolute_x),
            0x99 => I!(self, sta, absolute_y),
            0x81 => I!(self, sta, indexed_indirect),
            0x91 => I!(self, sta, indirect_indexed),

            0x86 => I!(self, stx, zeropage),
            0x96 => I!(self, stx, zeropage_y),
            0x8e => I!(self, stx, absolute),

            0x84 => I!(self, sty, zeropage),
            0x94 => I!(self, sty, zeropage_x),
            0x8c => I!(self, sty, absolute),

            0xaa => self.tax(),
            0xa8 => self.tay(),
            0xba => self.tsx(),
            0x8a => self.txa(),
            0x9a => self.txs(),
            0x98 => self.tya(),

            _ => panic!("unknown opcode {} pc={:x}", opcode, self.pc)
        }

        self.cycles += CYCLES[opcode as usize];
        if self.check_xpage {
            self.cycles += XPAGE_CYCLES[opcode as usize];
        }
    }
}
