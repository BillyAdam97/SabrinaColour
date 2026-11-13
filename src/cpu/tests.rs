use super::{Cpu, Flags};
use crate::bus::Bus;

#[test]
fn ld_r16_imm16() {
    for (opcode, destination) in [(0x01, 0), (0x11, 1), (0x21, 2), (0x31, 3)] {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();
        bus.write(0, opcode);
        bus.write(1, 0x34);
        bus.write(2, 0x12);

        cpu.step(&mut bus);

        let mut expected = [0; 4];
        expected[destination] = 0x1234;
        let registers = [
            u16::from_be_bytes([cpu.b, cpu.c]),
            u16::from_be_bytes([cpu.d, cpu.e]),
            u16::from_be_bytes([cpu.h, cpu.l]),
            cpu.sp,
        ];
        assert_eq!(registers, expected, "opcode {opcode:02X}");
        assert_eq!(cpu.pc, 3, "opcode {opcode:02X}");
    }
}

#[test]
fn ld_r16mem_a() {
    for (opcode, address, expected_hl) in [
        (0x02, 0xC010, 0xC030),
        (0x12, 0xC020, 0xC030),
        (0x22, 0xC030, 0xC031),
        (0x32, 0xC030, 0xC02F),
    ] {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();
        cpu.a = 0xA5;
        (cpu.b, cpu.c) = (0xC0, 0x10);
        (cpu.d, cpu.e) = (0xC0, 0x20);
        (cpu.h, cpu.l) = (0xC0, 0x30);
        bus.write(0, opcode);

        cpu.step(&mut bus);

        assert_eq!(bus.read(address), 0xA5, "opcode {opcode:02X}");
        assert_eq!(
            u16::from_be_bytes([cpu.h, cpu.l]),
            expected_hl,
            "opcode {opcode:02X}"
        );
        assert_eq!(cpu.pc, 1, "opcode {opcode:02X}");
    }
}

#[test]
fn ld_r8_imm8() {
    let opcodes = [0x06, 0x0E, 0x16, 0x1E, 0x26, 0x2E, 0x36, 0x3E];
    for (destination, opcode) in opcodes.into_iter().enumerate() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();
        cpu.h = 0xC0;
        bus.write(0, opcode);
        bus.write(1, 0xA5);

        cpu.step(&mut bus);

        let mut expected = [0, 0, 0, 0, 0xC0, 0, 0, 0];
        expected[destination] = 0xA5;
        let values = [
            cpu.b,
            cpu.c,
            cpu.d,
            cpu.e,
            cpu.h,
            cpu.l,
            bus.read(0xC000),
            cpu.a,
        ];
        assert_eq!(values, expected, "opcode {opcode:02X}");
        assert_eq!(cpu.pc, 2, "opcode {opcode:02X}");
    }
}

#[test]
fn ld_absolute_from_sp_stores_low_byte_first() {
    let mut cpu = Cpu::new();
    let mut bus = Bus::new();
    cpu.sp = 0xD135;
    bus.write(0, 0x08);
    bus.write(1, 0x34);
    bus.write(2, 0xC2);

    cpu.step(&mut bus);

    assert_eq!(bus.read(0xC234), 0x35);
    assert_eq!(bus.read(0xC235), 0xD1);
    assert_eq!(cpu.sp, 0xD135);
    assert_eq!(cpu.pc, 3);
}

#[test]
fn ld_a_r16mem() {
    for (opcode, expected_a, expected_hl) in [
        (0x0A, 0x11, 0xC030),
        (0x1A, 0x22, 0xC030),
        (0x2A, 0x33, 0xC031),
        (0x3A, 0x33, 0xC02F),
    ] {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();
        (cpu.b, cpu.c) = (0xC0, 0x10);
        (cpu.d, cpu.e) = (0xC0, 0x20);
        (cpu.h, cpu.l) = (0xC0, 0x30);
        bus.write(0, opcode);
        bus.write(0xC010, 0x11);
        bus.write(0xC020, 0x22);
        bus.write(0xC030, 0x33);

        cpu.step(&mut bus);

        assert_eq!(cpu.a, expected_a, "opcode {opcode:02X}");
        assert_eq!(
            u16::from_be_bytes([cpu.h, cpu.l]),
            expected_hl,
            "opcode {opcode:02X}"
        );
        assert_eq!(cpu.pc, 1, "opcode {opcode:02X}");
    }
}

#[test]
fn ld_r8_r8() {
    // Operand order in the opcode is B, C, D, E, H, L, [HL], A.
    let initial = [0x11, 0x22, 0x33, 0x44, 0xC0, 0x55, 0x66, 0x77];
    for destination in 0..8 {
        for source in 0..8 {
            let opcode = 0x40 + (destination * 8 + source) as u8;
            if opcode == 0x76 {
                continue; // HALT is still unimplemented.
            }

            let mut cpu = Cpu::new();
            let mut bus = Bus::new();
            [cpu.b, cpu.c, cpu.d, cpu.e, cpu.h, cpu.l, _, cpu.a] = initial;
            bus.write(0xC055, initial[6]);
            bus.write(0, opcode);

            cpu.step(&mut bus);

            let mut expected = initial;
            expected[destination] = initial[source];
            let values = [
                cpu.b,
                cpu.c,
                cpu.d,
                cpu.e,
                cpu.h,
                cpu.l,
                bus.read(0xC055),
                cpu.a,
            ];
            assert_eq!(values, expected, "opcode {opcode:02X}");
            assert_eq!(cpu.pc, 1, "opcode {opcode:02X}");
        }
    }
}

#[test]
fn ldh_imm8_from_a() {
    let mut cpu = Cpu::new();
    let mut bus = Bus::new();
    cpu.a = 0xA5;
    bus.write(0, 0xE0);
    bus.write(1, 0x80);

    cpu.step(&mut bus);

    assert_eq!(bus.read(0xFF80), 0xA5);
    assert_eq!(cpu.pc, 2);
}

#[test]
fn ldh_c_from_a() {
    let mut cpu = Cpu::new();
    let mut bus = Bus::new();
    cpu.a = 0xA5;
    cpu.c = 0x80;
    bus.write(0, 0xE2);

    cpu.step(&mut bus);

    assert_eq!(bus.read(0xFF80), 0xA5);
    assert_eq!(cpu.pc, 1);
}

#[test]
fn ld_absolute_from_a() {
    let mut cpu = Cpu::new();
    let mut bus = Bus::new();
    cpu.a = 0x12;
    bus.write(0, 0xEA);
    bus.write(1, 0x67);
    bus.write(2, 0xC1);

    cpu.step(&mut bus);

    assert_eq!(bus.read(0xC167), 0x12);
    assert_eq!(cpu.pc, 3);
}

#[test]
fn ldh_a_from_imm8() {
    let mut cpu = Cpu::new();
    let mut bus = Bus::new();
    bus.write(0, 0xF0);
    bus.write(1, 0x80);
    bus.write(0xFF80, 0xA5);

    cpu.step(&mut bus);

    assert_eq!(cpu.a, 0xA5);
    assert_eq!(cpu.pc, 2);
}

#[test]
fn ldh_a_from_c() {
    let mut cpu = Cpu::new();
    let mut bus = Bus::new();
    cpu.c = 0x80;
    bus.write(0, 0xF2);
    bus.write(0xFF80, 0xA5);

    cpu.step(&mut bus);

    assert_eq!(cpu.a, 0xA5);
    assert_eq!(cpu.pc, 1);
}

#[test]
fn ld_hl_sp_imm8() {
    for (sp, offset, initial_flags, expected_hl, expected_flags) in [
        (
            0xD0FF,
            0x01,
            Flags::ZERO | Flags::SUB,
            0xD100,
            Flags::HALF | Flags::CARRY,
        ),
        (0xD000, 0xFF, Flags::all(), 0xCFFF, Flags::empty()),
    ] {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();
        cpu.sp = sp;
        cpu.f = initial_flags;
        bus.write(0, 0xF8);
        bus.write(1, offset);

        cpu.step(&mut bus);

        assert_eq!(
            u16::from_be_bytes([cpu.h, cpu.l]),
            expected_hl,
            "offset {offset:02X}"
        );
        assert_eq!(cpu.f, expected_flags, "offset {offset:02X}");
        assert_eq!(cpu.sp, sp, "offset {offset:02X}");
        assert_eq!(cpu.pc, 2, "offset {offset:02X}");
    }
}

#[test]
fn ld_sp_hl() {
    let mut cpu = Cpu::new();
    let mut bus = Bus::new();
    (cpu.h, cpu.l) = (0xC0, 0x55);
    bus.write(0, 0xF9);

    cpu.step(&mut bus);

    assert_eq!(cpu.sp, 0xC055);
    assert_eq!(cpu.pc, 1);
}

#[test]
fn ld_a_from_absolute() {
    let mut cpu = Cpu::new();
    let mut bus = Bus::new();
    bus.write(0, 0xFA);
    bus.write(1, 0x67);
    bus.write(2, 0xC1);
    bus.write(0xC167, 0xA5);

    cpu.step(&mut bus);

    assert_eq!(cpu.a, 0xA5);
    assert_eq!(cpu.pc, 3);
}

#[test]
#[should_panic(expected = "Unknown opcode")]
fn unknown_opcode() {
    let mut cpu = Cpu::new();
    let mut bus = Bus::new();
    bus.write(0, 0xD3);

    cpu.step(&mut bus);
}
