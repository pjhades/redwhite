use mem::{Address, Memory, Access};

const FLAG_NEGATIVE:  u8 = 0b1000_0000;
const FLAG_OVERFLOW:  u8 = 0b0100_0000;
const FLAG_X:         u8 = 0b0010_0000;
const FLAG_BREAK:     u8 = 0b0001_0000;
const FLAG_DECIMAL:   u8 = 0b0000_1000;
const FLAG_INTERRUPT: u8 = 0b0000_0100;
const FLAG_ZERO:      u8 = 0b0000_0010;
const FLAG_CARRY:     u8 = 0b0000_0001;

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

struct AddressingMode {
    at: Address,
}

impl AddressingMode {
    fn new() -> Self {
        AddressingMode { at: 0 }
    }

    #[inline(always)]
    fn wb(&mut self, cpu: &mut Cpu, value: u8) {
        cpu.write(self.at, value);
    }

    #[inline(always)]
    fn accumulator(&mut self, cpu: &mut Cpu) -> u8 {
        cpu.a
    }

    #[inline(always)]
    fn accumulator_wb(&mut self, cpu: &mut Cpu, value: u8) {
        cpu.a = value;
    }

    #[inline(always)]
    fn immediate(&mut self, cpu: &mut Cpu) -> u8 {
        cpu.fetch()
    }

    fn zeropage(&mut self, cpu: &mut Cpu) -> u8 {
        self.at = cpu.fetch() as Address;
        cpu.read(self.at)
    }

    fn zeropage_x(&mut self, cpu: &mut Cpu) -> u8 {
        self.at = (cpu.fetch() as Address + cpu.x as Address) & 0x00ff;
        cpu.read(self.at)
    }

    fn zeropage_y(&mut self, cpu: &mut Cpu) -> u8 {
        self.at = (cpu.fetch() as Address + cpu.y as Address) & 0x00ff;
        cpu.read(self.at)
    }

    fn absolute(&mut self, cpu: &mut Cpu) -> u8 {
        self.at = cpu.fetch_word();
        cpu.read(self.at)
    }

    fn absolute_x(&mut self, cpu: &mut Cpu) -> u8 {
        let base = cpu.fetch_word();
        self.at = base.wrapping_add(cpu.x as Address);
        cpu.read(self.at)
    }

    fn absolute_y(&mut self, cpu: &mut Cpu) -> u8 {
        let base = cpu.fetch_word();
        self.at = base.wrapping_add(cpu.y as Address);
        cpu.read(self.at)
    }

    fn absolute_x_chk(&mut self, cpu: &mut Cpu) -> u8 {
        let base = cpu.fetch_word();
        self.at = base.wrapping_add(cpu.x as Address);

        if does_x_page(base, self.at) {
            cpu.cycle_count += 1;
        }

        cpu.read(self.at)
    }

    fn absolute_y_chk(&mut self, cpu: &mut Cpu) -> u8 {
        let base = cpu.fetch_word();
        self.at = base.wrapping_add(cpu.y as Address);

        if does_x_page(base, self.at) {
            cpu.cycle_count += 1;
        }

        cpu.read(self.at)
    }

    // indexed indirect
    fn indirect_x(&mut self, cpu: &mut Cpu) -> u8 {
        let base = cpu.fetch();
        let lo = base.wrapping_add(cpu.x) as Address;
        let hi = (lo + 1) & 0x00ff;

        self.at = cpu.read(lo) as Address |
                  (cpu.read(hi) as Address) << 8;

        cpu.read(self.at)
    }

    // indirect indexed
    fn indirect_y(&mut self, cpu: &mut Cpu) -> u8 {
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

    fn relative(&mut self, cpu: &mut Cpu) -> u8 {
        let offset = cpu.fetch() as i8 as i16;
        self.at = (cpu.pc as i16).wrapping_add(offset) as Address;
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
    fn set_zero_negative(&mut self, value: u8) {
        self.set_flag_if(FLAG_ZERO, value == 0);
        self.set_flag_if(FLAG_NEGATIVE, value & 0x80 != 0);
    }

    fn fetch(&mut self) -> u8 {
        let value = self.read(self.pc);
        self.pc += 1;
        value
    }

    fn fetch_word(&mut self) -> u16 {
        let value = self.read_word(self.pc);
        self.pc += 2;
        value
    }

    fn push(&mut self, value: u8) {
        let at = self.sp as Address + 0x0100;
        self.write(at, value);
        self.sp = self.sp.wrapping_sub(1);
    }

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
        if self.is_flag_set(FLAG_CARRY) {
            result += 1;
        }

        self.set_flag_if(FLAG_CARRY, result > 0xff);

        let result = result as u8;
        self.set_zero_negative(result);

        let a = self.a;
        self.set_flag_if(FLAG_OVERFLOW, (a ^ operand) & 0x80 == 0 && (a ^ result) & 0x80 != 0);

        self.a = result;
        result
    }

    fn and(&mut self, operand: u8) {
        let result = operand & self.a;
        self.set_zero_negative(result);
        self.a = result;
    }

    fn asl(&mut self, operand: u8) -> u8 {
        self.set_flag_if(FLAG_CARRY, operand & 0x80 != 0);
        let result = operand << 1;
        self.set_zero_negative(result);
        result
    }

    fn bcc(&mut self, at: Address) {
        let cond = !self.is_flag_set(FLAG_CARRY);
        self.jump_on_condition(at, cond);
    }

    fn bcs(&mut self, at: Address) {
        let cond = self.is_flag_set(FLAG_CARRY);
        self.jump_on_condition(at, cond);
    }

    fn beq(&mut self, at: Address) {
        let cond = self.is_flag_set(FLAG_ZERO);
        self.jump_on_condition(at, cond);
    }

    fn bmi(&mut self, at: Address) {
        let cond = self.is_flag_set(FLAG_NEGATIVE);
        self.jump_on_condition(at, cond);
    }

    fn bne(&mut self, at: Address) {
        let cond = !self.is_flag_set(FLAG_ZERO);
        self.jump_on_condition(at, cond);
    }

    fn bpl(&mut self, at: Address) {
        let cond = !self.is_flag_set(FLAG_NEGATIVE);
        self.jump_on_condition(at, cond);
    }

    fn bvc(&mut self, at: Address) {
        let cond = !self.is_flag_set(FLAG_OVERFLOW);
        self.jump_on_condition(at, cond);
    }

    fn bvs(&mut self, at: Address) {
        let cond = self.is_flag_set(FLAG_OVERFLOW);
        self.jump_on_condition(at, cond);
    }

    fn bit(&mut self, operand: u8) {
        self.set_flag_if(FLAG_NEGATIVE, operand & 0x80 != 0);
        self.set_flag_if(FLAG_OVERFLOW, operand & 0x40 != 0);
        if operand & self.a == 0 {
            self.set_flag(FLAG_ZERO);
        }
        else {
            self.clear_flag(FLAG_ZERO);
        }
    }

    #[inline(always)]
    fn clc(&mut self) {
        self.clear_flag(FLAG_CARRY);
    }

    #[inline(always)]
    fn cld(&mut self) {
        self.clear_flag(FLAG_DECIMAL);
    }

    #[inline(always)]
    fn cli(&mut self) {
        self.clear_flag(FLAG_INTERRUPT);
    }

    #[inline(always)]
    fn clv(&mut self) {
        self.clear_flag(FLAG_OVERFLOW);
    }

    // cmp, cpx, cpy
    fn cmp_with_reg(&mut self, reg: u8, operand: u8) {
        let result = reg as i8 - operand as i8;
        self.set_flag_if(FLAG_CARRY, result >= 0);
        self.set_zero_negative(result as u8);
    }

    fn dec(&mut self, operand: u8) -> u8 {
        let result = operand.wrapping_sub(1);
        self.set_zero_negative(result);
        result
    }

    fn dex(&mut self) {
        let result = self.x.wrapping_sub(1);
        self.set_zero_negative(result);
        self.x = result;
    }

    fn dey(&mut self) {
        let result = self.y.wrapping_sub(1);
        self.set_zero_negative(result);
        self.y = result;
    }

    fn eor(&mut self, operand: u8) {
        let result = self.a ^ operand;
        self.set_zero_negative(result);
        self.a = result;
    }

    fn inc(&mut self, operand: u8) -> u8 {
        let result = operand.wrapping_add(1);
        self.set_zero_negative(result);
        result
    }

    fn inx(&mut self) -> u8 {
        let result = self.x.wrapping_add(1);
        self.set_zero_negative(result);
        result
    }

    fn iny(&mut self) -> u8 {
        let result = self.y.wrapping_add(1);
        self.set_zero_negative(result);
        result
    }

    #[inline(always)]
    fn jmp(&mut self, at: Address) {
        self.pc = at;
    }

    fn jsr(&mut self, at: Address) {
        let ret = self.pc.saturating_sub(1);
        self.push(((ret & 0xff00) >> 8) as u8);
        self.push(ret as u8);
        self.pc = at;
    }

    fn lda(&mut self, operand: u8) {
        self.set_zero_negative(operand);
        self.a = operand;
    }

    fn ldx(&mut self, operand: u8) {
        self.set_zero_negative(operand);
        self.x = operand;
    }

    fn ldy(&mut self, operand: u8) {
        self.set_zero_negative(operand);
        self.y = operand;
    }

    fn lsr(&mut self, operand: u8) -> u8 {
        self.set_flag_if(FLAG_CARRY, operand & 0x1 != 0);
        let result = operand >> 1;
        self.set_zero_negative(result);
        result
    }

    fn ora(&mut self, operand: u8) {
        let result = operand | self.a;
        self.set_zero_negative(result);
        self.a = result;
    }

    fn rol(&mut self, operand: u8) -> u8 {
        let mut result = (operand as u16) << 1;
        if self.is_flag_set(FLAG_CARRY) {
            result |= 0x1;
        }
        self.set_flag_if(FLAG_CARRY, result > 0x00ff);
        let result = result as u8;
        self.set_zero_negative(result);
        result
    }

    fn ror(&mut self, operand: u8) -> u8 {
        let mut result = operand as u16;
        if self.is_flag_set(FLAG_CARRY) {
            result |= 0x0100;
        }
        self.set_flag_if(FLAG_CARRY, result & 0x0001 != 0);
        let result = (result >> 1) as u8;
        self.set_zero_negative(result);
        result
    }

    fn rti(&mut self) {
        self.p = self.pop();
        let at = self.pop() as Address |
                 (self.pop() as Address) << 8;
        self.pc = at;
    }

    fn rts(&mut self) {
        let at = self.pop() as Address |
                 (self.pop() as Address) << 8;
        let at = at.saturating_add(1);
        self.pc = at;
    }

    fn sbc(&mut self, operand: u8) -> u8 {
        let result = self.a as u16
            - operand as u16
            - if self.is_flag_set(FLAG_CARRY) { 0 } else { 1 };
        self.set_flag_if(FLAG_CARRY, result < 0x0100);
        let result = result as u8;
        self.set_zero_negative(result);
        let a = self.a;
        self.set_flag_if(FLAG_OVERFLOW, (a ^ result) & 0x80 != 0 && (a ^ operand) & 0x80 != 0);
        self.a = result;
        result
    }

    #[inline(always)]
    fn sec(&mut self) {
        self.set_flag(FLAG_CARRY);
    }

    #[inline(always)]
    fn sei(&mut self) {
        self.set_flag(FLAG_INTERRUPT);
    }

    fn sta(&mut self, at: Address) {
        let value = self.a;
        self.write(at, value);
    }

    fn stx(&mut self, at: Address) {
        let value = self.a;
        self.write(at, value);
    }

    fn sty(&mut self, at: Address) {
        let value = self.y;
        self.write(at, value);
    }

    fn tax(&mut self) {
        let value = self.a;
        self.set_zero_negative(value);
        self.x = value;
    }

    fn tay(&mut self) {
        let value = self.a;
        self.set_zero_negative(value);
        self.y = value;
    }

    fn tsx(&mut self) {
        let value = self.sp;
        self.set_zero_negative(value);
        self.x = value;
    }

    fn tsy(&mut self) {
        let value = self.sp;
        self.set_zero_negative(value);
        self.y = value;
    }

    fn txa(&mut self) {
        let value = self.x;
        self.set_zero_negative(value);
        self.a = value;
    }

    fn txs(&mut self) {
        let value = self.x;
        self.set_zero_negative(value);
        self.sp = value;
    }

    fn tya(&mut self) {
        let value = self.y;
        self.set_zero_negative(value);
        self.a = value;
    }
}

macro_rules! r {
    ($inst:ident, $cpu:ident, $m:ident, $mode:ident) => {
        {
            let operand = $m.$mode($cpu);
            $cpu.$inst(operand);
        }
    }
}

macro_rules! rw {
    ($inst:ident, $cpu:ident, $m:ident, accumulator) => {
        {
            let operand = $m.accumulator($cpu);
            let result = $cpu.$inst(operand);
            $m.accumulator_wb(result);
        }
    }

    ($inst:ident, $cpu:ident, $m:ident, $mode:ident) => {
        {
            let operand = $m.$mode($cpu);
            let result = $cpu.$inst(operand);
            $m.wb(result);
        }
    }
}

fn decode(cpu: &mut Cpu) {
    let opcode = cpu.fetch();
    let mut mode = AddressingMode::new();

    match opcode {
        0x69 => r!(adc, cpu, mode, immediate),
        0x65 => r!(adc, cpu, mode, zeropage),
        0x75 => r!(adc, cpu, mode, zeropage_x),
        0x60 => r!(adc, cpu, mode, absolute),
        0x70 => r!(adc, cpu, mode, absolute_x_chk),
        0x79 => r!(adc, cpu, mode, absolute_y_chk),
        0x61 => r!(adc, cpu, mode, indirect_x),
        0x71 => r!(adc, cpu, mode, indirect_y),

        0x29 => r!(and, cpu, mode, immediate),
        0x25 => r!(and, cpu, mode, zeropage),
        0x35 => r!(and, cpu, mode, zeropage_x),
        0x2d => r!(and, cpu, mode, absolute),
        0x3d => r!(and, cpu, mode, absolute_x_chk),
        0x39 => r!(and, cpu, mode, absolute_y_chk),
        0x21 => r!(and, cpu, mode, indirect_x),
        0x31 => r!(and, cpu, mode, indirect_y),

        0x0a => rw!(asl, cpu, mode, accumulator),
        0x06 => rw!(asl, cpu, mode, zeropage),
        0x16 => rw!(asl, cpu, mode, zeropage_x),
        0x0e => rw!(asl, cpu, mode, absolute),
        0x1e => rw!(asl, cpu, mode, absolute_x),

        _ => panic!("unknown opcode {} at pc={:x}", opcode, cpu.pc - 1)
    }
}
