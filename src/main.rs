extern crate redwhite;

use redwhite::cpu::{Cpu, Flags};

fn main() {
    let mut cpu = Cpu::new();
    println!("{:x}", cpu.flags_as_byte());
    cpu.set_flag(Flags::V | Flags::N);
    println!("{:x}", cpu.flags_as_byte());
}
