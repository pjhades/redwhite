use mem::{Memory, Access};

const CYCLE_TABLE: [u8;256] = [
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

const CROSSPAGE_CYCLE_TABLE: [u8;256] = [
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

pub struct Cpu {
    // registers
    a: u8,
    x: u8,
    y: u8,
    sp: u8,
    pc: u16,
    p: u8,

    // status flags
    negative: bool,
    overflow: bool,
    unused: bool,
    brk: bool,
    decimal: bool,
    interrupt: bool,
    zero: bool,
    carry: bool,

    // misc
    cycle_count: usize,
    mem: Memory,
}

impl Access for Cpu {
    fn read(&self, at: u16) -> u8 {
        self.mem.read(at)
    }

    fn write(&mut self, at: u16, value: u8) {
        self.mem.write(at, value)
    }
}

struct AddressingMode {
    at: u16,
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
        self.at = cpu.fetch() as u16;
        cpu.read(self.at)
    }

    fn zeropage_x(&mut self, cpu: &mut Cpu) -> u8 {
        self.at = (cpu.fetch() as u16 + cpu.x as u16) & 0x00ff;
        cpu.read(self.at)
    }

    fn zeropage_y(&mut self, cpu: &mut Cpu) -> u8 {
        self.at = (cpu.fetch() as u16 + cpu.y as u16) & 0x00ff;
        cpu.read(self.at)
    }

    fn absolute(&mut self, cpu: &mut Cpu) -> u8 {
        self.at = cpu.fetch_word();
        cpu.read(self.at)
    }

    fn absolute_x(&mut self, cpu: &mut Cpu) -> u8 {
        let base = cpu.fetch_word();
        self.at = base.wrapping_add(cpu.x as u16);
        cpu.read(self.at)
    }

    fn absolute_y(&mut self, cpu: &mut Cpu) -> u8 {
        let base = cpu.fetch_word();
        self.at = base.wrapping_add(cpu.y as u16);
        cpu.read(self.at)
    }

    // indexed indirect
    fn indirect_x(&mut self, cpu: &mut Cpu) -> u8 {
        let base = cpu.fetch();
        let lo = base.wrapping_add(cpu.x) as u16;
        let hi = (lo as u8).wrapping_add(1) as u16;

        self.at = cpu.read(lo) as u16 | (cpu.read(hi) as u16) << 8;

        cpu.read(self.at)
    }

    // indirect indexed
    fn indirect_y(&mut self, cpu: &mut Cpu) -> u8 {
        let lo = cpu.fetch() as u16;
        let hi = (lo as u8).wrapping_add(1) as u16;
        let base = cpu.read(lo) as u16 | (cpu.read(hi) as u16) << 8;

        self.at = base.wrapping_add(cpu.y as u16);

        cpu.read(self.at)
    }

    fn relative(&mut self, cpu: &mut Cpu) {
        let offset = cpu.fetch() as i8 as i16;
        self.at = (cpu.pc as i16).wrapping_add(offset) as u16;
    }
}

impl Cpu {
    pub fn new() -> Self {
        let mut cpu = Cpu {
            a: 0,
            x: 0,
            y: 0,
            sp: 0,
            pc: 0,
            p: 0,

            negative: false,
            overflow: false,
            unused: false,
            brk: false,
            decimal: false,
            interrupt: false,
            zero: false,
            carry: false,

            cycle_count: 0,
            mem: Memory::new(),
        };

        // Set power-up state
        // Ref: https://wiki.nesdev.com/w/index.php/CPU_power_up_state
        cpu.sp = 0xfd;
        cpu.set_status_from_byte(0x34);
        cpu
    }

    fn status_as_byte(&self) -> u8 {
        (if self.negative  { 1 } else { 0 }) << 7 |
        (if self.overflow  { 1 } else { 0 }) << 6 |
        (if self.unused    { 1 } else { 0 }) << 5 |
        (if self.brk       { 1 } else { 0 }) << 4 |
        (if self.decimal   { 1 } else { 0 }) << 3 |
        (if self.interrupt { 1 } else { 0 }) << 2 |
        (if self.zero      { 1 } else { 0 }) << 1 |
        (if self.carry     { 1 } else { 0 })
    }

    fn set_status_from_byte(&mut self, byte: u8) {
        if (byte & 0b1000_0000) != 0 { self.negative = true; }
        if (byte & 0b0100_0000) != 0 { self.overflow = true; }
        if (byte & 0b0010_0000) != 0 { self.unused = true; }
        if (byte & 0b0001_0000) != 0 { self.brk = true; }
        if (byte & 0b0000_1000) != 0 { self.decimal = true; }
        if (byte & 0b0000_0100) != 0 { self.interrupt = true; }
        if (byte & 0b0000_0010) != 0 { self.zero = true; }
        if (byte & 0b0000_0001) != 0 { self.carry = true; }
    }

    #[inline(always)]
    fn set_zero_negative(&mut self, value: u8) {
        if value == 0 {
            self.zero = true;
        }
        if value & 0x80 != 0 {
            self.negative = true;
        }
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
        let at = self.sp as u16 + 0x0100;
        self.write(at, value);
        self.sp = self.sp.wrapping_sub(1);
    }

    fn push_word(&mut self, value: u16) {
        self.push(((value & 0xff00) >> 8) as u8);
        self.push(value as u8);
    }

    fn pop(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        self.read(self.sp as u16 + 0x0100)
    }

    fn pop_word(&mut self) -> u16 {
        let value = self.pop() as u16;
        let value = value | (self.pop() as u16) << 8;
        value
    }

    fn jump_on_condition(&mut self, at: u16, condition: bool) {
        if condition {
            self.cycle_count += 1;
            self.pc = at;
        }
    }

    fn adc(&mut self, operand: u8) {
        let mut result = operand as u16 + self.a as u16;
        if self.carry {
            result += 1;
        }
        if result > 0xff {
            self.carry = true;
        }
        let result = result as u8;
        self.set_zero_negative(result);
        let a = self.a;
        if (a ^ operand) & 0x80 == 0 && (a ^ result) & 0x80 != 0 {
            self.overflow = true;
        }
        // XXX merge with accumulator
        self.a = result;
    }

    fn and(&mut self, operand: u8) {
        let result = operand & self.a;
        self.set_zero_negative(result);
        // XXX merge with accumulator
        self.a = result;
    }

    fn asl(&mut self, operand: u8) -> u8 {
        if operand & 0x80 != 0 {
            self.carry = true;
        }
        let result = operand << 1;
        self.set_zero_negative(result);
        result
    }

    fn bcc(&mut self, at: u16) {
        let cond = !self.carry;
        self.jump_on_condition(at, cond);
    }

    fn bcs(&mut self, at: u16) {
        let cond = self.carry;
        self.jump_on_condition(at, cond);
    }

    fn beq(&mut self, at: u16) {
        let cond = self.zero;
        self.jump_on_condition(at, cond);
    }

    fn bmi(&mut self, at: u16) {
        let cond = self.negative;
        self.jump_on_condition(at, cond);
    }

    fn bne(&mut self, at: u16) {
        let cond = !self.zero;
        self.jump_on_condition(at, cond);
    }

    fn bpl(&mut self, at: u16) {
        let cond = !self.negative;
        self.jump_on_condition(at, cond);
    }

    fn bvc(&mut self, at: u16) {
        let cond = !self.overflow;
        self.jump_on_condition(at, cond);
    }

    fn bvs(&mut self, at: u16) {
        let cond = self.overflow;
        self.jump_on_condition(at, cond);
    }

    fn bit(&mut self, operand: u8) {
        if operand & 0x80 != 0 {
            self.negative = true;
        }
        if operand & 0x40 != 0 {
            self.overflow = true;
        }
        if operand & self.a == 0 {
            self.zero = true;
        }
        else {
            self.zero = false;
        }
    }

    fn cmp(&mut self, operand: u8) {
        let result = self.a as i8 - operand as i8;
        if result >= 0 {
            self.carry = true;
        }
        self.set_zero_negative(result as u8);
    }

    fn cpx(&mut self, operand: u8) {
        let result = self.x as i8 - operand as i8;
        if result >= 0 {
            self.carry = true;
        }
        self.set_zero_negative(result as u8);
    }

    fn cpy(&mut self, operand: u8) {
        let result = self.y as i8 - operand as i8;
        if result >= 0 {
            self.carry = true;
        }
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
        // XXX merge with accumulator
        self.a = result;
    }

    fn inc(&mut self, operand: u8) -> u8 {
        let result = operand.wrapping_add(1);
        self.set_zero_negative(result);
        result
    }

    fn inx(&mut self) {
        let result = self.x.wrapping_add(1);
        self.set_zero_negative(result);
        self.x = result;
    }

    fn iny(&mut self) {
        let result = self.y.wrapping_add(1);
        self.set_zero_negative(result);
        self.y = result;
    }

    fn jsr(&mut self, at: u16) {
        let ret = self.pc.saturating_sub(1);
        self.push_word(ret);
        self.pc = at;
    }

    fn lda(&mut self, operand: u8) {
        self.set_zero_negative(operand);
        // XXX merge with accumulator
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
        if operand & 0x1 != 0 {
            self.carry = true;
        }
        let result = operand >> 1;
        self.set_zero_negative(result);
        result
    }

    fn ora(&mut self, operand: u8) {
        let result = operand | self.a;
        self.set_zero_negative(result);
        // XXX merge with accumulator
        self.a = result;
    }

    fn rol(&mut self, operand: u8) -> u8 {
        let mut result = (operand as u16) << 1;
        if self.carry {
            result |= 0x1;
        }
        if result > 0x00ff {
            self.carry = true;
        }
        let result = result as u8;
        self.set_zero_negative(result);
        result
    }

    fn ror(&mut self, operand: u8) -> u8 {
        let mut result = operand as u16;
        if self.carry {
            result |= 0x0100;
        }
        if result & 0x0001 != 0 {
            self.carry = true;
        }
        let result = (result >> 1) as u8;
        self.set_zero_negative(result);
        result
    }

    fn rti(&mut self) {
        self.p = self.pop();
        self.pc = self.pop_word() as u16;
    }

    fn rts(&mut self) {
        let at = self.pop_word() as u16;
        let at = at.saturating_add(1);
        self.pc = at;
    }

    fn sbc(&mut self, operand: u8) {
        let result = self.a as u16
            - operand as u16
            - if self.carry { 0 } else { 1 };
        if result < 0x0100 {
            self.carry = true;
        }
        let result = result as u8;
        self.set_zero_negative(result);
        let a = self.a;
        if (a ^ result) & 0x80 != 0 && (a ^ operand) & 0x80 != 0 {
            self.overflow = true;
        }
        // XXX merge with accumulator
        self.a = result;
    }

    fn sta(&mut self, at: u16) {
        let value = self.a;
        self.write(at, value);
    }

    fn stx(&mut self, at: u16) {
        let value = self.x;
        self.write(at, value);
    }

    fn sty(&mut self, at: u16) {
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

macro_rules! rdonly {
    ($inst:ident, $cpu:ident, $m:ident, $mode:ident) => {
        {
            let operand = $m.$mode($cpu);
            $cpu.$inst(operand);
        }
    }
}

macro_rules! rdwr {
    ($inst:ident, $cpu:ident, $m:ident, accumulator) => {
        {
            let operand = $m.accumulator($cpu);
            let result = $cpu.$inst(operand);
            $m.accumulator_wb($cpu, result);
        }
    };

    ($inst:ident, $cpu:ident, $m:ident, $mode:ident) => {
        {
            let operand = $m.$mode($cpu);
            let result = $cpu.$inst(operand);
            $m.wb($cpu, result);
        }
    }
}

macro_rules! other {
    ($inst:ident, $cpu:ident, $m:ident, $mode:ident) => {
        {
            $m.$mode($cpu);
            $cpu.$inst($m.at);
        }
    }
}

fn decode(cpu: &mut Cpu) {
    let opcode = cpu.fetch();
    let mut m = AddressingMode::new();

    match opcode {
        0x69 => rdonly!(adc, cpu, m, immediate),
        0x65 => rdonly!(adc, cpu, m, zeropage),
        0x75 => rdonly!(adc, cpu, m, zeropage_x),
        0x6d => rdonly!(adc, cpu, m, absolute),
        0x7d => rdonly!(adc, cpu, m, absolute_x),
        0x79 => rdonly!(adc, cpu, m, absolute_y),
        0x61 => rdonly!(adc, cpu, m, indirect_x),
        0x71 => rdonly!(adc, cpu, m, indirect_y),

        0x29 => rdonly!(and, cpu, m, immediate),
        0x25 => rdonly!(and, cpu, m, zeropage),
        0x35 => rdonly!(and, cpu, m, zeropage_x),
        0x2d => rdonly!(and, cpu, m, absolute),
        0x3d => rdonly!(and, cpu, m, absolute_x),
        0x39 => rdonly!(and, cpu, m, absolute_y),
        0x21 => rdonly!(and, cpu, m, indirect_x),
        0x31 => rdonly!(and, cpu, m, indirect_y),

        0x0a => rdwr!(asl, cpu, m, accumulator),
        0x06 => rdwr!(asl, cpu, m, zeropage),
        0x16 => rdwr!(asl, cpu, m, zeropage_x),
        0x0e => rdwr!(asl, cpu, m, absolute),
        0x1e => rdwr!(asl, cpu, m, absolute_x),

        0x90 => other!(bcc, cpu, m, relative),
        0xb0 => other!(bcs, cpu, m, relative),
        0xf0 => other!(beq, cpu, m, relative),
        0x30 => other!(bmi, cpu, m, relative),
        0xd0 => other!(bne, cpu, m, relative),
        0x10 => other!(bpl, cpu, m, relative),
        0x50 => other!(bvc, cpu, m, relative),
        0x70 => other!(bvs, cpu, m, relative),

        0x24 => rdonly!(bit, cpu, m, zeropage),
        0x2c => rdonly!(bit, cpu, m, absolute),

        0x18 => cpu.carry = false,     // clc
        0xd8 => cpu.decimal = false,   // cld
        0x58 => cpu.interrupt = false, // cli
        0xb8 => cpu.overflow = false,  // clv

        0xc9 => rdonly!(cmp, cpu, m, immediate),
        0xc5 => rdonly!(cmp, cpu, m, zeropage),
        0xd5 => rdonly!(cmp, cpu, m, zeropage_x),
        0xcd => rdonly!(cmp, cpu, m, absolute),
        0xdd => rdonly!(cmp, cpu, m, absolute_x),
        0xd9 => rdonly!(cmp, cpu, m, absolute_y),
        0xc1 => rdonly!(cmp, cpu, m, indirect_x),
        0xd1 => rdonly!(cmp, cpu, m, indirect_y),

        0xe0 => rdonly!(cpx, cpu, m, immediate),
        0xe4 => rdonly!(cpx, cpu, m, zeropage),
        0xec => rdonly!(cpx, cpu, m, absolute),

        0xc0 => rdonly!(cpy, cpu, m, immediate),
        0xc4 => rdonly!(cpy, cpu, m, zeropage),
        0xcc => rdonly!(cpy, cpu, m, absolute),

        0xc6 => rdwr!(dec, cpu, m, zeropage),
        0xd6 => rdwr!(dec, cpu, m, zeropage_x),
        0xce => rdwr!(dec, cpu, m, absolute),
        0xde => rdwr!(dec, cpu, m, absolute_x),

        0xca => cpu.dex(),
        0x88 => cpu.dey(),

        0x49 => rdonly!(eor, cpu, m, immediate),
        0x45 => rdonly!(eor, cpu, m, zeropage),
        0x55 => rdonly!(eor, cpu, m, zeropage_x),
        0x4d => rdonly!(eor, cpu, m, absolute),
        0x5d => rdonly!(eor, cpu, m, absolute_x),
        0x59 => rdonly!(eor, cpu, m, absolute_y),
        0x41 => rdonly!(eor, cpu, m, indirect_x),
        0x51 => rdonly!(eor, cpu, m, indirect_y),

        0xe6 => rdwr!(inc, cpu, m, zeropage),
        0xf6 => rdwr!(inc, cpu, m, zeropage_x),
        0xee => rdwr!(inc, cpu, m, absolute),
        0xfe => rdwr!(inc, cpu, m, absolute_x),

        0xe8 => cpu.inx(),
        0xc8 => cpu.iny(),

        // jmp
        0x4c => cpu.pc = cpu.fetch_word(),
        0x6c => {
            let at = cpu.fetch_word();
            cpu.pc = cpu.read(at) as u16 | (cpu.read(at.wrapping_add(1)) as u16) << 8;
        }

        0x20 => {
            let at = cpu.fetch_word();
            cpu.jsr(at);
        }

        0x60 => cpu.rts(),

        0xa9 => rdonly!(lda, cpu, m, immediate),
        0xa5 => rdonly!(lda, cpu, m, zeropage),
        0xb5 => rdonly!(lda, cpu, m, zeropage_x),
        0xad => rdonly!(lda, cpu, m, absolute),
        0xbd => rdonly!(lda, cpu, m, absolute_x),
        0xb9 => rdonly!(lda, cpu, m, absolute_y),
        0xa1 => rdonly!(lda, cpu, m, indirect_x),
        0xb1 => rdonly!(lda, cpu, m, indirect_y),

        0xa2 => rdonly!(ldx, cpu, m, immediate),
        0xa6 => rdonly!(ldx, cpu, m, zeropage),
        0xb6 => rdonly!(ldx, cpu, m, zeropage_y),
        0xae => rdonly!(ldx, cpu, m, absolute),
        0xbe => rdonly!(ldx, cpu, m, absolute_y),

        0xa0 => rdonly!(ldy, cpu, m, immediate),
        0xa4 => rdonly!(ldy, cpu, m, zeropage),
        0xb4 => rdonly!(ldy, cpu, m, zeropage_x),
        0xac => rdonly!(ldy, cpu, m, absolute),
        0xbc => rdonly!(ldy, cpu, m, absolute_x),

        0x4a => rdwr!(lsr, cpu, m, accumulator),
        0x46 => rdwr!(lsr, cpu, m, zeropage),
        0x56 => rdwr!(lsr, cpu, m, zeropage_x),
        0x4e => rdwr!(lsr, cpu, m, absolute),
        0x5e => rdwr!(lsr, cpu, m, absolute_x),

        0xea => (), // nop

        0x09 => rdonly!(ora, cpu, m, immediate),
        0x05 => rdonly!(ora, cpu, m, zeropage),
        0x15 => rdonly!(ora, cpu, m, zeropage_x),
        0x0d => rdonly!(ora, cpu, m, absolute),
        0x1d => rdonly!(ora, cpu, m, absolute_x),
        0x19 => rdonly!(ora, cpu, m, absolute_y),
        0x01 => rdonly!(ora, cpu, m, indirect_x),
        0x11 => rdonly!(ora, cpu, m, indirect_y),

        // pha
        0x48 => {
            let value = cpu.a;
            cpu.push(value);
        }

        // php
        0x08 => {
            let value = cpu.p;
            cpu.push(value);
        }

        0x68 => cpu.a = cpu.pop(), // pla
        0x28 => cpu.p = cpu.pop(), // plp

        0x2a => rdwr!(rol, cpu, m, accumulator),
        0x26 => rdwr!(rol, cpu, m, zeropage),
        0x36 => rdwr!(rol, cpu, m, zeropage_x),
        0x2e => rdwr!(rol, cpu, m, absolute),
        0x3e => rdwr!(rol, cpu, m, absolute_x),

        0x6a => rdwr!(ror, cpu, m, accumulator),
        0x66 => rdwr!(ror, cpu, m, zeropage),
        0x76 => rdwr!(ror, cpu, m, zeropage_x),
        0x6e => rdwr!(ror, cpu, m, absolute),
        0x7e => rdwr!(ror, cpu, m, absolute_x),

        0xe9 => rdonly!(sbc, cpu, m, immediate),
        0xe5 => rdonly!(sbc, cpu, m, zeropage),
        0xf5 => rdonly!(sbc, cpu, m, zeropage_x),
        0xed => rdonly!(sbc, cpu, m, absolute),
        0xfd => rdonly!(sbc, cpu, m, absolute_x),
        0xf9 => rdonly!(sbc, cpu, m, absolute_y),
        0xe1 => rdonly!(sbc, cpu, m, indirect_x),
        0xf1 => rdonly!(sbc, cpu, m, indirect_y),

        0x38 => cpu.carry = true,     // sec
        0xf8 => cpu.decimal = true,   // sed
        0x78 => cpu.interrupt = true, // sei

        0x85 => other!(sta, cpu, m, zeropage),
        0x95 => other!(sta, cpu, m, zeropage_x),
        0x8d => other!(sta, cpu, m, absolute),
        0x9d => other!(sta, cpu, m, absolute_x),
        0x99 => other!(sta, cpu, m, absolute_y),
        0x81 => other!(sta, cpu, m, indirect_x),
        0x91 => other!(sta, cpu, m, indirect_y),

        0x86 => other!(stx, cpu, m, zeropage),
        0x96 => other!(stx, cpu, m, zeropage_y),
        0x8e => other!(stx, cpu, m, absolute),

        0x84 => other!(sty, cpu, m, zeropage),
        0x94 => other!(sty, cpu, m, zeropage_x),
        0x8c => other!(sty, cpu, m, absolute),

        0xaa => cpu.tax(),
        0xa8 => cpu.tay(),
        0xba => cpu.tsx(),
        0x8a => cpu.txa(),
        0x9a => cpu.txs(),
        0x98 => cpu.tya(),

        _ => panic!("unknown opcode {} at pc={:x}", opcode, cpu.pc - 1)
    }

    cpu.cycle_count += CYCLE_TABLE[opcode as usize] as usize;
    cpu.cycle_count += CROSSPAGE_CYCLE_TABLE[opcode as usize] as usize;
}
