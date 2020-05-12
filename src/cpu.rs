use memory::{Access, CpuMem};
//use std::ops::Deref;

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
    //check_cross_page: bool,
    nmi: bool,
    irq: bool,
    delayed_set_iflag: Option<bool>,
}


impl Access for Cpu {
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
            //check_cross_page: false,
            nmi: false,
            irq: false,
            delayed_set_iflag: None,
        }
    }

    fn tick(&mut self) {
        self.cycles += 1;
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

    // XXX
    fn jump(&mut self, addr: u16, condition: bool) {
        if condition {
            self.cycles += 1;
            self.check_cross_page = page_crossed(self.pc, addr);
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
            self.mem.write(0x2004, self.mem.read(addr))
        }
    }

    // XXX need rework
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

    fn dispatch(&mut self) {
        let opcode = self.read_at_pc();

        //match opcode {
        //    0x00 => { // brk
        //        self.do_interrupt(0xfffe, true);
        //        self.p.interrupt = true;
        //    }

        //    0x69 => inst!(self, adc, immediate),
        //    0x65 => inst!(self, adc, zeropage),
        //    0x75 => inst!(self, adc, zeropage_x),
        //    0x6d => inst!(self, adc, absolute),
        //    0x7d => inst!(self, adc, absolute_x),
        //    0x79 => inst!(self, adc, absolute_y),
        //    0x61 => inst!(self, adc, indexed_indirect),
        //    0x71 => inst!(self, adc, indirect_indexed),

        //    0x29 => inst!(self, and, immediate),
        //    0x25 => inst!(self, and, zeropage),
        //    0x35 => inst!(self, and, zeropage_x),
        //    0x2d => inst!(self, and, absolute),
        //    0x3d => inst!(self, and, absolute_x),
        //    0x39 => inst!(self, and, absolute_y),
        //    0x21 => inst!(self, and, indexed_indirect),
        //    0x31 => inst!(self, and, indirect_indexed),

        //    0x0a => inst!(self, asl, accumulator),
        //    0x06 => inst!(self, asl, zeropage),
        //    0x16 => inst!(self, asl, zeropage_x),
        //    0x0e => inst!(self, asl, absolute),
        //    0x1e => inst!(self, asl, absolute_x),

        //    0x90 => inst!(self, bcc, relative),
        //    0xb0 => inst!(self, bcs, relative),
        //    0xf0 => inst!(self, beq, relative),
        //    0x30 => inst!(self, bmi, relative),
        //    0xd0 => inst!(self, bne, relative),
        //    0x10 => inst!(self, bpl, relative),
        //    0x50 => inst!(self, bvc, relative),
        //    0x70 => inst!(self, bvs, relative),

        //    0x24 => inst!(self, bit, zeropage),
        //    0x2c => inst!(self, bit, absolute),

        //    0x18 => self.p.carry = false,     // clc
        //    0xd8 => self.p.decimal = false,   // cld
        //    0xb8 => self.p.overflow = false,  // clv
        //    0x58 => self.delayed_set_iflag = Some(false), // cli

        //    0xc9 => inst!(self, cmp, immediate),
        //    0xc5 => inst!(self, cmp, zeropage),
        //    0xd5 => inst!(self, cmp, zeropage_x),
        //    0xcd => inst!(self, cmp, absolute),
        //    0xdd => inst!(self, cmp, absolute_x),
        //    0xd9 => inst!(self, cmp, absolute_y),
        //    0xc1 => inst!(self, cmp, indexed_indirect),
        //    0xd1 => inst!(self, cmp, indirect_indexed),

        //    0xe0 => inst!(self, cpx, immediate),
        //    0xe4 => inst!(self, cpx, zeropage),
        //    0xec => inst!(self, cpx, absolute),

        //    0xc0 => inst!(self, cpy, immediate),
        //    0xc4 => inst!(self, cpy, zeropage),
        //    0xcc => inst!(self, cpy, absolute),

        //    0xc6 => inst!(self, dec, zeropage),
        //    0xd6 => inst!(self, dec, zeropage_x),
        //    0xce => inst!(self, dec, absolute),
        //    0xde => inst!(self, dec, absolute_x),

        //    0xca => { // dex
        //        let result = self.x.wrapping_sub(1);
        //        self.update_zero_negative(result);
        //        self.x = result;
        //    }

        //    0x88 => { // dey
        //        let result = self.y.wrapping_sub(1);
        //        self.update_zero_negative(result);
        //        self.y = result;
        //    }

        //    0x49 => inst!(self, eor, immediate),
        //    0x45 => inst!(self, eor, zeropage),
        //    0x55 => inst!(self, eor, zeropage_x),
        //    0x4d => inst!(self, eor, absolute),
        //    0x5d => inst!(self, eor, absolute_x),
        //    0x59 => inst!(self, eor, absolute_y),
        //    0x41 => inst!(self, eor, indexed_indirect),
        //    0x51 => inst!(self, eor, indirect_indexed),

        //    0xe6 => inst!(self, inc, zeropage),
        //    0xf6 => inst!(self, inc, zeropage_x),
        //    0xee => inst!(self, inc, absolute),
        //    0xfe => inst!(self, inc, absolute_x),

        //    0xe8 => { // inx
        //        let result = self.x.wrapping_add(1);
        //        self.update_zero_negative(result);
        //        self.x = result;
        //    }

        //    0xc8 => { // iny
        //        let result = self.y.wrapping_add(1);
        //        self.update_zero_negative(result);
        //        self.y = result;
        //    }

        //    0x4c => inst!(self, jmp, absolute),
        //    0x6c => inst!(self, jmp, indirect),
    
        //    0x20 => inst!(self, jsr, absolute),
    
        //    0xa9 => inst!(self, lda, immediate),
        //    0xa5 => inst!(self, lda, zeropage),
        //    0xb5 => inst!(self, lda, zeropage_x),
        //    0xad => inst!(self, lda, absolute),
        //    0xbd => inst!(self, lda, absolute_x),
        //    0xb9 => inst!(self, lda, absolute_y),
        //    0xa1 => inst!(self, lda, indexed_indirect),
        //    0xb1 => inst!(self, lda, indirect_indexed),

        //    0xa2 => inst!(self, ldx, immediate),
        //    0xa6 => inst!(self, ldx, zeropage),
        //    0xb6 => inst!(self, ldx, zeropage_y),
        //    0xae => inst!(self, ldx, absolute),
        //    0xbe => inst!(self, ldx, absolute_y),

        //    0xa0 => inst!(self, ldy, immediate),
        //    0xa4 => inst!(self, ldy, zeropage),
        //    0xb4 => inst!(self, ldy, zeropage_x),
        //    0xac => inst!(self, ldy, absolute),
        //    0xbc => inst!(self, ldy, absolute_x),

        //    0x4a => inst!(self, lsr, accumulator),
        //    0x46 => inst!(self, lsr, zeropage),
        //    0x56 => inst!(self, lsr, zeropage_x),
        //    0x4e => inst!(self, lsr, absolute),
        //    0x5e => inst!(self, lsr, absolute_x),

        //    0xea => (), // nop

        //    0x09 => inst!(self, ora, immediate),
        //    0x05 => inst!(self, ora, zeropage),
        //    0x15 => inst!(self, ora, zeropage_x),
        //    0x0d => inst!(self, ora, absolute),
        //    0x1d => inst!(self, ora, absolute_x),
        //    0x19 => inst!(self, ora, absolute_y),
        //    0x01 => inst!(self, ora, indexed_indirect),
        //    0x11 => inst!(self, ora, indirect_indexed),

        //    0x48 => { // pha
        //        let a = self.a;
        //        self.push(a);
        //    }

        //    0x08 => self.push(self.p.as_byte() | 0x30), // php

        //    0x68 => self.a = self.pop(), // pla

        //    0x28 => { // plp
        //        let old = self.p.interrupt;
        //        self.p = Status::from_byte(self.pop());
        //        let new = self.p.interrupt;
        //        self.p.interrupt = old;
        //        self.delayed_set_iflag = Some(new);
        //    }

        //    0x2a => inst!(self, rol, accumulator),
        //    0x26 => inst!(self, rol, zeropage),
        //    0x36 => inst!(self, rol, zeropage_x),
        //    0x2e => inst!(self, rol, absolute),
        //    0x3e => inst!(self, rol, absolute_x),

        //    0x6a => inst!(self, ror, accumulator),
        //    0x66 => inst!(self, ror, zeropage),
        //    0x76 => inst!(self, ror, zeropage_x),
        //    0x6e => inst!(self, ror, absolute),
        //    0x7e => inst!(self, ror, absolute_x),

        //    0x60 => self.pc = self.pop16() + 1, // rts

        //    0x40 => { // rti
        //        self.p = Status::from_byte(self.pop());
        //        self.pc = self.pop16();
        //    }

        //    0xe9 => inst!(self, sbc, immediate),
        //    0xe5 => inst!(self, sbc, zeropage),
        //    0xf5 => inst!(self, sbc, zeropage_x),
        //    0xed => inst!(self, sbc, absolute),
        //    0xfd => inst!(self, sbc, absolute_x),
        //    0xf9 => inst!(self, sbc, absolute_y),
        //    0xe1 => inst!(self, sbc, indexed_indirect),
        //    0xf1 => inst!(self, sbc, indirect_indexed),

        //    0x38 => self.p.carry = true,     // sec
        //    0xf8 => self.p.decimal = true,   // sed
        //    0x78 => self.delayed_set_iflag = Some(true), // sei

        //    0x85 => inst!(self, sta, zeropage),
        //    0x95 => inst!(self, sta, zeropage_x),
        //    0x8d => inst!(self, sta, absolute),
        //    0x9d => inst!(self, sta, absolute_x),
        //    0x99 => inst!(self, sta, absolute_y),
        //    0x81 => inst!(self, sta, indexed_indirect),
        //    0x91 => inst!(self, sta, indirect_indexed),

        //    0x86 => inst!(self, stx, zeropage),
        //    0x96 => inst!(self, stx, zeropage_y),
        //    0x8e => inst!(self, stx, absolute),

        //    0x84 => inst!(self, sty, zeropage),
        //    0x94 => inst!(self, sty, zeropage_x),
        //    0x8c => inst!(self, sty, absolute),

        //    0xaa => { // tax
        //        let a = self.a;
        //        self.update_zero_negative(a);
        //        self.x = a;
        //    }

        //    0xa8 => { // tay
        //        let a = self.a;
        //        self.update_zero_negative(a);
        //        self.y = a;
        //    }

        //    0xba => { // tsx
        //        let sp = self.sp;
        //        self.update_zero_negative(sp);
        //        self.x = sp;
        //    }

        //    0x8a => { // txa
        //        let x = self.x;
        //        self.update_zero_negative(x);
        //        self.a = x;
        //    }

        //    0x9a => { // txs
        //        let x = self.x;
        //        self.update_zero_negative(x);
        //        self.sp = x;
        //    }

        //    0x98 => { // tya
        //        let y = self.y;
        //        self.update_zero_negative(y);
        //        self.a = y;
        //    }

        //    _ => panic!("unknown opcode {} pc={:x}", opcode, self.pc)
        //}

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
            self.dispatch();

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

// XXXXXXXXXXXXXXXXXXXXXXX

trait AddressingMode {
    fn get_operand(&self, cpu: &mut Cpu) -> u8;
    fn writeback(&self, _cpu: &mut Cpu, _value: u8) {}
}

struct Implied;
struct Immediate;
struct Accumulator;

struct FromMemory {
    addr: u16,
    crosspage: bool,
}

impl AddressingMode for Immediate {
    fn get_operand(&self, cpu: &mut Cpu) -> u8 {
        cpu.read_at_pc()
    }
}

impl AddressingMode for Accumulator {
    fn get_operand(&self, cpu: &mut Cpu) -> u8 {
        cpu.a
    }

    fn write_back(&self, cpu: &mut Cpu, value: u8) {
        cpu.a = value;
    }
}

impl AddressingMode for FromMemory {
    fn get_operand(&self, cpu: &mut Cpu) -> u8 {
        cpu.mem.read(self.addr)
    }

    fn write_back(&self, cpu: &mut Cpu, value: u8) {
        cpu.mem.write(self.addr, value);
    }
}

fn implied(_cpu: &Cpu) -> Implied {
    Implied {}
}

fn immediate(_cpu: &Cpu) -> Immediate {
    Immediate {}
}

fn accumulator(_cpu: &Cpu) -> Accumulator {
    Accumulator {}
}

fn zeropage(cpu: &mut Cpu) -> FromMemory {
    FromMemory {
        addr: cpu.read_at_pc() as u16,
        crosspage: false,
    }
}

fn zeropage_x(cpu: &mut Cpu) -> FromMemory {
    FromMemory {
        addr: cpu.read_at_pc().wrapping_add(cpu.x) as u16,
        crosspage: false,
    }
}

fn zeropage_y(cpu: &mut Cpu) -> FromMemory {
    FromMemory {
        addr: cpu.read_at_pc().wrapping_add(cpu.y) as u16,
        crosspage: false,
    }
}

fn absolute(cpu: &mut Cpu) -> FromMemory {
    FromMemory {
        addr: cpu.read16_at_pc(),
        crosspage: false,
    }
}

fn absolute_x(cpu: &mut Cpu) -> FromMemory {
    let addr = cpu.read16_at_pc().wrapping_add(cpu.x as u16);

    FromMemory {
        addr,
        crosspage: page_crossed(cpu.pc, addr),
    }
}

fn absolute_y(cpu: &mut Cpu) -> FromMemory {
    let addr = self.read16_at_pc().wrapping_add(self.y as u16);

    FromMemory {
        addr,
        crosspage: page_crossed(cpu.pc, addr),
    }
}

fn indirect(cpu: &mut Cpu) -> FromMemory {
    FromMemory {
        addr: cpu.mem.read16_wrapped(cpu.read16_at_pc()),
        crosspage: false,
    }
}

fn indexed_indirect(cpu: &mut Cpu) -> FromMemory {
    let addr = cpu.read_at_pc().wrapping_add(cpu.x);

    FromMemory {
        addr: cpu.mem.read16_wrapped(addr as u16),
        crosspage: false,
    }
}

fn indirect_indexed(cpu: &mut Cpu) -> FromMemory {
    let addr1 = cpu.mem.read16_wrapped(cpu.read_at_pc() as u16);
    let addr2 = addr1.wrapping_add(cpu.y as u16);

    FromMemory {
        addr,
        crosspage: page_crossed(addr1, addr2),
    }
}

fn relative(cpu: &mut Cpu) -> FromMemory {
    let offset = cpu.read_at_pc();
    let addr = ((cpu.pc as i16) + (offset as i16)) as u16;

    FromMemory {
        addr,
        crosspage: false,
    }
}


// INSTRUCTIONS

macro_rules! inst {
    ($fn:ident, $mode:ident, $cycles:expr, $extra:expr) => {{
        |cpu: &mut Cpu| {
            let mode = $mode();
            $fn(cpu, mode);
            cpu.cycles += $cycles;
            if cpu.check_cross_page {
                cpu.cycles += $extra;
                cpu.check_cross_page = false;
            }
        }
    }}
}

macro_rules! invalid {
    () => {{
        |_cpu: &mut Cpu| { panic!("invalid 6502 instruction"); }
    }}
}

#[inline(always)]
fn nop(_cpu: &mut Cpu, _mode: Implied) {
}

#[inline(always)]
fn tax(cpu: &mut Cpu, _mode: Implied) {
    cpu.update_zero_negative(cpu.a);
    cpu.x = cpu.a;
}

#[inline(always)]
fn tay(cpu: &mut Cpu, _mode: Implied) {
    cpu.update_zero_negative(cpu.a);
    cpu.y = cpu.a;
}

#[inline(always)]
fn tsx(cpu: &mut Cpu, _mode: Implied) {
    cpu.update_zero_negative(cpu.sp);
    cpu.x = cpu.sp;
}

#[inline(always)]
fn txa(cpu: &mut Cpu, _mode: Implied) {
    cpu.update_zero_negative(cpu.x);
    cpu.a = cpu.x;
}

#[inline(always)]
fn txs(cpu: &mut Cpu, _mode: Implied) {
    cpu.update_zero_negative(cpu.x);
    cpu.sp = cpu.x;
}

#[inline(always)]
fn tya(cpu: &mut Cpu, _mode: Implied) {
    cpu.update_zero_negative(cpu.y);
    cpu.a = cpu.y;
}

#[inline(always)]]
fn sec(cpu: &mut Cpu, _mode: Implied) {
    cpu.p.carry = true;
}

#[inline(always)]]
fn sed(cpu: &mut Cpu, _mode: Implied) {
    cpu.p.decimal = true;
}

#[inline(always)]]
fn sei(cpu: &mut Cpu, _mode: Implied) {
    cpu.delayed_set_iflag = Some(true);
}

#[inline(always)]
fn rti(cpu: &mut Cput, _mode: Implied) {
    cpu.p = Status::from_byte(cpu.pop());
    cpu.pc = cpu.pop16();
}

#[inline(always)]
fn rts(cpu: &mut Cput, _mode: Implied) {
    cpu.pc = cpu.pop16() + 1;
}

#[inline(always)]
fn pha(cpu: &mut Cpu, _mode: Implied) {
    cpu.push(cpu.a);
}

#[inline(always)]
fn php(cpu: &mut Cpu, _mode: Implied) {
    cpu.push(cpu.p.as_byte() | 0x30);
}

#[inline(always)]
fn pla(cpu: &mut Cpu, _mode: Implied) {
    cpu.a = cpu.pop();
}

#[inline(always)]
fn plp(cpu: &mut Cpu, _mode: Implied) {
    let old = cpu.p.interrupt;
    cpu.p = Status::from_byte(cpu.pop());
    let new = cpu.p.interrupt;
    cpu.p.interrupt = old;
    cpu.delayed_set_iflag = Some(new);
}

#[inline(always)]
fn brk(cpu: &mut Cpu, _mode: Implied) {
    cpu.do_interrupt(0xfffe, true);
    cpu.p.interrupt = true;
}

#[inline(always)]
fn dex(cpu: &mut Cpu, _mode: Implied) {
    let result = cpu.x.wrapping_sub(1);

    cpu.update_zero_negative(result);
    cpu.x = result;
}

#[inline(always)]
fn dey(cpu: &mut Cpu, _mode: Implied) {
    let result = cpu.y.wrapping_sub(1);

    cpu.update_zero_negative(result);
    cpu.y = result;
}

#[inline(always)]
fn inx(cpu: &mut Cpu, _mode: Implied) {
    let result = cpu.x.wrapping_add(1);

    cpu.update_zero_negative(result);
    cpu.x = result;
}

#[inline(always)]
fn iny(cpu: &mut Cpu, _mode: Implied) {
    let result = cpu.y.wrapping_add(1);

    cpu.update_zero_negative(result);
    cpu.y = result;
}

#[inline(always)]
fn clc(cpu: &mut Cpu, _mode: Implied) {
    cpu.p.carry = false;
}

#[inline(always)]
fn cld(cpu: &mut Cpu, _mode: Implied) {
    cpu.p.decimal = false;
}

#[inline(always)]
fn clv(cpu: &mut Cpu, _mode: Implied) {
    cpu.p.overflow = false;
}

#[inline(always)]
fn cli(cpu: &mut Cpu, _mode: Implied) {
    cpu.delayed_set_iflag = Some(false);
}

fn adc<T: AddressingMode>(cpu: &mut Cpu, mode: T) {
    let operand = mode.address(cpu);
    let mut sum = operand as u16 +
                  cpu.a as u16 +
                  cpu.p.carry as u16;
    cpu.p.carry = sum > 0xff;
    let result = sum as u8;
    cpu.update_zero_negative(result);
    let a = cpu.a;
    let cond = ((a ^ operand) & 0x80 == 0) && ((a ^ result) & 0x80 != 0);
    cpu.p.overflow = cond;
    cpu.a = result;
}

fn and<T: AddressingMode>(&mut cpu, mode: T) {
    let operand = mode.address(cpu);
    let result = operand & cpu.a;
    cpu.update_zero_negative(result);
    cpu.a = result;
}

fn asl<T: AddressingMode>(&mut cpu, mode: T) {
    let operand = mode.address(cpu);
    cpu.p.carry = operand & 0x80 != 0;
    let result = operand << 1;
    cpu.update_zero_negative(result);
    mode.writeback(cpu, result);
}

fn bcc(&mut cpu, mode: FromMemory) {
    cpu.jump(mode.addr, !cpu.p.carry);
}

fn bcs(&mut cpu, mode: FromMemory) {
    cpu.jump(mode.addr, cpu.p.carry);
}

fn beq(&mut cpu, mode: FromMemory) {
    cpu.jump(mode.addr, cpu.p.zero);
}

fn bmi(&mut cpu, mode: FromMemory) {
    cpu.jump(mode.addr, cpu.p.negative);
}

fn bne(&mut cpu, mode: FromMemory) {
    cpu.jump(mode.addr, !cpu.p.zero);
}

fn bpl(&mut cpu, mode: FromMemory) {
    cpu.jump(mode.addr, !cpu.p.negative);
}

fn bvc(&mut cpu, mode: FromMemory) {
    cpu.jump(mode.addr, !cpu.p.overflow);
}

fn bvs(&mut cpu, mode: FromMemory) {
    cpu.jump(mode.addr, cpu.p.overflow);
}

fn bit<T: AddressingMode>(&mut cpu, mode: T) {
    let operand = mode.address(cpu);
    cpu.p.negative = operand & 0x80 != 0;
    cpu.p.overflow = operand & 0x40 != 0;
    cpu.p.zero = operand & cpu.a == 0;
}

fn cmp<T: AddressingMode>(&mut cpu, mode: T) {
    let operand = mode.address(cpu);
    let result = cpu.a as i8 - operand as i8;
    cpu.p.carry = result >= 0;
    cpu.update_zero_negative(result as u8);
}

fn cpx<T: AddressingMode>(&mut cpu, mode: T) {
    let operand = mode.address(cpu);
    let result = cpu.x as i8 - operand as i8;
    cpu.p.carry = result >= 0;
    cpu.update_zero_negative(result as u8);
}

fn cpy<T: AddressingMode>(&mut cpu, mode: T) {
    let operand = mode.address(cpu);
    let result = cpu.y as i8 - operand as i8;
    cpu.p.carry = result >= 0;
    cpu.update_zero_negative(result as u8);
}

fn dec<T: AddressingMode>(&mut cpu, mode: T) {
    let operand = mode.address(cpu);
    let result = operand.wrapping_sub(1);
    cpu.update_zero_negative(result);
    mode.writeback(cpu, result);
}

fn eor<T: AddressingMode>(&mut cpu, mode: T) {
    let operand = mode.address(cpu);
    let result = cpu.a ^ operand;
    cpu.update_zero_negative(result);
    cpu.a = result;
}

fn inc<T: AddressingMode>(&mut cpu, mode: T) {
    let operand = mode.address(cpu);
    let result = operand.wrapping_add(1);
    cpu.update_zero_negative(result);
    mode.writeback(cpu, result);
}

fn jmp(&mut cpu, mode: FromMemory) {
    cpu.pc = mode.addr;
}

fn jsr(&mut cpu, mode: FromMemory) {
    let ret = cpu.pc - 1;
    cpu.push16(ret);
    cpu.pc = mode.addr;
}

fn lda<T: AddressingMode>(&mut cpu, mode: T) {
    let operand = mode.address(cpu);
    cpu.update_zero_negative(operand);
    cpu.a = operand;
}

fn ldx<T: AddressingMode>(&mut cpu, mode: T) {
    let operand = mode.address(cpu);
    cpu.update_zero_negative(operand);
    cpu.x = operand;
}

fn ldy<T: AddressingMode>(&mut cpu, mode: T) {
    let operand = mode.address(cpu);
    cpu.update_zero_negative(operand);
    cpu.y = operand;
}

fn lsr<T: AddressingMode>(&mut cpu, mode: T) {
    let operand = mode.address(cpu);
    cpu.p.carry = operand & 0x1 != 0;
    let result = operand >> 1;
    cpu.update_zero_negative(result);
    mode.writeback(cpu, result);
}

fn ora<T: AddressingMode>(&mut cpu, mode: T) {
    let operand = mode.address(cpu);
    let result = operand | cpu.a;
    cpu.update_zero_negative(result);
    cpu.a = result;
}

fn rol<T: AddressingMode>(&mut cpu, mode: T) {
    let operand = mode.address(cpu);
    let mut shift = (operand as u16) << 1;
    shift |= cpu.p.carry as u16;
    cpu.p.carry = shift > 0xff;
    let result = shift as u8;
    cpu.update_zero_negative(result);
    mode.writeback(cpu, result);
}

fn ror<T: AddressingMode>(&mut cpu, mode: T) {
    let operand = mode.address(cpu);
    let mut shift = operand as u16;
    shift |= (cpu.p.carry as u16) << 8;
    cpu.p.carry = shift & 0x1 != 0;
    let result = (shift >> 1) as u8;
    cpu.update_zero_negative(result);
    mode.writeback(cpu, result);
}

fn sbc<T: AddressingMode>(&mut cpu, mode: T) {
    let operand = mode.address(cpu);
    let diff = cpu.a as u16 -
               operand as u16 -
               !cpu.p.carry as u16;
    cpu.p.carry = diff < 0x100;
    let result = diff as u8;
    cpu.update_zero_negative(result);
    let a = cpu.a;
    let cond = ((a ^ result) & 0x80 != 0) && ((a ^ operand) & 0x80 != 0);
    cpu.p.overflow = cond;
    cpu.a = result;
}

fn sta<T: AddressingMode>(&mut cpu, mode: T) {
    let a = cpu.a;
    mode.writeback(cpu, a);
}

fn stx<T: AddressingMode>(&mut cpu, mode: T) {
    let x = cpu.x;
    mode.writeback(cpu, x);
}

fn sty<T: AddressingMode>(&mut cpu, mode: T) {
    let y = cpu.y;
    mode.writeback(cpu, y);
}

pub const INST: [fn(cpu: &mut Cpu) -> (); 256] = [
    /* 0x00 */ inst!(brk, implied, 7, 0),
    /* 0x01 */ inst!(ora, indexed_indirect, 6, 0),
    /* 0x02 */ invalid!(),
    /* 0x03 */ invalid!(),
    /* 0x04 */ invalid!(),
    /* 0x05 */ inst!(ora, zeropage, 3, 0),
    /* 0x06 */ inst!(asl, zeropage, 5, 0),
    /* 0x07 */ invalid!(),
    /* 0x08 */ inst!(php, implied, 3, 0),
    /* 0x09 */ inst!(ora, immediate, 2, 0),
    /* 0x0A */ inst!(asl, accumulator, 2, 0),
    /* 0x0B */ invalid!(),
    /* 0x0C */ invalid!(),
    /* 0x0D */ inst!(ora, absolute, 4, 0),
    /* 0x0E */ inst!(asl, absolute, 6, 0),
    /* 0x0F */ invalid!(),
    /* 0x10 */ inst!(bpl, relative, 2, 0),
    /* 0x11 */ inst!(ora, indirect_indexed, 5, 1),
    /* 0x12 */ invalid!(),
    /* 0x13 */ invalid!(),
    /* 0x14 */ invalid!(),
    /* 0x15 */ inst!(ora, zeropage_x, 4, 0),
    /* 0x16 */ inst!(asl, zeropage_x, 6, 0),
    /* 0x17 */ invalid!(),
    /* 0x18 */ inst!(clc, implied, 2, 0),
    /* 0x19 */ inst!(ora, absolute_y, 4, 1),
    /* 0x1A */ invalid!(),
    /* 0x1B */ invalid!(),
    /* 0x1C */ invalid!(),
    /* 0x1D */ inst!(ora, absolute_x, 4, 1),
    /* 0x1E */ inst!(asl, absolute_x, 7, 0),
    /* 0x1F */ invalid!(),
    /* 0x20 */ inst!(jsr, absolute, 6, 0),
    /* 0x21 */ inst!(and, indexed_indirect, 6, 0),
    /* 0x22 */ invalid!(),
    /* 0x23 */ invalid!(),
    /* 0x24 */ inst!(bit, zeropage, 3, 0),
    /* 0x25 */ inst!(and, zeropage, 3, 0),
    /* 0x26 */ inst!(rol, zeropage, 5, 0),
    /* 0x27 */ invalid!(),
    /* 0x28 */ inst!(plp, implied, 4, 0),
    /* 0x29 */ inst!(and, immediate 2, 0),
    /* 0x2A */ inst!(rol, accumulator, 2, 0),
    /* 0x2B */ invalid!(),
    /* 0x2C */ inst!(bit, absolute, 4, 0),
    /* 0x2D */ inst!(and, absolute, 4, 0),
    /* 0x2E */ inst!(rol, absolute, 6, 0),
    /* 0x2F */ invalid!(),
    /* 0x30 */ inst!(bmi, relative, 2, 0),
    /* 0x31 */ inst!(and, indirect_indexed, 5, 1),
    /* 0x32 */ invalid!(),
    /* 0x33 */ invalid!(),
    /* 0x34 */ invalid!(),
    /* 0x35 */ inst!(and, zeropage_x, 4, 0),
    /* 0x36 */ inst!(rol, zeropage_x, 6, 0),
    /* 0x37 */ invalid!(),
    /* 0x38 */ inst!(sec, implied, 2, 0),
    /* 0x39 */ inst!(and, absolute_y, 4, 1),
    /* 0x3A */ invalid!(),
    /* 0x3B */ invalid!(),
    /* 0x3C */ invalid!(),
    /* 0x3D */ inst!(and, absolute_x, 4, 1),
    /* 0x3E */ inst!(rol, absolute_x, 7, 0),
    /* 0x3F */ invalid!(),
    /* 0x40 */ inst!(rti, implied, 6, 0),
    /* 0x41 */ inst!(eor, indexed_indirect, 6, 0),
    /* 0x42 */ invalid!(),
    /* 0x43 */ invalid!(),
    /* 0x44 */ invalid!(),
    /* 0x45 */ inst!(eor, zeropage, 3, 0),
    /* 0x46 */ inst!(lsr, zeropage, 5, 0),
    /* 0x47 */ invalid!(),
    /* 0x48 */ inst!(pha, implied, 3, 0),
    /* 0x49 */ inst!(eor, immediate, 2, 0),
    /* 0x4A */ inst!(lsr, accumulator, 2, 0),
    /* 0x4B */ invalid!(),
    /* 0x4C */ inst!(jmp, absolute, 3, 0),
    /* 0x4D */ inst!(eor, absolute, 4, 0),
    /* 0x4E */ inst!(lsr, absolute, 6, 0),
    /* 0x4F */ invalid!(),
    /* 0x50 */ inst!(bvc, relative, 2, 0),
    /* 0x51 */ inst!(eor, indirect_indexed, 5, 1),
    /* 0x52 */ invalid!(),
    /* 0x53 */ invalid!(),
    /* 0x54 */ invalid!(),
    /* 0x55 */ inst!(eor, zeropage_x, 4, 0),
    /* 0x56 */ inst!(lsr, zeropage_x, 6, 0),
    /* 0x57 */ invalid!(),
    /* 0x58 */ inst!(cli, implied, 2, 0),
    /* 0x59 */ inst!(eor, absolute_y, 4, 1),
    /* 0x5A */ invalid!(),
    /* 0x5B */ invalid!(),
    /* 0x5C */ invalid!(),
    /* 0x5D */ inst!(eor, absolute_x, 4, 1),
    /* 0x5E */ inst!(lsr, absolute_x, 7, 0),
    /* 0x5F */ invalid!(),
    /* 0x60 */ inst!(rts, implied, 6, 0),
    /* 0x61 */ inst!(adc, indexed_indirect, 6, 0),
    /* 0x62 */ invalid!(),
    /* 0x63 */ invalid!(),
    /* 0x64 */ invalid!(),
    /* 0x65 */ inst!(adc, zeropage, 3, 0),
    /* 0x66 */ inst!(ror, zeropage, 5, 0),
    /* 0x67 */ invalid!(),
    /* 0x68 */ inst!(pla, implied, 4, 0),
    /* 0x69 */ inst!(adc, immediate, 2, 0),
    /* 0x6A */ inst!(ror, accumulator, 2, 0),
    /* 0x6B */ invalid!(),
    /* 0x6C */ inst!(jmp, indirect, 5, 0),
    /* 0x6D */ inst!(adc, absolute, 4, 0),
    /* 0x6E */ inst!(ror, absolute, 6, 0),
    /* 0x6F */ invalid!(),
    /* 0x70 */ inst!(bvs, relative, 2, 0),
    /* 0x71 */ inst!(adc, indirect_indexed, 5, 1),
    /* 0x72 */ invalid!(),
    /* 0x73 */ invalid!(),
    /* 0x74 */ invalid!(),
    /* 0x75 */ inst!(adc, zeropage_x, 4, 0),
    /* 0x76 */ inst!(ror, zeropage_x, 6, 0),
    /* 0x77 */ invalid!(),
    /* 0x78 */ inst!(sei, implied, 2, 0),
    /* 0x79 */ inst!(adc, absolute_y, 4, 1),
    /* 0x7A */ invalid!(),
    /* 0x7B */ invalid!(),
    /* 0x7C */ invalid!(),
    /* 0x7D */ inst!(adc, absolute_x, 4, 1),
    /* 0x7E */ inst!(ror, absolute_x, 7, 0),
    /* 0x7F */ invalid!(),
    /* 0x80 */ invalid!(),
    /* 0x81 */ inst!(sta, indexed_indirect, 6, 0),
    /* 0x82 */ invalid!(),
    /* 0x83 */ invalid!(),
    /* 0x84 */ inst!(sty, zeropage, 3, 0),
    /* 0x85 */ inst!(sta, zeropage, 3, 0),
    /* 0x86 */ inst!(stx, zeropage, 3, 0),
    /* 0x87 */ invalid!(),
    /* 0x88 */ inst!(dey, implied, 2, 0),
    /* 0x89 */ invalid!(),
    /* 0x8A */ inst!(txa, implied, 2, 0),
    /* 0x8B */ invalid!(),
    /* 0x8C */ inst!(sty, absolute, 4, 0),
    /* 0x8D */ inst!(sta, absolute, 4, 0),
    /* 0x8E */ inst!(stx, absolute, 4, 0),
    /* 0x8F */ invalid!(),
    /* 0x90 */ inst!(bcc, relative, 2, 0),
    /* 0x91 */ inst!(sta, indirect_indexed, 6, 0),
    /* 0x92 */ invalid!(),
    /* 0x93 */ invalid!(),
    /* 0x94 */ inst!(sty, zeropage_x, 4, 0),
    /* 0x95 */ inst!(sta, zeropage_x, 4, 0),
    /* 0x96 */ inst!(stx, zeropage_y, 4, 0),
    /* 0x97 */ invalid!(),
    /* 0x98 */ inst!(tya, implied, 2, 0),
    /* 0x99 */ inst!(sta, absolute_y, 5, 0),
    /* 0x9A */ inst!(txs, implied, 2, 0),
    /* 0x9B */ invalid!(),
    /* 0x9C */ invalid!(),
    /* 0x9D */ inst!(sta, absolute_x, 5, 0),
    /* 0x9E */ invalid!(),
    /* 0x9F */ invalid!(),
    /* 0xA0 */ inst!(ldy, immediate, 2, 0),
    /* 0xA1 */ inst!(lda, indexed_indirect, 6, 0),
    /* 0xA2 */ inst!(ldx, immediate, 2, 0),
    /* 0xA3 */ invalid!(),
    /* 0xA4 */ inst!(ldy, zeropage, 3, 0),
    /* 0xA5 */ inst!(lda, zeropage, 3, 0),
    /* 0xA6 */ inst!(ldx, zeropage, 3, 0),
    /* 0xA7 */ invalid!(),
    /* 0xA8 */ inst!(tay, implied, 2, 0),
    /* 0xA9 */ inst!(lda, immediate, 2, 0),
    /* 0xAA */ inst!(tax, implied, 2, 0),
    /* 0xAB */ invalid!(),
    /* 0xAC */ inst!(ldy, absolute, 4, 0),
    /* 0xAD */ inst!(lda, absolute, 4, 0),
    /* 0xAE */ inst!(ldx, absolute, 4, 0),
    /* 0xAF */ invalid!(),
    /* 0xB0 */ inst!(bcs, relative, 2, 0),
    /* 0xB1 */ inst!(lda, indirect_indexed, 5, 1),
    /* 0xB2 */ invalid!(),
    /* 0xB3 */ invalid!(),
    /* 0xB4 */ inst!(ldy, zeropage_x, 4, 0),
    /* 0xB5 */ inst!(lda, zeropage_x, 4, 0),
    /* 0xB6 */ inst!(ldx, zeropage_y, 4, 0),
    /* 0xB7 */ invalid!(),
    /* 0xB8 */ inst!(clv, implied, 2, 0),
    /* 0xB9 */ inst!(lda, absolute_y, 4, 1),
    /* 0xBA */ inst!(tsx, implied, 2, 0),
    /* 0xBB */ invalid!(),
    /* 0xBC */ inst!(ldy, absolute_x, 4, 1),
    /* 0xBD */ inst!(lda, absolute_x, 4, 1),
    /* 0xBE */ inst!(ldx, absolute_y, 4, 1),
    /* 0xBF */ invalid!(),
    /* 0xC0 */ inst!(cpy, immediate, 2, 0),
    /* 0xC1 */ inst!(cmp, indexed_indirect, 6, 0),
    /* 0xC2 */ invalid!(),
    /* 0xC3 */ invalid!(),
    /* 0xC4 */ inst!(cpy, zeropage, 3, 0),
    /* 0xC5 */ inst!(cmp, zeropage, 3, 0),
    /* 0xC6 */ inst!(dec, zeropage, 5, 0),
    /* 0xC7 */ invalid!(),
    /* 0xC8 */ inst!(iny, implied, 2, 0),
    /* 0xC9 */ inst!(cmp, immediate, 2, 0),
    /* 0xCA */ inst!(dex, implied, 2, 0),
    /* 0xCB */ invalid!(),
    /* 0xCC */ inst!(cpy, absolute, 4, 0),
    /* 0xCD */ inst!(cmp, absolute, 4, 0),
    /* 0xCE */ inst!(dec, absolute, 6, 0),
    /* 0xCF */ invalid!(),
    /* 0xD0 */ inst!(bne, relative, 2, 0),
    /* 0xD1 */ inst!(cmp, indirect_indexed, 5, 1),
    /* 0xD2 */ invalid!(),
    /* 0xD3 */ invalid!(),
    /* 0xD4 */ invalid!(),
    /* 0xD5 */ inst!(cmp, zeropage_x, 4, 0),
    /* 0xD6 */ inst!(dec, zeropage_x, 6, 0),
    /* 0xD7 */ invalid!(),
    /* 0xD8 */ inst!(cld, implied, 2, 0),
    /* 0xD9 */ inst!(cmp, absolute_y, 4, 1),
    /* 0xDA */ invalid!(),
    /* 0xDB */ invalid!(),
    /* 0xDC */ invalid!(),
    /* 0xDD */ inst!(cmp, absolute_x, 4, 1),
    /* 0xDE */ inst!(dec, absolute_x, 7, 0),
    /* 0xDF */ invalid!(),
    /* 0xE0 */ inst!(cpx, immediate, 2, 0),
    /* 0xE1 */ inst!(sbc, indexed_indirect, 6, 0),
    /* 0xE2 */ invalid!(),
    /* 0xE3 */ invalid!(),
    /* 0xE4 */ inst!(cpx, zeropage, 3, 0),
    /* 0xE5 */ inst!(sbc, zeropage, 3, 0),
    /* 0xE6 */ inst!(inc, zeropage, 5, 0),
    /* 0xE7 */ invalid!(),
    /* 0xE8 */ inst!(inx, implied, 2, 0),
    /* 0xE9 */ inst!(sbc, immediate, 2, 0),
    /* 0xEA */ inst!(nop, implied, 2, 0),
    /* 0xEB */ invalid!(),
    /* 0xEC */ inst!(cpx, absolute, 4, 0),
    /* 0xED */ inst!(sbc, absolute, 4, 0),
    /* 0xEE */ inst!(inc, absolute, 6, 0),
    /* 0xEF */ invalid!(),
    /* 0xF0 */ inst!(beq, relative, 2, 0),
    /* 0xF1 */ inst!(sbc, indirect_indexed, 5, 1),
    /* 0xF2 */ invalid!(),
    /* 0xF3 */ invalid!(),
    /* 0xF4 */ invalid!(),
    /* 0xF5 */ inst!(sbc, zeropage_x, 4, 0),
    /* 0xF6 */ inst!(inc, zeropage_x, 6, 0),
    /* 0xF7 */ invalid!(),
    /* 0xF8 */ inst!(sed, implied, 2, 0),
    /* 0xF9 */ inst!(sbc, absolute_y, 4, 1),
    /* 0xFA */ invalid!(),
    /* 0xFB */ invalid!(),
    /* 0xFC */ invalid!(),
    /* 0xFD */ inst!(sbc, absolute_x, 4, 1),
    /* 0xFE */ inst!(inc, absolute_x, 7, 0),
    /* 0xFF */ invalid!(),
];
