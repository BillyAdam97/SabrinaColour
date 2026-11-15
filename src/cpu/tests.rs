mod arithmetic;
mod jumps;
mod loads;

use super::{Cpu, Flags};
use crate::bus::Bus;

// Distinct values expose accidental writes to unrelated registers.
fn seeded_cpu(pc: u16, flags: u8) -> Cpu {
    Cpu {
        a: 0xA5,
        f: Flags::from_bits_retain(flags << 4),
        b: 0xC0,
        c: 0x10,
        d: 0xC0,
        e: 0x20,
        h: 0xC0,
        l: 0x30,
        sp: 0xD135,
        pc,
    }
}

#[test]
#[should_panic(expected = "not yet implemented: Implement")]
fn halt() {
    let mut cpu = Cpu::new();
    let mut bus = Bus::new();
    bus.write(0, 0x76);

    cpu.step(&mut bus);
}

#[test]
#[should_panic(expected = "Unknown opcode")]
fn unknown_opcode() {
    let mut cpu = Cpu::new();
    let mut bus = Bus::new();
    bus.write(0, 0xD3);

    cpu.step(&mut bus);
}
