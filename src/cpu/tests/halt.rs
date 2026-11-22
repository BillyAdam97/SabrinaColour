use super::seeded_cpu;
use crate::bus::{Bus, IE, IF};

#[test]
fn halt_stays_asleep_without_an_enabled_interrupt_request() {
    let mut cpu = seeded_cpu(0x8123, 0);
    let mut bus = Bus::new();
    bus.write(0x8123, 0x76); // HALT
    bus.write(0x8124, 0x3C); // INC A
    bus.write(IE, 0x01);
    bus.write(IF, 0x02);

    let mut expected = seeded_cpu(0x8124, 0);
    expected.halted = true;
    cpu.step(&mut bus);
    assert_eq!(cpu, expected);

    cpu.step(&mut bus);
    assert_eq!(cpu, expected);
    assert_eq!(bus.read(IF), 0x02);
}

#[test]
fn halt_wakes_and_resumes_execution_with_ime_disabled() {
    let mut cpu = seeded_cpu(0x8123, 0);
    let mut bus = Bus::new();
    bus.write(0x8123, 0x76); // HALT
    bus.write(0x8124, 0x3C); // INC A
    bus.write(IE, 0x01);

    cpu.step(&mut bus);
    assert!(cpu.halted);

    bus.write(IF, 0x01);
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x8125);
    assert_eq!(cpu.a, 0xA6);
    assert_eq!(cpu.sp, 0xD135);
    assert!(!cpu.halted);
    assert!(!cpu.ime);
    assert_eq!(bus.read(IF), 0x01);
}

#[test]
fn halt_wakes_and_services_an_interrupt_with_ime_enabled() {
    let mut cpu = seeded_cpu(0x8123, 0);
    let mut bus = Bus::new();
    cpu.ime = true;
    bus.write(0x8123, 0x76); // HALT
    bus.write(0x8124, 0x3C); // INC A
    bus.write(IE, 0x01);

    cpu.step(&mut bus);
    assert!(cpu.halted);

    bus.write(IF, 0x01);
    cpu.step(&mut bus);

    let mut expected = seeded_cpu(0x0040, 0);
    expected.sp = 0xD133;
    assert_eq!(cpu, expected);
    assert_eq!(bus.read(0xD133), 0x24);
    assert_eq!(bus.read(0xD134), 0x81);
    assert_eq!(bus.read(IF), 0x00);
}

#[test]
fn halt_bug_suppresses_the_next_opcode_fetch_increment() {
    let mut cpu = seeded_cpu(0x8123, 0);
    let mut bus = Bus::new();
    bus.write(0x8123, 0x76); // HALT
    bus.write(0x8124, 0x3E); // LD A, imm8
    bus.write(0x8125, 0x42);
    bus.write(IE, 0x01);
    bus.write(IF, 0x01);

    cpu.step(&mut bus);
    assert!(!cpu.halted);
    assert_eq!(cpu.pc, 0x8124);

    cpu.step(&mut bus);
    assert_eq!(cpu.a, 0x3E);
    assert_eq!(cpu.pc, 0x8125);
    assert_eq!(bus.read(IF), 0x01);

    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x8126);
}

#[test]
fn ei_then_halt_with_a_pending_interrupt_returns_to_halt() {
    let mut cpu = seeded_cpu(0x8123, 0);
    let mut bus = Bus::new();
    bus.write(0x8123, 0xFB); // EI
    bus.write(0x8124, 0x76); // HALT
    bus.write(0x8125, 0x3C); // INC A
    bus.write(0x0040, 0xD9); // RETI
    bus.write(IE, 0x01);
    bus.write(IF, 0x01);

    cpu.step(&mut bus); // EI
    cpu.step(&mut bus); // HALT
    cpu.step(&mut bus); // Interrupt entry
    assert_eq!(cpu.pc, 0x0040);
    assert_eq!(bus.read(0xD133), 0x24);
    assert_eq!(bus.read(0xD134), 0x81);
    assert_eq!(bus.read(IF), 0x00);

    cpu.step(&mut bus); // RETI
    assert_eq!(cpu.pc, 0x8124);
    assert_eq!(cpu.sp, 0xD135);

    cpu.step(&mut bus); // HALT again
    let mut expected = seeded_cpu(0x8125, 0);
    expected.halted = true;
    expected.ime = true;
    assert_eq!(cpu, expected);
}
