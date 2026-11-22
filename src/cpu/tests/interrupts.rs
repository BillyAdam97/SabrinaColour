use super::seeded_cpu;
use crate::bus::{Bus, IE, IF};

#[test]
fn interrupt_dispatch_services_the_highest_priority_enabled_request() {
    for (enabled, requested, vector, remaining_requests) in [
        (0x07, 0x07, 0x0040, 0x06),
        (0x1E, 0x1F, 0x0048, 0x1D),
        (0x14, 0xFC, 0x0050, 0xF8),
        (0x18, 0x18, 0x0058, 0x10),
        (0x10, 0x10, 0x0060, 0x00),
    ] {
        let mut cpu = seeded_cpu(0x8123, 0xF);
        let mut bus = Bus::new();
        cpu.ime = true;
        bus.write(IE, enabled);
        bus.write(IF, requested);

        cpu.step(&mut bus);

        let case = format!("IE={enabled:02X}, IF={requested:02X}");
        let mut expected = seeded_cpu(vector, 0xF);
        expected.sp = 0xD133;
        assert_eq!(bus.read(0xD133), 0x23, "{case}");
        assert_eq!(bus.read(0xD134), 0x81, "{case}");
        assert!(!cpu.ime, "{case}");
        assert_eq!(bus.read(IE), enabled, "{case}");
        assert_eq!(bus.read(IF), remaining_requests, "{case}");
        assert_eq!(cpu, expected, "{case}");
    }
}

#[test]
fn interrupt_entry_leaves_the_handler_instruction_for_the_next_step() {
    let mut cpu = seeded_cpu(0x8123, 0);
    let mut bus = Bus::new();
    cpu.ime = true;
    bus.write(IE, 0x01);
    bus.write(IF, 0x01);
    // Give every vector the same observable instruction to isolate the step boundary.
    for vector in [0x0040, 0x0048, 0x0050, 0x0058, 0x0060] {
        bus.write(vector, 0x3C); // INC A
    }

    cpu.step(&mut bus);
    assert_eq!(cpu.a, 0xA5);

    cpu.step(&mut bus);
    assert_eq!(cpu.a, 0xA6);
}

#[test]
fn ei_enables_interrupts_after_the_following_instruction() {
    let mut cpu = seeded_cpu(0x8123, 0);
    let mut bus = Bus::new();
    bus.write(0x8123, 0xFB); // EI
    bus.write(0x8124, 0x3C); // INC A
    bus.write(IE, 0x01);
    bus.write(IF, 0x01);

    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x8124);
    assert!(!cpu.ime);

    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x8125);
    assert_eq!(cpu.a, 0xA6);
    assert!(cpu.ime);
    assert_eq!(cpu.sp, 0xD135);
    assert_eq!(bus.read(IF), 0x01);
}

#[test]
fn di_cancels_a_preceding_ei() {
    let mut cpu = seeded_cpu(0x8123, 0);
    let mut bus = Bus::new();
    bus.write(0x8123, 0xFB); // EI
    bus.write(0x8124, 0xF3); // DI
    bus.write(0x8125, 0x3C); // INC A
    bus.write(IE, 0x01);
    bus.write(IF, 0x01);

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x8125);
    assert!(!cpu.ime);

    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x8126);
    assert_eq!(cpu.a, 0xA6);
    assert!(!cpu.ime);
    assert_eq!(cpu.sp, 0xD135);
    assert_eq!(bus.read(IF), 0x01);
}

#[test]
fn di_disables_interrupts_immediately() {
    let mut cpu = seeded_cpu(0x8123, 0);
    let mut bus = Bus::new();
    cpu.ime = true;
    bus.write(0x8123, 0xF3); // DI
    bus.write(0x8124, 0x3C); // INC A
    bus.write(IE, 0x01);

    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x8124);
    assert!(!cpu.ime);

    bus.write(IF, 0x01);
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x8125);
    assert_eq!(cpu.a, 0xA6);
    assert_eq!(cpu.sp, 0xD135);
    assert_eq!(bus.read(IF), 0x01);
}

#[test]
fn reti_pops_the_return_address_and_enables_interrupts_immediately() {
    let mut cpu = seeded_cpu(0x8123, 0xF);
    let mut bus = Bus::new();
    bus.write(0x8123, 0xD9);
    bus.write(0xD135, 0x46);
    bus.write(0xD136, 0xC1);
    bus.write(IE, 0x01);
    bus.write(IF, 0x01);

    cpu.step(&mut bus);

    let mut expected = seeded_cpu(0xC146, 0xF);
    expected.sp = 0xD137;
    expected.ime = true;
    assert_eq!(cpu, expected);
    assert_eq!(bus.read(0xD135), 0x46);
    assert_eq!(bus.read(0xD136), 0xC1);
    assert_eq!(bus.read(IF), 0x01);
}
