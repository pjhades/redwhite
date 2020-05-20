use memory::{Mem, CpuMem};

struct Status {
    pub negative: bool,
    pub overflow: bool,
    pub decimal: bool,
    pub interrupt: bool,
    pub zero: bool,
    pub carry: bool,
}

impl Status {
    fn new() -> Self {
        Status {
            negative: false,
            overflow: false,
            decimal: false,
            interrupt: false,
            zero: false,
            carry: false,
        }
    }

    fn set(&mut self, value: u8) {
        self.negative  = value & 0x80 != 0;
        self.overflow  = value & 0x40 != 0;
        self.decimal   = value & 0x08 != 0;
        self.interrupt = value & 0x04 != 0;
        self.zero      = value & 0x02 != 0;
        self.carry     = value & 0x01 != 0;
    }

    fn set_zn(&mut self, value: u8) {
        self.zero = value == 0;
        self.negative = value & 0x80 != 0;
    }

    fn as_byte(&self) -> u8 {
        (self.negative as u8)  << 7 |
        (self.overflow as u8)  << 6 |
        (self.decimal as u8)   << 3 |
        (self.interrupt as u8) << 2 |
        (self.zero as u8)      << 1 |
        (self.carry as u8)
    }
}

pub struct Cpu {
    a:  u8,
    x:  u8,
    y:  u8,
    sp: u8,
    p: Status,
    pc: u16,
    mem: CpuMem,
    cycles: usize,
    nmi: bool,
    irq: bool,
    delayed_set_iflag: Option<bool>,
}


impl Mem for Cpu {
    fn read(&mut self, addr: u16) -> u8 {
        self.tick();
        self.mem.read(addr)
    }

    fn write(&mut self, addr: u16, value: u8) {
        self.tick();
        if addr == 0x4014 {
            self.do_dma(value);
        }
        else {
            self.mem.write(addr, value);
        }
    }
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            a:  0,
            x:  0,
            y:  0,
            sp: 0xfd,
            pc: 0,
            p:  Status::new(),
            mem: CpuMem::new(),
            cycles: 0,
            nmi: false,
            irq: false,
            delayed_set_iflag: None,
        }
    }

    #[inline(always)]
    fn tick(&mut self) {
        self.cycles += 1;
    }

    #[inline(always)]
    fn dummy_read(&mut self) {
        self.tick();
    }

    fn read_increment_pc(&mut self) -> u8 {
        let value = self.mem.read(self.pc);
        self.pc.wrapping_add(1);
        value
    }

    fn read16_increment_pc(&mut self) -> u16 {
        let lo = self.read_increment_pc() as u16;
        let hi = self.read_increment_pc() as u16;
        hi << 8 | lo
    }

    fn push(&mut self, value: u8) {
        self.mem.write(self.sp as u16 + 0x0100, value);
        self.sp.wrapping_sub(1);
    }

    fn push16(&mut self, value: u16) {
        self.push((value >> 8) as u8);
        self.push(value as u8);
    }

    fn pop(&mut self) -> u8 {
        self.sp.wrapping_add(1);
        self.mem.read(self.sp as u16 + 0x0100)
    }

    fn pop16(&mut self) -> u16 {
        let lo = self.pop() as u16;
        let hi = self.pop() as u16;
        hi << 8 | lo
    }

    fn branch(&mut self, addr: u16, condition: bool) {
        if condition {
            self.tick();
            if page_crossed(self.pc, addr) {
                self.tick();
            }
            self.pc = addr;
        }
    }

    fn do_dma(&mut self, value: u8) {
        if self.cycles % 2 != 0 {
            self.cycles += 2;
        }
        else {
            self.cycles += 1;
        }

        let base = (value as u16) << 8;

        for addr in base .. base + 0x00ff {
            let byte = self.mem.read(addr);
            self.mem.write(0x2004, byte)
        }
    }

    // XXX need rework on interrupts
    fn do_interrupt(&mut self, vector: u16, isbrk: bool) {
        self.push16(self.pc);
        if isbrk {
            self.push(self.p.as_byte() | 0x30);
        }
        else {
            self.push(self.p.as_byte() | 0x20);
        }
        self.pc = self.mem.read16(vector);
    }

    fn run_instruction(&mut self) {
        let opcode = self.read_increment_pc();
        INST[opcode as usize](self);

        //self.cycles += CYCLES[opcode as usize];
        //if self.cross_page {
        //    self.cycles += XPAGE_CYCLES[opcode as usize];
        //}
        //self.cross_page = false;
    }

    fn run(&mut self, upto: usize) {
        let start = self.cycles;

        while self.cycles - start < upto {
            self.run_instruction();

            // XXX need rework
            if self.nmi {
                self.do_interrupt(0xfffa, false);
                self.p.interrupt = true;
                self.nmi = false;
            }
            else if self.irq && !self.p.interrupt {
                self.do_interrupt(0xfffe, false);
                self.p.interrupt = true;
                self.irq = false;
            }

            if let Some(value) = self.delayed_set_iflag {
                self.p.interrupt = value;
            }
        }
    }
}

trait Mode {
    fn operand(&self, cpu: &mut Cpu) -> u8;

    fn writeback(&self, _cpu: &mut Cpu, _value: u8) {
        panic!("addressing mode never write back");
    }
}

struct Imm;
struct Acc;
struct FromMemory { addr: u16 }

impl Mode for Imm {
    fn operand(&self, cpu: &mut Cpu) -> u8 {
        cpu.read_increment_pc()
    }
}

impl Mode for Acc {
    fn operand(&self, cpu: &mut Cpu) -> u8 {
        cpu.a
    }

    fn writeback(&self, cpu: &mut Cpu, value: u8) {
        cpu.a = value;
    }
}

impl Mode for FromMemory {
    fn operand(&self, cpu: &mut Cpu) -> u8 {
        cpu.mem.read(self.addr)
    }

    fn writeback(&self, cpu: &mut Cpu, value: u8) {
        cpu.mem.write(self.addr, value);
    }
}

#[inline(always)]
fn imm(_cpu: &Cpu) -> Imm {
    Imm {}
}

#[inline(always)]
fn acc(_cpu: &Cpu) -> Acc {
    Acc {}
}

#[inline(always)]
fn zero(cpu: &mut Cpu) -> FromMemory {
    FromMemory {
        addr: cpu.read_increment_pc() as u16
    }
}

#[inline(always)]
fn zero_x(cpu: &mut Cpu) -> FromMemory {
    FromMemory {
        addr: cpu.read_increment_pc().wrapping_add(cpu.x) as u16
    }
}

#[inline(always)]
fn zero_y(cpu: &mut Cpu) -> FromMemory {
    FromMemory {
        addr: cpu.read_increment_pc().wrapping_add(cpu.y) as u16
    }
}

#[inline(always)]
fn abs(cpu: &mut Cpu) -> FromMemory {
    FromMemory {
        addr: cpu.read16_increment_pc()
    }
}

#[inline(always)]
fn abs_x(cpu: &mut Cpu) -> FromMemory {
    FromMemory {
        addr: cpu.read16_increment_pc().wrapping_add(cpu.x as u16)
    }
}

fn abs_x_check(cpu: &mut Cpu) -> FromMemory {
    let addr = cpu.read16_increment_pc().wrapping_add(cpu.x as u16);

    if page_crossed(cpu.pc, addr) {
        cpu.dummy_read();
    }

    FromMemory { addr }
}

#[inline(always)]
fn abs_y(cpu: &mut Cpu) -> FromMemory {
    FromMemory {
        addr: cpu.read16_increment_pc().wrapping_add(cpu.y as u16)
    }
}

fn abs_y_check(cpu: &mut Cpu) -> FromMemory {
    let addr = cpu.read16_increment_pc().wrapping_add(cpu.y as u16);

    if page_crossed(cpu.pc, addr) {
        cpu.dummy_read();
    }

    FromMemory { addr }
}

#[inline(always)]
fn indir(cpu: &mut Cpu) -> FromMemory {
    let at = cpu.read16_increment_pc();
    FromMemory {
        addr: cpu.mem.read16_wrapped(at)
    }
}

fn idx_indir(cpu: &mut Cpu) -> FromMemory {
    let at = cpu.read_increment_pc().wrapping_add(cpu.x);
    FromMemory {
        addr: cpu.mem.read16_wrapped(at as u16)
    }
}

fn indir_idx(cpu: &mut Cpu) -> FromMemory {
    let at = cpu.read_increment_pc();
    let at = cpu.mem.read16_wrapped(at as u16);
    FromMemory {
        addr: at.wrapping_add(cpu.y as u16)
    }
}

fn indir_idx_check(cpu: &mut Cpu) -> FromMemory {
    let at = cpu.read_increment_pc();
    let at = cpu.mem.read16_wrapped(at as u16);
    let addr = at.wrapping_add(cpu.y as u16);

    if page_crossed(at, addr) {
        cpu.dummy_read();
    }

    FromMemory { addr }
}

fn relative(cpu: &mut Cpu) -> FromMemory {
    let offset = cpu.read_increment_pc() as i16;

    FromMemory {
        addr: (cpu.pc as i16 + offset) as u16
    }
}


// INSTRUCTIONS

macro_rules! inst {
    ($fn:ident, $mode:ident) => {
        |cpu: &mut Cpu| {
            let mode = $mode(cpu);
            $fn(cpu, mode);
        }
    }
}

macro_rules! invalid {
    () => {
        |_cpu: &mut Cpu| { panic!("invalid 6502 instruction"); }
    }
}

fn adc<T: Mode>(cpu: &mut Cpu, mode: T) {
    let operand = mode.operand(cpu);
    let mut sum = operand as u16 + cpu.a as u16 + cpu.p.carry as u16;

    cpu.p.carry = sum > 0xff;

    let result = sum as u8;

    cpu.p.set_zn(result);
    cpu.p.overflow = ((cpu.a ^ operand) & 0x80 == 0) && ((cpu.a ^ result) & 0x80 != 0);
    cpu.a = result;
}

fn and<T: Mode>(cpu: &mut Cpu, mode: T) {
    let operand = mode.operand(cpu);
    let result = operand & cpu.a;
    cpu.p.set_zn(result);
    cpu.a = result;
}

fn asl<T: Mode>(cpu: &mut Cpu, mode: T) {
    let operand = mode.operand(cpu);
    cpu.p.carry = operand & 0x80 != 0;
    let result = operand << 1;
    cpu.p.set_zn(result);
    mode.writeback(cpu, result);
}

#[inline(always)]
fn bcc(cpu: &mut Cpu, mode: FromMemory) {
    cpu.branch(mode.addr, !cpu.p.carry);
}

#[inline(always)]
fn bcs(cpu: &mut Cpu, mode: FromMemory) {
    cpu.branch(mode.addr, cpu.p.carry);
}

#[inline(always)]
fn beq(cpu: &mut Cpu, mode: FromMemory) {
    cpu.branch(mode.addr, cpu.p.zero);
}

#[inline(always)]
fn bmi(cpu: &mut Cpu, mode: FromMemory) {
    cpu.branch(mode.addr, cpu.p.negative);
}

#[inline(always)]
fn bne(cpu: &mut Cpu, mode: FromMemory) {
    cpu.branch(mode.addr, !cpu.p.zero);
}

#[inline(always)]
fn bpl(cpu: &mut Cpu, mode: FromMemory) {
    cpu.branch(mode.addr, !cpu.p.negative);
}

#[inline(always)]
fn bvc(cpu: &mut Cpu, mode: FromMemory) {
    cpu.branch(mode.addr, !cpu.p.overflow);
}

#[inline(always)]
fn bvs(cpu: &mut Cpu, mode: FromMemory) {
    cpu.branch(mode.addr, cpu.p.overflow);
}

fn bit<T: Mode>(cpu: &mut Cpu, mode: T) {
    let operand = mode.operand(cpu);
    cpu.p.negative = operand & 0x80 != 0;
    cpu.p.overflow = operand & 0x40 != 0;
    cpu.p.zero = operand & cpu.a == 0;
}

fn cmp<T: Mode>(cpu: &mut Cpu, mode: T) {
    let operand = mode.operand(cpu);
    let result = cpu.a as i8 - operand as i8;
    cpu.p.carry = result >= 0;
    cpu.p.set_zn(result as u8);
}

fn cpx<T: Mode>(cpu: &mut Cpu, mode: T) {
    let operand = mode.operand(cpu);
    let result = cpu.x as i8 - operand as i8;
    cpu.p.carry = result >= 0;
    cpu.p.set_zn(result as u8);
}

fn cpy<T: Mode>(cpu: &mut Cpu, mode: T) {
    let operand = mode.operand(cpu);
    let result = cpu.y as i8 - operand as i8;
    cpu.p.carry = result >= 0;
    cpu.p.set_zn(result as u8);
}

fn dec<T: Mode>(cpu: &mut Cpu, mode: T) {
    let operand = mode.operand(cpu);
    let result = operand.wrapping_sub(1);
    cpu.p.set_zn(result);
    mode.writeback(cpu, result);
}

fn eor<T: Mode>(cpu: &mut Cpu, mode: T) {
    let operand = mode.operand(cpu);
    let result = cpu.a ^ operand;
    cpu.p.set_zn(result);
    cpu.a = result;
}

fn inc<T: Mode>(cpu: &mut Cpu, mode: T) {
    let operand = mode.operand(cpu);
    let result = operand.wrapping_add(1);
    cpu.p.set_zn(result);
    mode.writeback(cpu, result);
}

#[inline(always)]
fn jmp(cpu: &mut Cpu, mode: FromMemory) {
    cpu.pc = mode.addr;
}

fn jsr(cpu: &mut Cpu, mode: FromMemory) {
    let ret = cpu.pc - 1;
    cpu.push16(ret);
    cpu.pc = mode.addr;
}

fn lda<T: Mode>(cpu: &mut Cpu, mode: T) {
    let operand = mode.operand(cpu);
    cpu.p.set_zn(operand);
    cpu.a = operand;
}

fn ldx<T: Mode>(cpu: &mut Cpu, mode: T) {
    let operand = mode.operand(cpu);
    cpu.p.set_zn(operand);
    cpu.x = operand;
}

fn ldy<T: Mode>(cpu: &mut Cpu, mode: T) {
    let operand = mode.operand(cpu);
    cpu.p.set_zn(operand);
    cpu.y = operand;
}

fn lsr<T: Mode>(cpu: &mut Cpu, mode: T) {
    let operand = mode.operand(cpu);
    cpu.p.carry = operand & 0x1 != 0;
    let result = operand >> 1;
    cpu.p.set_zn(result);
    mode.writeback(cpu, result);
}

fn ora<T: Mode>(cpu: &mut Cpu, mode: T) {
    let operand = mode.operand(cpu);
    let result = operand | cpu.a;
    cpu.p.set_zn(result);
    cpu.a = result;
}

fn rol<T: Mode>(cpu: &mut Cpu, mode: T) {
    let operand = mode.operand(cpu);
    let mut shift = (operand as u16) << 1;
    shift |= cpu.p.carry as u16;
    cpu.p.carry = shift > 0xff;
    let result = shift as u8;
    cpu.p.set_zn(result);
    mode.writeback(cpu, result);
}

fn ror<T: Mode>(cpu: &mut Cpu, mode: T) {
    let operand = mode.operand(cpu);
    let mut shift = operand as u16;
    shift |= (cpu.p.carry as u16) << 8;
    cpu.p.carry = shift & 0x1 != 0;
    let result = (shift >> 1) as u8;
    cpu.p.set_zn(result);
    mode.writeback(cpu, result);
}

fn sbc<T: Mode>(cpu: &mut Cpu, mode: T) {
    let operand = mode.operand(cpu);
    let diff = cpu.a as u16 - operand as u16 - !cpu.p.carry as u16;

    cpu.p.carry = diff < 0x100;

    let result = diff as u8;

    cpu.p.set_zn(result);
    cpu.p.overflow = ((cpu.a ^ result) & 0x80 != 0) && ((cpu.a ^ operand) & 0x80 != 0);
    cpu.a = result;
}

#[inline(always)]
fn sta<T: Mode>(cpu: &mut Cpu, mode: T) {
    mode.writeback(cpu, cpu.a);
}

#[inline(always)]
fn stx<T: Mode>(cpu: &mut Cpu, mode: T) {
    mode.writeback(cpu, cpu.x);
}

#[inline(always)]
fn sty<T: Mode>(cpu: &mut Cpu, mode: T) {
    mode.writeback(cpu, cpu.y);
}

pub const INST: [fn(cpu: &mut Cpu) -> (); 256] = [
    /* 0x00 */
    |cpu: &mut Cpu| { // brk
        cpu.dummy_read();
        cpu.do_interrupt(0xfffe, true);
        cpu.p.interrupt = true;
    },
    /* 0x01 */ inst!(ora, idx_indir),
    /* 0x02 */ invalid!(),
    /* 0x03 */ invalid!(),
    /* 0x04 */ invalid!(),
    /* 0x05 */ inst!(ora, zero),
    /* 0x06 */ inst!(asl, zero),
    /* 0x07 */ invalid!(),
    /* 0x08 */
    |cpu: &mut Cpu| { // php
        cpu.dummy_read();
        cpu.push(cpu.p.as_byte() | 0x30);
    },
    /* 0x09 */ inst!(ora, imm),
    /* 0x0A */ inst!(asl, acc),
    /* 0x0B */ invalid!(),
    /* 0x0C */ invalid!(),
    /* 0x0D */ inst!(ora, abs),
    /* 0x0E */ inst!(asl, abs),
    /* 0x0F */ invalid!(),
    /* 0x10 */ inst!(bpl, relative),
    /* 0x11 */ inst!(ora, indir_idx_check),
    /* 0x12 */ invalid!(),
    /* 0x13 */ invalid!(),
    /* 0x14 */ invalid!(),
    /* 0x15 */ inst!(ora, zero_x),
    /* 0x16 */ inst!(asl, zero_x),
    /* 0x17 */ invalid!(),
    /* 0x18 */
    |cpu: &mut Cpu| { // clc
        cpu.dummy_read();
        cpu.p.carry = false;
    },
    /* 0x19 */ inst!(ora, abs_y_check),
    /* 0x1A */ invalid!(),
    /* 0x1B */ invalid!(),
    /* 0x1C */ invalid!(),
    /* 0x1D */ inst!(ora, abs_x_check),
    /* 0x1E */ inst!(asl, abs_x),
    /* 0x1F */ invalid!(),
    /* 0x20 */ inst!(jsr, abs),
    /* 0x21 */ inst!(and, idx_indir),
    /* 0x22 */ invalid!(),
    /* 0x23 */ invalid!(),
    /* 0x24 */ inst!(bit, zero),
    /* 0x25 */ inst!(and, zero),
    /* 0x26 */ inst!(rol, zero),
    /* 0x27 */ invalid!(),
    /* 0x28 */
    |cpu: &mut Cpu| { // plp
        cpu.dummy_read();

        let old = cpu.p.interrupt;

        let byte = cpu.pop();
        cpu.p.set(byte);

        let new = cpu.p.interrupt;

        cpu.p.interrupt = old;
        cpu.delayed_set_iflag = Some(new);
    },
    /* 0x29 */ inst!(and, imm),
    /* 0x2A */ inst!(rol, acc),
    /* 0x2B */ invalid!(),
    /* 0x2C */ inst!(bit, abs),
    /* 0x2D */ inst!(and, abs),
    /* 0x2E */ inst!(rol, abs),
    /* 0x2F */ invalid!(),
    /* 0x30 */ inst!(bmi, relative),
    /* 0x31 */ inst!(and, indir_idx_check),
    /* 0x32 */ invalid!(),
    /* 0x33 */ invalid!(),
    /* 0x34 */ invalid!(),
    /* 0x35 */ inst!(and, zero_x),
    /* 0x36 */ inst!(rol, zero_x),
    /* 0x37 */ invalid!(),
    /* 0x38 */
    |cpu: &mut Cpu| { // sec
        cpu.dummy_read();
        cpu.p.carry = true;
    },
    /* 0x39 */ inst!(and, abs_y_check),
    /* 0x3A */ invalid!(),
    /* 0x3B */ invalid!(),
    /* 0x3C */ invalid!(),
    /* 0x3D */ inst!(and, abs_x_check),
    /* 0x3E */ inst!(rol, abs_x),
    /* 0x3F */ invalid!(),
    /* 0x40 */
    |cpu: &mut Cpu| { // rti
        cpu.dummy_read();
        let byte = cpu.pop();
        cpu.p.set(byte);
        cpu.pc = cpu.pop16();
    },
    /* 0x41 */ inst!(eor, idx_indir),
    /* 0x42 */ invalid!(),
    /* 0x43 */ invalid!(),
    /* 0x44 */ invalid!(),
    /* 0x45 */ inst!(eor, zero),
    /* 0x46 */ inst!(lsr, zero),
    /* 0x47 */ invalid!(),
    /* 0x48 */
    |cpu: &mut Cpu| { // pha
        cpu.dummy_read();
        cpu.push(cpu.a);
    },
    /* 0x49 */ inst!(eor, imm),
    /* 0x4A */ inst!(lsr, acc),
    /* 0x4B */ invalid!(),
    /* 0x4C */ inst!(jmp, abs),
    /* 0x4D */ inst!(eor, abs),
    /* 0x4E */ inst!(lsr, abs),
    /* 0x4F */ invalid!(),
    /* 0x50 */ inst!(bvc, relative),
    /* 0x51 */ inst!(eor, indir_idx_check),
    /* 0x52 */ invalid!(),
    /* 0x53 */ invalid!(),
    /* 0x54 */ invalid!(),
    /* 0x55 */ inst!(eor, zero_x),
    /* 0x56 */ inst!(lsr, zero_x),
    /* 0x57 */ invalid!(),
    /* 0x58 */
    |cpu: &mut Cpu| { // cli
        cpu.dummy_read();
        cpu.delayed_set_iflag = Some(false);
    },
    /* 0x59 */ inst!(eor, abs_y_check),
    /* 0x5A */ invalid!(),
    /* 0x5B */ invalid!(),
    /* 0x5C */ invalid!(),
    /* 0x5D */ inst!(eor, abs_x_check),
    /* 0x5E */ inst!(lsr, abs_x),
    /* 0x5F */ invalid!(),
    /* 0x60 */
    |cpu: &mut Cpu| { // rts
        cpu.dummy_read();
        cpu.pc = cpu.pop16() + 1;
    },

    /* 0x61 */ inst!(adc, idx_indir),
    /* 0x62 */ invalid!(),
    /* 0x63 */ invalid!(),
    /* 0x64 */ invalid!(),
    /* 0x65 */ inst!(adc, zero),
    /* 0x66 */ inst!(ror, zero),
    /* 0x67 */ invalid!(),
    /* 0x68 */
    |cpu: &mut Cpu| { // pla
        cpu.dummy_read();
        cpu.a = cpu.pop();
    },
    /* 0x69 */ inst!(adc, imm),
    /* 0x6A */ inst!(ror, acc),
    /* 0x6B */ invalid!(),
    /* 0x6C */ inst!(jmp, indir),
    /* 0x6D */ inst!(adc, abs),
    /* 0x6E */ inst!(ror, abs),
    /* 0x6F */ invalid!(),
    /* 0x70 */ inst!(bvs, relative),
    /* 0x71 */ inst!(adc, indir_idx_check),
    /* 0x72 */ invalid!(),
    /* 0x73 */ invalid!(),
    /* 0x74 */ invalid!(),
    /* 0x75 */ inst!(adc, zero_x),
    /* 0x76 */ inst!(ror, zero_x),
    /* 0x77 */ invalid!(),
    /* 0x78 */
    |cpu: &mut Cpu| { // sei
        cpu.dummy_read();
        cpu.delayed_set_iflag = Some(true);
    },
    /* 0x79 */ inst!(adc, abs_y_check),
    /* 0x7A */ invalid!(),
    /* 0x7B */ invalid!(),
    /* 0x7C */ invalid!(),
    /* 0x7D */ inst!(adc, abs_x_check),
    /* 0x7E */ inst!(ror, abs_x),
    /* 0x7F */ invalid!(),
    /* 0x80 */ invalid!(),
    /* 0x81 */ inst!(sta, idx_indir),
    /* 0x82 */ invalid!(),
    /* 0x83 */ invalid!(),
    /* 0x84 */ inst!(sty, zero),
    /* 0x85 */ inst!(sta, zero),
    /* 0x86 */ inst!(stx, zero),
    /* 0x87 */ invalid!(),
    /* 0x88 */
    |cpu: &mut Cpu| { // dey
        cpu.dummy_read();

        let result = cpu.y.wrapping_sub(1);

        cpu.p.set_zn(result);
        cpu.y = result;
    },
    /* 0x89 */ invalid!(),
    /* 0x8A */
    |cpu: &mut Cpu| { // txa
        cpu.dummy_read();
        cpu.p.set_zn(cpu.x);
        cpu.a = cpu.x;
    },
    /* 0x8B */ invalid!(),
    /* 0x8C */ inst!(sty, abs),
    /* 0x8D */ inst!(sta, abs),
    /* 0x8E */ inst!(stx, abs),
    /* 0x8F */ invalid!(),
    /* 0x90 */ inst!(bcc, relative),
    /* 0x91 */ inst!(sta, indir_idx),
    /* 0x92 */ invalid!(),
    /* 0x93 */ invalid!(),
    /* 0x94 */ inst!(sty, zero_x),
    /* 0x95 */ inst!(sta, zero_x),
    /* 0x96 */ inst!(stx, zero_y),
    /* 0x97 */ invalid!(),
    /* 0x98 */
    |cpu: &mut Cpu| { // tya
        cpu.dummy_read();
        cpu.p.set_zn(cpu.y);
        cpu.a = cpu.y;
    },
    /* 0x99 */ inst!(sta, abs_y),
    /* 0x9A */
    |cpu: &mut Cpu| { // txs
        cpu.dummy_read();
        cpu.p.set_zn(cpu.x);
        cpu.sp = cpu.x;
    },
    /* 0x9B */ invalid!(),
    /* 0x9C */ invalid!(),
    /* 0x9D */ inst!(sta, abs_x),
    /* 0x9E */ invalid!(),
    /* 0x9F */ invalid!(),
    /* 0xA0 */ inst!(ldy, imm),
    /* 0xA1 */ inst!(lda, idx_indir),
    /* 0xA2 */ inst!(ldx, imm),
    /* 0xA3 */ invalid!(),
    /* 0xA4 */ inst!(ldy, zero),
    /* 0xA5 */ inst!(lda, zero),
    /* 0xA6 */ inst!(ldx, zero),
    /* 0xA7 */ invalid!(),
    /* 0xA8 */
    |cpu: &mut Cpu| { // tay
        cpu.dummy_read();
        cpu.p.set_zn(cpu.a);
        cpu.y = cpu.a;
    },
    /* 0xA9 */ inst!(lda, imm),
    /* 0xAA */
    |cpu: &mut Cpu| { // tax
        cpu.dummy_read();
        cpu.p.set_zn(cpu.a);
        cpu.x = cpu.a;
    },
    /* 0xAB */ invalid!(),
    /* 0xAC */ inst!(ldy, abs),
    /* 0xAD */ inst!(lda, abs),
    /* 0xAE */ inst!(ldx, abs),
    /* 0xAF */ invalid!(),
    /* 0xB0 */ inst!(bcs, relative),
    /* 0xB1 */ inst!(lda, indir_idx_check),
    /* 0xB2 */ invalid!(),
    /* 0xB3 */ invalid!(),
    /* 0xB4 */ inst!(ldy, zero_x),
    /* 0xB5 */ inst!(lda, zero_x),
    /* 0xB6 */ inst!(ldx, zero_y),
    /* 0xB7 */ invalid!(),
    /* 0xB8 */
    |cpu: &mut Cpu| { // clv
        cpu.dummy_read();
        cpu.p.overflow = false;
    },
    /* 0xB9 */ inst!(lda, abs_y_check),
    /* 0xBA */
    |cpu: &mut Cpu| { // tsx
        cpu.dummy_read();
        cpu.p.set_zn(cpu.sp);
        cpu.x = cpu.sp;
    },
    /* 0xBB */ invalid!(),
    /* 0xBC */ inst!(ldy, abs_x_check),
    /* 0xBD */ inst!(lda, abs_x_check),
    /* 0xBE */ inst!(ldx, abs_y_check),
    /* 0xBF */ invalid!(),
    /* 0xC0 */ inst!(cpy, imm),
    /* 0xC1 */ inst!(cmp, idx_indir),
    /* 0xC2 */ invalid!(),
    /* 0xC3 */ invalid!(),
    /* 0xC4 */ inst!(cpy, zero),
    /* 0xC5 */ inst!(cmp, zero),
    /* 0xC6 */ inst!(dec, zero),
    /* 0xC7 */ invalid!(),
    /* 0xC8 */
    |cpu: &mut Cpu| { // iny
        cpu.dummy_read();
        let result = cpu.y.wrapping_add(1);
        cpu.p.set_zn(result);
        cpu.y = result;
    },
    /* 0xC9 */ inst!(cmp, imm),
    /* 0xCA */
    |cpu: &mut Cpu| { // dex
        cpu.dummy_read();
        let result = cpu.x.wrapping_sub(1);
        cpu.p.set_zn(result);
        cpu.x = result;
    },
    /* 0xCB */ invalid!(),
    /* 0xCC */ inst!(cpy, abs),
    /* 0xCD */ inst!(cmp, abs),
    /* 0xCE */ inst!(dec, abs),
    /* 0xCF */ invalid!(),
    /* 0xD0 */ inst!(bne, relative),
    /* 0xD1 */ inst!(cmp, indir_idx_check),
    /* 0xD2 */ invalid!(),
    /* 0xD3 */ invalid!(),
    /* 0xD4 */ invalid!(),
    /* 0xD5 */ inst!(cmp, zero_x),
    /* 0xD6 */ inst!(dec, zero_x),
    /* 0xD7 */ invalid!(),
    /* 0xD8 */
    |cpu: &mut Cpu| { // cld
        cpu.dummy_read();
        cpu.p.decimal = false;
    },
    /* 0xD9 */ inst!(cmp, abs_y_check),
    /* 0xDA */ invalid!(),
    /* 0xDB */ invalid!(),
    /* 0xDC */ invalid!(),
    /* 0xDD */ inst!(cmp, abs_x_check),
    /* 0xDE */ inst!(dec, abs_x),
    /* 0xDF */ invalid!(),
    /* 0xE0 */ inst!(cpx, imm),
    /* 0xE1 */ inst!(sbc, idx_indir),
    /* 0xE2 */ invalid!(),
    /* 0xE3 */ invalid!(),
    /* 0xE4 */ inst!(cpx, zero),
    /* 0xE5 */ inst!(sbc, zero),
    /* 0xE6 */ inst!(inc, zero),
    /* 0xE7 */ invalid!(),
    /* 0xE8 */
    |cpu: &mut Cpu| { // inx
        cpu.dummy_read();
        let result = cpu.x.wrapping_add(1);
        cpu.p.set_zn(result);
        cpu.x = result;
    },
    /* 0xE9 */ inst!(sbc, imm),
    /* 0xEA */
    |cpu: &mut Cpu| { // nop
        cpu.dummy_read();
    },
    /* 0xEB */ invalid!(),
    /* 0xEC */ inst!(cpx, abs),
    /* 0xED */ inst!(sbc, abs),
    /* 0xEE */ inst!(inc, abs),
    /* 0xEF */ invalid!(),
    /* 0xF0 */ inst!(beq, relative),
    /* 0xF1 */ inst!(sbc, indir_idx_check),
    /* 0xF2 */ invalid!(),
    /* 0xF3 */ invalid!(),
    /* 0xF4 */ invalid!(),
    /* 0xF5 */ inst!(sbc, zero_x),
    /* 0xF6 */ inst!(inc, zero_x),
    /* 0xF7 */ invalid!(),
    /* 0xF8 */
    |cpu: &mut Cpu| { // sed
        cpu.dummy_read();
        cpu.p.decimal = true;
    },
    /* 0xF9 */ inst!(sbc, abs_y_check),
    /* 0xFA */ invalid!(),
    /* 0xFB */ invalid!(),
    /* 0xFC */ invalid!(),
    /* 0xFD */ inst!(sbc, abs_x_check),
    /* 0xFE */ inst!(inc, abs_x),
    /* 0xFF */ invalid!(),
];

fn page_crossed(addr1: u16, addr2: u16) -> bool {
    addr1 & 0xff00 != addr2 & 0xff00
}
