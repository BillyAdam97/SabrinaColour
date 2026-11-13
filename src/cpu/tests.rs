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
fn ld_r16_imm16() {
    for (opcode, destination) in [(0x01, 0), (0x11, 1), (0x21, 2), (0x31, 3)] {
        for value in [0u16, 0x1234, 0xFFFF] {
            for pc in [0x8000u16, 0xFFFE, 0xFFFF] {
                for flags in 0..16 {
                    let mut cpu = seeded_cpu(pc, flags);
                    let mut bus = Bus::new();
                    bus.write(pc, opcode);
                    bus.write(pc.wrapping_add(1), value as u8);
                    bus.write(pc.wrapping_add(2), (value >> 8) as u8);
                    let mut expected = seeded_cpu(pc, flags);
                    match destination {
                        0 => [expected.b, expected.c] = value.to_be_bytes(),
                        1 => [expected.d, expected.e] = value.to_be_bytes(),
                        2 => [expected.h, expected.l] = value.to_be_bytes(),
                        3 => expected.sp = value,
                        _ => unreachable!(),
                    }
                    expected.pc = pc.wrapping_add(3);
                    cpu.step(&mut bus);
                    assert_eq!(cpu, expected, "PC={pc:04X}, initial flags={flags:X}");
                }
            }
        }
    }
}

#[test]
fn ld_r16mem_a() {
    for (opcode, pair) in [(0x02, 0), (0x12, 1), (0x22, 2), (0x32, 3)] {
        for address in [0u16, 0x00FF, 0x0100, 0xC030, 0xFFFF] {
            for pc in [0x8000u16, 0xFFFE, 0xFFFF] {
                for flags in 0..16 {
                    let mut cpu = seeded_cpu(pc, flags);
                    let mut bus = Bus::new();
                    match pair {
                        0 => [cpu.b, cpu.c] = address.to_be_bytes(),
                        1 => [cpu.d, cpu.e] = address.to_be_bytes(),
                        _ => [cpu.h, cpu.l] = address.to_be_bytes(),
                    }
                    // Avoid overlapping data with the instruction when PC is at the boundary.
                    if address == pc {
                        continue;
                    }
                    bus.write(pc, opcode);
                    bus.write(address, 0x5A);
                    let mut expected = seeded_cpu(pc, flags);
                    match pair {
                        0 => [expected.b, expected.c] = address.to_be_bytes(),
                        1 => [expected.d, expected.e] = address.to_be_bytes(),
                        2 => [expected.h, expected.l] = address.wrapping_add(1).to_be_bytes(),
                        3 => [expected.h, expected.l] = address.wrapping_sub(1).to_be_bytes(),
                        _ => unreachable!(),
                    }

                    expected.pc = pc.wrapping_add(1);
                    cpu.step(&mut bus);
                    assert_eq!(cpu, expected, "PC={pc:04X}, initial flags={flags:X}");
                    assert_eq!(bus.read(address), 0xA5);
                }
            }
        }
    }
}

#[test]
fn ld_a_r16mem() {
    for (opcode, pair) in [(0x0A, 0), (0x1A, 1), (0x2A, 2), (0x3A, 3)] {
        for address in [0u16, 0x00FF, 0x0100, 0xC030, 0xFFFF] {
            for pc in [0x8000u16, 0xFFFE, 0xFFFF] {
                for flags in 0..16 {
                    let mut cpu = seeded_cpu(pc, flags);
                    let mut bus = Bus::new();
                    match pair {
                        0 => [cpu.b, cpu.c] = address.to_be_bytes(),
                        1 => [cpu.d, cpu.e] = address.to_be_bytes(),
                        _ => [cpu.h, cpu.l] = address.to_be_bytes(),
                    }
                    // Avoid overlapping data with the instruction when PC is at the boundary.
                    if address == pc {
                        continue;
                    }
                    bus.write(pc, opcode);
                    bus.write(address, 0x5A);
                    let mut expected = seeded_cpu(pc, flags);
                    match pair {
                        0 => [expected.b, expected.c] = address.to_be_bytes(),
                        1 => [expected.d, expected.e] = address.to_be_bytes(),
                        2 => [expected.h, expected.l] = address.wrapping_add(1).to_be_bytes(),
                        3 => [expected.h, expected.l] = address.wrapping_sub(1).to_be_bytes(),
                        _ => unreachable!(),
                    }
                    expected.a = 0x5A;
                    expected.pc = pc.wrapping_add(1);
                    cpu.step(&mut bus);
                    assert_eq!(cpu, expected, "PC={pc:04X}, initial flags={flags:X}");
                    assert_eq!(bus.read(address), 0x5A);
                }
            }
        }
    }
}

#[test]
fn ld_r8_imm8() {
    for destination in 0..8 {
        for value in [0, 0xA5, 0xFF] {
            for pc in [0x8000u16, 0xFFFE, 0xFFFF] {
                for flags in 0..16 {
                    let mut cpu = seeded_cpu(pc, flags);
                    let mut bus = Bus::new();
                    bus.write(pc, 0x06 + destination * 8);
                    bus.write(pc.wrapping_add(1), value);
                    bus.write(0xC030, 0x66);
                    let mut expected = seeded_cpu(pc, flags);
                    match destination {
                        0 => expected.b = value,
                        1 => expected.c = value,
                        2 => expected.d = value,
                        3 => expected.e = value,
                        4 => expected.h = value,
                        5 => expected.l = value,
                        6 => {}
                        7 => expected.a = value,
                        _ => unreachable!(),
                    }
                    expected.pc = pc.wrapping_add(2);
                    cpu.step(&mut bus);
                    assert_eq!(cpu, expected, "PC={pc:04X}, initial flags={flags:X}");
                    assert_eq!(
                        bus.read(0xC030),
                        if destination == 6 { value } else { 0x66 }
                    );
                }
            }
        }
    }
}

#[test]
fn ld_absolute_from_sp_stores_low_byte_first() {
    for address in [0xC234u16, 0xFFFF] {
        for sp in [0u16, 0xD135, 0xFFFF] {
            for pc in [0x8000u16, 0xFFFE, 0xFFFF] {
                for flags in 0..16 {
                    let mut cpu = seeded_cpu(pc, flags);
                    let mut bus = Bus::new();
                    cpu.sp = sp;
                    bus.write(pc, 0x08);
                    bus.write(pc.wrapping_add(1), address as u8);
                    bus.write(pc.wrapping_add(2), (address >> 8) as u8);
                    let mut expected = seeded_cpu(pc, flags);
                    expected.sp = sp;
                    expected.pc = pc.wrapping_add(3);
                    cpu.step(&mut bus);
                    assert_eq!(cpu, expected, "PC={pc:04X}, initial flags={flags:X}");
                    assert_eq!(bus.read(address), sp as u8);
                    assert_eq!(bus.read(address.wrapping_add(1)), (sp >> 8) as u8);
                }
            }
        }
    }
}

#[test]
fn ld_r8_r8() {
    for destination in 0..8 {
        for source in 0..8 {
            for pc in [0x8000u16, 0xFFFE, 0xFFFF] {
                for flags in 0..16 {
                    let mut cpu = seeded_cpu(pc, flags);
                    let mut bus = Bus::new();
                    let opcode = 0x40 + destination * 8 + source;
                    if opcode == 0x76 {
                        continue;
                    } // HALT is unimplemented.
                    let initial = [cpu.b, cpu.c, cpu.d, cpu.e, cpu.h, cpu.l, 0x66, cpu.a];
                    bus.write(0xC030, initial[6]);
                    bus.write(pc, opcode);
                    let value = initial[source as usize];
                    let mut expected = seeded_cpu(pc, flags);
                    match destination {
                        0 => expected.b = value,
                        1 => expected.c = value,
                        2 => expected.d = value,
                        3 => expected.e = value,
                        4 => expected.h = value,
                        5 => expected.l = value,
                        6 => {}
                        7 => expected.a = value,
                        _ => unreachable!(),
                    }
                    expected.pc = pc.wrapping_add(1);
                    cpu.step(&mut bus);
                    assert_eq!(cpu, expected, "PC={pc:04X}, initial flags={flags:X}");
                    assert_eq!(
                        bus.read(0xC030),
                        if destination == 6 { value } else { 0x66 }
                    );
                }
            }
        }
    }
}

#[test]
fn ldh_imm8_from_a() {
    for offset in [0u8, 0x80, 0xFF] {
        for pc in [0x8000u16, 0xFFFE, 0xFFFF] {
            for flags in 0..16 {
                let mut cpu = seeded_cpu(pc, flags);
                let mut bus = Bus::new();
                let address = 0xFF00 + u16::from(offset);
                if address == pc || address == pc.wrapping_add(1) {
                    continue;
                }
                cpu.c = offset;
                bus.write(pc, 224);
                bus.write(pc.wrapping_add(1), offset);
                bus.write(address, 0x5A);
                let mut expected = seeded_cpu(pc, flags);
                expected.c = offset;

                expected.pc = pc.wrapping_add(2);
                cpu.step(&mut bus);
                assert_eq!(cpu, expected, "PC={pc:04X}, initial flags={flags:X}");
                assert_eq!(bus.read(address), 0xA5);
            }
        }
    }
}

#[test]
fn ldh_c_from_a() {
    for offset in [0u8, 0x80, 0xFF] {
        for pc in [0x8000u16, 0xFFFE, 0xFFFF] {
            for flags in 0..16 {
                let mut cpu = seeded_cpu(pc, flags);
                let mut bus = Bus::new();
                let address = 0xFF00 + u16::from(offset);
                if address == pc {
                    continue;
                }
                cpu.c = offset;
                bus.write(pc, 226);
                bus.write(address, 0x5A);
                let mut expected = seeded_cpu(pc, flags);
                expected.c = offset;

                expected.pc = pc.wrapping_add(1);
                cpu.step(&mut bus);
                assert_eq!(cpu, expected, "PC={pc:04X}, initial flags={flags:X}");
                assert_eq!(bus.read(address), 0xA5);
            }
        }
    }
}

#[test]
fn ldh_a_from_imm8() {
    for offset in [0u8, 0x80, 0xFF] {
        for pc in [0x8000u16, 0xFFFE, 0xFFFF] {
            for flags in 0..16 {
                let mut cpu = seeded_cpu(pc, flags);
                let mut bus = Bus::new();
                let address = 0xFF00 + u16::from(offset);
                if address == pc || address == pc.wrapping_add(1) {
                    continue;
                }
                cpu.c = offset;
                bus.write(pc, 240);
                bus.write(pc.wrapping_add(1), offset);
                bus.write(address, 0x5A);
                let mut expected = seeded_cpu(pc, flags);
                expected.c = offset;
                expected.a = 0x5A;
                expected.pc = pc.wrapping_add(2);
                cpu.step(&mut bus);
                assert_eq!(cpu, expected, "PC={pc:04X}, initial flags={flags:X}");
                assert_eq!(bus.read(address), 0x5A);
            }
        }
    }
}

#[test]
fn ldh_a_from_c() {
    for offset in [0u8, 0x80, 0xFF] {
        for pc in [0x8000u16, 0xFFFE, 0xFFFF] {
            for flags in 0..16 {
                let mut cpu = seeded_cpu(pc, flags);
                let mut bus = Bus::new();
                let address = 0xFF00 + u16::from(offset);
                if address == pc {
                    continue;
                }
                cpu.c = offset;
                bus.write(pc, 242);
                bus.write(address, 0x5A);
                let mut expected = seeded_cpu(pc, flags);
                expected.c = offset;
                expected.a = 0x5A;
                expected.pc = pc.wrapping_add(1);
                cpu.step(&mut bus);
                assert_eq!(cpu, expected, "PC={pc:04X}, initial flags={flags:X}");
                assert_eq!(bus.read(address), 0x5A);
            }
        }
    }
}

#[test]
fn ld_absolute_from_a() {
    for address in [0xC167u16, 0x0000, 0xFFFF] {
        for pc in [0x8000u16, 0xFFFE, 0xFFFF] {
            for flags in 0..16 {
                let mut cpu = seeded_cpu(pc, flags);
                let mut bus = Bus::new();
                if (0..3).any(|i| pc.wrapping_add(i) == address) {
                    continue;
                }
                bus.write(pc, 234);
                bus.write(pc.wrapping_add(1), address as u8);
                bus.write(pc.wrapping_add(2), (address >> 8) as u8);
                bus.write(address, 0x5A);
                let mut expected = seeded_cpu(pc, flags);

                expected.pc = pc.wrapping_add(3);
                cpu.step(&mut bus);
                assert_eq!(cpu, expected, "PC={pc:04X}, initial flags={flags:X}");
                assert_eq!(bus.read(address), 0xA5);
            }
        }
    }
}

#[test]
fn ld_a_from_absolute() {
    for address in [0xC167u16, 0x0000, 0xFFFF] {
        for pc in [0x8000u16, 0xFFFE, 0xFFFF] {
            for flags in 0..16 {
                let mut cpu = seeded_cpu(pc, flags);
                let mut bus = Bus::new();
                if (0..3).any(|i| pc.wrapping_add(i) == address) {
                    continue;
                }
                bus.write(pc, 250);
                bus.write(pc.wrapping_add(1), address as u8);
                bus.write(pc.wrapping_add(2), (address >> 8) as u8);
                bus.write(address, 0x5A);
                let mut expected = seeded_cpu(pc, flags);
                expected.a = 0x5A;
                expected.pc = pc.wrapping_add(3);
                cpu.step(&mut bus);
                assert_eq!(cpu, expected, "PC={pc:04X}, initial flags={flags:X}");
                assert_eq!(bus.read(address), 0x5A);
            }
        }
    }
}

#[test]
fn ld_hl_sp_imm8() {
    for (sp, offset, hl, result_flags) in [
        (0xD0FF, 0x01, 0xD100u16, Flags::HALF | Flags::CARRY),
        (0xD000, 0xFF, 0xCFFF, Flags::empty()),
        (0x0000, 0x80, 0xFF80, Flags::empty()),
        (0xFFFF, 0x01, 0x0000, Flags::HALF | Flags::CARRY),
        (0x0001, 0xFF, 0x0000, Flags::HALF | Flags::CARRY),
        (0x000F, 0x01, 0x0010, Flags::HALF),
        (0x0080, 0x80, 0x0000, Flags::CARRY),
        (0xD000, 0x7F, 0xD07F, Flags::empty()),
        (0xD000, 0x00, 0xD000, Flags::empty()),
    ] {
        for pc in [0x8000u16, 0xFFFE, 0xFFFF] {
            for flags in 0..16 {
                let mut cpu = seeded_cpu(pc, flags);
                let mut bus = Bus::new();
                cpu.sp = sp;
                bus.write(pc, 0xF8);
                bus.write(pc.wrapping_add(1), offset);
                let mut expected = seeded_cpu(pc, flags);
                expected.sp = sp;
                [expected.h, expected.l] = hl.to_be_bytes();
                expected.f = Flags::from_bits_retain(result_flags.bits());
                expected.pc = pc.wrapping_add(2);
                cpu.step(&mut bus);
                assert_eq!(cpu, expected, "PC={pc:04X}, initial flags={flags:X}");
            }
        }
    }
}

#[test]
fn ld_sp_hl() {
    for hl in [0u16, 0xC055, 0xFFFF] {
        for pc in [0x8000u16, 0xFFFE, 0xFFFF] {
            for flags in 0..16 {
                let mut cpu = seeded_cpu(pc, flags);
                let mut bus = Bus::new();
                [cpu.h, cpu.l] = hl.to_be_bytes();
                bus.write(pc, 0xF9);
                let mut expected = seeded_cpu(pc, flags);
                [expected.h, expected.l] = hl.to_be_bytes();
                expected.sp = hl;
                expected.pc = pc.wrapping_add(1);
                cpu.step(&mut bus);
                assert_eq!(cpu, expected, "PC={pc:04X}, initial flags={flags:X}");
            }
        }
    }
}

#[test]
fn add_a_r8() {
    for source in 0..8 {
        for (a, operand, result, result_flags) in [
            (0x10, 0xF0, 0x00, Flags::ZERO | Flags::CARRY),
            (0x80, 0x80, 0x00, Flags::ZERO | Flags::CARRY),
            (0x08, 0x08, 0x10, Flags::HALF),
            (0xFF, 0x01, 0x00, Flags::ZERO | Flags::HALF | Flags::CARRY),
            (0xFF, 0xFF, 0xFE, Flags::HALF | Flags::CARRY),
            (0x01, 0x02, 0x03, Flags::empty()),
            (0x00, 0x00, 0x00, Flags::ZERO),
        ] {
            for pc in [0x8000u16, 0xFFFE, 0xFFFF] {
                for flags in 0..16 {
                    let mut cpu = seeded_cpu(pc, flags);
                    let mut bus = Bus::new();
                    // ADD A,A can only use equal operands.
                    if source == 7 && a != operand {
                        continue;
                    }
                    cpu.a = a;
                    match source {
                        0 => cpu.b = operand,
                        1 => cpu.c = operand,
                        2 => cpu.d = operand,
                        3 => cpu.e = operand,
                        4 => cpu.h = operand,
                        5 => cpu.l = operand,
                        6 | 7 => {}
                        _ => unreachable!(),
                    }
                    let address = u16::from_be_bytes([cpu.h, cpu.l]);
                    bus.write(address, if source == 6 { operand } else { 0x66 });
                    bus.write(pc, 0x80 + source);
                    let mut expected = seeded_cpu(pc, flags);
                    expected.a = result;
                    match source {
                        0 => expected.b = operand,
                        1 => expected.c = operand,
                        2 => expected.d = operand,
                        3 => expected.e = operand,
                        4 => expected.h = operand,
                        5 => expected.l = operand,
                        6 | 7 => {}
                        _ => unreachable!(),
                    }
                    expected.f = Flags::from_bits_retain(result_flags.bits());
                    expected.pc = pc.wrapping_add(1);
                    cpu.step(&mut bus);
                    assert_eq!(cpu, expected, "PC={pc:04X}, initial flags={flags:X}");
                    assert_eq!(bus.read(address), if source == 6 { operand } else { 0x66 });
                }
            }
        }
    }
}

#[test]
fn adc_a_r8() {
    for source in 0..8 {
        // Expected outcomes with carry clear and set, respectively.
        for (a, operand, without_carry, with_carry) in [
            (
                0x10,
                0xF0,
                (0x00, Flags::ZERO | Flags::CARRY),
                (0x01, Flags::CARRY),
            ),
            (
                0x80,
                0x80,
                (0x00, Flags::ZERO | Flags::CARRY),
                (0x01, Flags::CARRY),
            ),
            (0x08, 0x08, (0x10, Flags::HALF), (0x11, Flags::HALF)),
            (0x0F, 0x00, (0x0F, Flags::empty()), (0x10, Flags::HALF)),
            (
                0xFF,
                0x00,
                (0xFF, Flags::empty()),
                (0x00, Flags::ZERO | Flags::HALF | Flags::CARRY),
            ),
            (
                0x00,
                0xFF,
                (0xFF, Flags::empty()),
                (0x00, Flags::ZERO | Flags::HALF | Flags::CARRY),
            ),
            (
                0xFF,
                0xFF,
                (0xFE, Flags::HALF | Flags::CARRY),
                (0xFF, Flags::HALF | Flags::CARRY),
            ),
            (0x7F, 0x7F, (0xFE, Flags::HALF), (0xFF, Flags::HALF)),
            (0x01, 0x02, (0x03, Flags::empty()), (0x04, Flags::empty())),
            (0x00, 0x00, (0x00, Flags::ZERO), (0x01, Flags::empty())),
        ] {
            for pc in [0x8000u16, 0xFFFE, 0xFFFF] {
                for flags in 0..16 {
                    let mut cpu = seeded_cpu(pc, flags);
                    let mut bus = Bus::new();
                    // ADC A,A can only use equal operands.
                    if source == 7 && a != operand {
                        continue;
                    }
                    cpu.a = a;
                    match source {
                        0 => cpu.b = operand,
                        1 => cpu.c = operand,
                        2 => cpu.d = operand,
                        3 => cpu.e = operand,
                        4 => cpu.h = operand,
                        5 => cpu.l = operand,
                        6 | 7 => {}
                        _ => unreachable!(),
                    }
                    let address = u16::from_be_bytes([cpu.h, cpu.l]);
                    bus.write(address, if source == 6 { operand } else { 0x66 });
                    bus.write(pc, 0x88 + source);
                    let mut expected = seeded_cpu(pc, flags);
                    let (result, result_flags) = if cpu.f.contains(Flags::CARRY) {
                        &with_carry
                    } else {
                        &without_carry
                    };
                    expected.a = *result;
                    match source {
                        0 => expected.b = operand,
                        1 => expected.c = operand,
                        2 => expected.d = operand,
                        3 => expected.e = operand,
                        4 => expected.h = operand,
                        5 => expected.l = operand,
                        6 | 7 => {}
                        _ => unreachable!(),
                    }
                    expected.f = Flags::from_bits_retain(result_flags.bits());
                    expected.pc = pc.wrapping_add(1);
                    cpu.step(&mut bus);
                    assert_eq!(
                        cpu, expected,
                        "source={source}, A={a:02X}, operand={operand:02X}, PC={pc:04X}, initial flags={flags:X}"
                    );
                    assert_eq!(bus.read(address), if source == 6 { operand } else { 0x66 });
                }
            }
        }
    }
}

#[test]
fn add_a_imm8() {
    for (a, operand, result, result_flags) in [
        (0x10, 0xF0, 0x00, Flags::ZERO | Flags::CARRY),
        (0x80, 0x80, 0x00, Flags::ZERO | Flags::CARRY),
        (0x08, 0x08, 0x10, Flags::HALF),
        (0xFF, 0x01, 0x00, Flags::ZERO | Flags::HALF | Flags::CARRY),
        (0xFF, 0xFF, 0xFE, Flags::HALF | Flags::CARRY),
        (0x01, 0x02, 0x03, Flags::empty()),
        (0x00, 0x00, 0x00, Flags::ZERO),
    ] {
        for pc in [0x8000u16, 0xFFFE, 0xFFFF] {
            for flags in 0..16 {
                let mut cpu = seeded_cpu(pc, flags);
                let mut bus = Bus::new();
                cpu.a = a;
                let operand_address = pc.wrapping_add(1);
                let following_address = pc.wrapping_add(2);
                bus.write(pc, 0xC6);
                bus.write(operand_address, operand);
                bus.write(following_address, 0xD3); // Must not fetch another opcode.
                bus.write(0xC030, 0x66); // [HL] must remain untouched.

                let mut expected = seeded_cpu(pc, flags);
                expected.a = result;
                expected.f = Flags::from_bits_retain(result_flags.bits());
                expected.pc = following_address;

                cpu.step(&mut bus);

                assert_eq!(
                    cpu, expected,
                    "A={a:02X}, operand={operand:02X}, PC={pc:04X}, initial flags={flags:X}"
                );
                assert_eq!(bus.read(pc), 0xC6);
                assert_eq!(bus.read(operand_address), operand);
                assert_eq!(bus.read(following_address), 0xD3);
                assert_eq!(bus.read(0xC030), 0x66);
            }
        }
    }
}

#[test]
fn adc_a_imm8() {
    for (a, operand, without_carry, with_carry) in [
        (
            0x10,
            0xF0,
            (0x00, Flags::ZERO | Flags::CARRY),
            (0x01, Flags::CARRY),
        ),
        (
            0x80,
            0x80,
            (0x00, Flags::ZERO | Flags::CARRY),
            (0x01, Flags::CARRY),
        ),
        (0x08, 0x08, (0x10, Flags::HALF), (0x11, Flags::HALF)),
        (0x0F, 0x00, (0x0F, Flags::empty()), (0x10, Flags::HALF)),
        (
            0xFF,
            0x00,
            (0xFF, Flags::empty()),
            (0x00, Flags::ZERO | Flags::HALF | Flags::CARRY),
        ),
        (
            0x00,
            0xFF,
            (0xFF, Flags::empty()),
            (0x00, Flags::ZERO | Flags::HALF | Flags::CARRY),
        ),
        (
            0xFF,
            0xFF,
            (0xFE, Flags::HALF | Flags::CARRY),
            (0xFF, Flags::HALF | Flags::CARRY),
        ),
        (0x7F, 0x7F, (0xFE, Flags::HALF), (0xFF, Flags::HALF)),
        (0x01, 0x02, (0x03, Flags::empty()), (0x04, Flags::empty())),
        (0x00, 0x00, (0x00, Flags::ZERO), (0x01, Flags::empty())),
    ] {
        for pc in [0x8000u16, 0xFFFE, 0xFFFF] {
            for flags in 0..16 {
                let mut cpu = seeded_cpu(pc, flags);
                let mut bus = Bus::new();
                cpu.a = a;
                let operand_address = pc.wrapping_add(1);
                let following_address = pc.wrapping_add(2);
                bus.write(pc, 0xCE);
                bus.write(operand_address, operand);
                bus.write(following_address, 0xD3); // Must not fetch another opcode.
                bus.write(0xC030, 0x66); // [HL] must remain untouched.

                let mut expected = seeded_cpu(pc, flags);
                let (result, result_flags) = if cpu.f.contains(Flags::CARRY) {
                    &with_carry
                } else {
                    &without_carry
                };
                expected.a = *result;
                expected.f = Flags::from_bits_retain(result_flags.bits());
                expected.pc = following_address;

                cpu.step(&mut bus);

                assert_eq!(
                    cpu, expected,
                    "A={a:02X}, operand={operand:02X}, PC={pc:04X}, initial flags={flags:X}"
                );
                assert_eq!(bus.read(pc), 0xCE);
                assert_eq!(bus.read(operand_address), operand);
                assert_eq!(bus.read(following_address), 0xD3);
                assert_eq!(bus.read(0xC030), 0x66);
            }
        }
    }
}

#[test]
fn sub_a_r8() {
    for source in 0..8 {
        for (a, operand, result, result_flags) in [
            (0x00, 0x00, 0x00, Flags::ZERO | Flags::SUB),
            (0x80, 0x80, 0x00, Flags::ZERO | Flags::SUB),
            (0xFF, 0xFF, 0x00, Flags::ZERO | Flags::SUB),
            (0x10, 0x01, 0x0F, Flags::SUB | Flags::HALF),
            (0x1F, 0x01, 0x1E, Flags::SUB),
            (0x10, 0x20, 0xF0, Flags::SUB | Flags::CARRY),
            (0x00, 0x01, 0xFF, Flags::SUB | Flags::HALF | Flags::CARRY),
            (0x00, 0xFF, 0x01, Flags::SUB | Flags::HALF | Flags::CARRY),
            (0xFF, 0x00, 0xFF, Flags::SUB),
            (0x03, 0x02, 0x01, Flags::SUB),
        ] {
            for pc in [0x8000u16, 0xFFFE, 0xFFFF] {
                for flags in 0..16 {
                    let mut cpu = seeded_cpu(pc, flags);
                    let mut bus = Bus::new();
                    // SUB A,A can only use equal operands.
                    if source == 7 && a != operand {
                        continue;
                    }
                    cpu.a = a;
                    match source {
                        0 => cpu.b = operand,
                        1 => cpu.c = operand,
                        2 => cpu.d = operand,
                        3 => cpu.e = operand,
                        4 => cpu.h = operand,
                        5 => cpu.l = operand,
                        6 | 7 => {}
                        _ => unreachable!(),
                    }
                    let address = u16::from_be_bytes([cpu.h, cpu.l]);
                    bus.write(address, if source == 6 { operand } else { 0x66 });
                    bus.write(pc, 0x90 + source);
                    let mut expected = seeded_cpu(pc, flags);
                    expected.a = result;
                    match source {
                        0 => expected.b = operand,
                        1 => expected.c = operand,
                        2 => expected.d = operand,
                        3 => expected.e = operand,
                        4 => expected.h = operand,
                        5 => expected.l = operand,
                        6 | 7 => {}
                        _ => unreachable!(),
                    }
                    expected.f = Flags::from_bits_retain(result_flags.bits());
                    expected.pc = pc.wrapping_add(1);
                    cpu.step(&mut bus);
                    assert_eq!(
                        cpu, expected,
                        "source={source}, A={a:02X}, operand={operand:02X}, PC={pc:04X}, initial flags={flags:X}"
                    );
                    assert_eq!(bus.read(address), if source == 6 { operand } else { 0x66 });
                }
            }
        }
    }
}

#[test]
fn sub_a_imm8() {
    for (a, operand, result, result_flags) in [
        (0x00, 0x00, 0x00, Flags::ZERO | Flags::SUB),
        (0x80, 0x80, 0x00, Flags::ZERO | Flags::SUB),
        (0xFF, 0xFF, 0x00, Flags::ZERO | Flags::SUB),
        (0x10, 0x01, 0x0F, Flags::SUB | Flags::HALF),
        (0x1F, 0x01, 0x1E, Flags::SUB),
        (0x10, 0x20, 0xF0, Flags::SUB | Flags::CARRY),
        (0x00, 0x01, 0xFF, Flags::SUB | Flags::HALF | Flags::CARRY),
        (0x00, 0xFF, 0x01, Flags::SUB | Flags::HALF | Flags::CARRY),
        (0xFF, 0x00, 0xFF, Flags::SUB),
        (0x03, 0x02, 0x01, Flags::SUB),
    ] {
        for pc in [0x8000u16, 0xFFFE, 0xFFFF] {
            for flags in 0..16 {
                let mut cpu = seeded_cpu(pc, flags);
                let mut bus = Bus::new();
                cpu.a = a;
                let operand_address = pc.wrapping_add(1);
                let following_address = pc.wrapping_add(2);
                bus.write(pc, 0xD6);
                bus.write(operand_address, operand);
                bus.write(following_address, 0xD3); // Must not fetch another opcode.
                bus.write(0xC030, 0x66); // [HL] must remain untouched.

                let mut expected = seeded_cpu(pc, flags);
                expected.a = result;
                expected.f = Flags::from_bits_retain(result_flags.bits());
                expected.pc = following_address;

                cpu.step(&mut bus);

                assert_eq!(
                    cpu, expected,
                    "A={a:02X}, operand={operand:02X}, PC={pc:04X}, initial flags={flags:X}"
                );
                assert_eq!(bus.read(pc), 0xD6);
                assert_eq!(bus.read(operand_address), operand);
                assert_eq!(bus.read(following_address), 0xD3);
                assert_eq!(bus.read(0xC030), 0x66);
            }
        }
    }
}

#[test]
fn sbc_a_r8() {
    for source in 0..8 {
        // Expected outcomes with carry clear and set, respectively.
        for (a, operand, without_carry, with_carry) in [
            (
                0x00,
                0x00,
                (0x00, Flags::ZERO | Flags::SUB),
                (0xFF, Flags::SUB | Flags::HALF | Flags::CARRY),
            ),
            (
                0x80,
                0x80,
                (0x00, Flags::ZERO | Flags::SUB),
                (0xFF, Flags::SUB | Flags::HALF | Flags::CARRY),
            ),
            (
                0xFF,
                0xFF,
                (0x00, Flags::ZERO | Flags::SUB),
                (0xFF, Flags::SUB | Flags::HALF | Flags::CARRY),
            ),
            (
                0x10,
                0x01,
                (0x0F, Flags::SUB | Flags::HALF),
                (0x0E, Flags::SUB | Flags::HALF),
            ),
            (
                0x10,
                0x0F,
                (0x01, Flags::SUB | Flags::HALF),
                (0x00, Flags::ZERO | Flags::SUB | Flags::HALF),
            ),
            (
                0x10,
                0x00,
                (0x10, Flags::SUB),
                (0x0F, Flags::SUB | Flags::HALF),
            ),
            (
                0x10,
                0x20,
                (0xF0, Flags::SUB | Flags::CARRY),
                (0xEF, Flags::SUB | Flags::HALF | Flags::CARRY),
            ),
            (
                0x00,
                0xFF,
                (0x01, Flags::SUB | Flags::HALF | Flags::CARRY),
                (0x00, Flags::ZERO | Flags::SUB | Flags::HALF | Flags::CARRY),
            ),
            (0xFF, 0x00, (0xFF, Flags::SUB), (0xFE, Flags::SUB)),
            (
                0x03,
                0x02,
                (0x01, Flags::SUB),
                (0x00, Flags::ZERO | Flags::SUB),
            ),
        ] {
            for pc in [0x8000u16, 0xFFFE, 0xFFFF] {
                for flags in 0..16 {
                    let mut cpu = seeded_cpu(pc, flags);
                    let mut bus = Bus::new();
                    // SBC A,A can only use equal operands.
                    if source == 7 && a != operand {
                        continue;
                    }
                    cpu.a = a;
                    match source {
                        0 => cpu.b = operand,
                        1 => cpu.c = operand,
                        2 => cpu.d = operand,
                        3 => cpu.e = operand,
                        4 => cpu.h = operand,
                        5 => cpu.l = operand,
                        6 | 7 => {}
                        _ => unreachable!(),
                    }
                    let address = u16::from_be_bytes([cpu.h, cpu.l]);
                    bus.write(address, if source == 6 { operand } else { 0x66 });
                    bus.write(pc, 0x98 + source);
                    let mut expected = seeded_cpu(pc, flags);
                    let (result, result_flags) = if cpu.f.contains(Flags::CARRY) {
                        &with_carry
                    } else {
                        &without_carry
                    };
                    expected.a = *result;
                    match source {
                        0 => expected.b = operand,
                        1 => expected.c = operand,
                        2 => expected.d = operand,
                        3 => expected.e = operand,
                        4 => expected.h = operand,
                        5 => expected.l = operand,
                        6 | 7 => {}
                        _ => unreachable!(),
                    }
                    expected.f = Flags::from_bits_retain(result_flags.bits());
                    expected.pc = pc.wrapping_add(1);
                    cpu.step(&mut bus);
                    assert_eq!(
                        cpu, expected,
                        "source={source}, A={a:02X}, operand={operand:02X}, PC={pc:04X}, initial flags={flags:X}"
                    );
                    assert_eq!(bus.read(address), if source == 6 { operand } else { 0x66 });
                }
            }
        }
    }
}

#[test]
fn sbc_a_imm8() {
    for (a, operand, without_carry, with_carry) in [
        (
            0x00,
            0x00,
            (0x00, Flags::ZERO | Flags::SUB),
            (0xFF, Flags::SUB | Flags::HALF | Flags::CARRY),
        ),
        (
            0x80,
            0x80,
            (0x00, Flags::ZERO | Flags::SUB),
            (0xFF, Flags::SUB | Flags::HALF | Flags::CARRY),
        ),
        (
            0xFF,
            0xFF,
            (0x00, Flags::ZERO | Flags::SUB),
            (0xFF, Flags::SUB | Flags::HALF | Flags::CARRY),
        ),
        (
            0x10,
            0x01,
            (0x0F, Flags::SUB | Flags::HALF),
            (0x0E, Flags::SUB | Flags::HALF),
        ),
        (
            0x10,
            0x0F,
            (0x01, Flags::SUB | Flags::HALF),
            (0x00, Flags::ZERO | Flags::SUB | Flags::HALF),
        ),
        (
            0x10,
            0x00,
            (0x10, Flags::SUB),
            (0x0F, Flags::SUB | Flags::HALF),
        ),
        (
            0x10,
            0x20,
            (0xF0, Flags::SUB | Flags::CARRY),
            (0xEF, Flags::SUB | Flags::HALF | Flags::CARRY),
        ),
        (
            0x00,
            0xFF,
            (0x01, Flags::SUB | Flags::HALF | Flags::CARRY),
            (0x00, Flags::ZERO | Flags::SUB | Flags::HALF | Flags::CARRY),
        ),
        (0xFF, 0x00, (0xFF, Flags::SUB), (0xFE, Flags::SUB)),
        (
            0x03,
            0x02,
            (0x01, Flags::SUB),
            (0x00, Flags::ZERO | Flags::SUB),
        ),
    ] {
        for pc in [0x8000u16, 0xFFFE, 0xFFFF] {
            for flags in 0..16 {
                let mut cpu = seeded_cpu(pc, flags);
                let mut bus = Bus::new();
                cpu.a = a;
                let operand_address = pc.wrapping_add(1);
                let following_address = pc.wrapping_add(2);
                bus.write(pc, 0xDE);
                bus.write(operand_address, operand);
                bus.write(following_address, 0xD3); // Must not fetch another opcode.
                bus.write(0xC030, 0x66); // [HL] must remain untouched.

                let mut expected = seeded_cpu(pc, flags);
                let (result, result_flags) = if cpu.f.contains(Flags::CARRY) {
                    &with_carry
                } else {
                    &without_carry
                };
                expected.a = *result;
                expected.f = Flags::from_bits_retain(result_flags.bits());
                expected.pc = following_address;

                cpu.step(&mut bus);

                assert_eq!(
                    cpu, expected,
                    "A={a:02X}, operand={operand:02X}, PC={pc:04X}, initial flags={flags:X}"
                );
                assert_eq!(bus.read(pc), 0xDE);
                assert_eq!(bus.read(operand_address), operand);
                assert_eq!(bus.read(following_address), 0xD3);
                assert_eq!(bus.read(0xC030), 0x66);
            }
        }
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
