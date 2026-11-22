mod arithmetic;
mod flags;
mod halt;
mod interrupts;
mod jumps;
mod loads;
mod registers;
mod stack;

use super::Cpu;
use crate::bus::Bus;
use crate::cpu::registers::Flags;

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
        halted: false,
        halt_bug: false,
        ime: false,
        ime_pending: false,
    }
}

#[test]
fn nop_advances_pc_and_preserves_cpu_state() {
    for pc in [0x8123u16, 0xFFFF] {
        let mut cpu = seeded_cpu(pc, 0xF);
        let mut bus = Bus::new();
        let next_pc = pc.wrapping_add(1);
        bus.write(pc, 0x00);
        bus.write(next_pc, 0x3C); // INC A would change the state if executed.

        cpu.step(&mut bus);

        assert_eq!(cpu, seeded_cpu(next_pc, 0xF), "PC={pc:04X}");
        assert_eq!(bus.read(next_pc), 0x3C);
    }
}

#[test]
#[should_panic(expected = "Unknown opcode")]
fn unknown_opcode() {
    let mut cpu = Cpu::new();
    let mut bus = Bus::new();
    bus.write(0, 0xD3);

    cpu.step(&mut bus);
}
