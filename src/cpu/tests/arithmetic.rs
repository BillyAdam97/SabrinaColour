use super::seeded_cpu;
use crate::bus::Bus;
use crate::cpu::registers::Flags;

#[test]
fn inc_r16() {
    for (opcode, destination) in [(0x03, 0), (0x13, 1), (0x23, 2), (0x33, 3)] {
        for (value, result) in [
            (0x0000u16, 0x0001u16),
            (0x000F, 0x0010),
            (0x00FF, 0x0100),
            (0x0FFF, 0x1000),
            (0x7FFF, 0x8000),
            (0xFFFE, 0xFFFF),
            (0xFFFF, 0x0000),
        ] {
            for pc in [0x8000u16, 0xFFFE, 0xFFFF] {
                for flags in 0..16 {
                    let mut cpu = seeded_cpu(pc, flags);
                    let mut bus = Bus::new();
                    match destination {
                        0 => [cpu.b, cpu.c] = value.to_be_bytes(),
                        1 => [cpu.d, cpu.e] = value.to_be_bytes(),
                        2 => [cpu.h, cpu.l] = value.to_be_bytes(),
                        3 => cpu.sp = value,
                        _ => unreachable!(),
                    }
                    bus.write(pc, opcode);
                    bus.write(0xC030, 0x66);

                    let mut expected = seeded_cpu(pc, flags);
                    match destination {
                        0 => [expected.b, expected.c] = result.to_be_bytes(),
                        1 => [expected.d, expected.e] = result.to_be_bytes(),
                        2 => [expected.h, expected.l] = result.to_be_bytes(),
                        3 => expected.sp = result,
                        _ => unreachable!(),
                    }
                    expected.pc = pc.wrapping_add(1);

                    cpu.step(&mut bus);

                    assert_eq!(
                        cpu, expected,
                        "destination={destination}, value={value:04X}, PC={pc:04X}, initial flags={flags:X}"
                    );
                    assert_eq!(bus.read(pc), opcode);
                    assert_eq!(bus.read(0xC030), 0x66);
                }
            }
        }
    }
}

#[test]
fn dec_r16() {
    for (opcode, destination) in [(0x0B, 0), (0x1B, 1), (0x2B, 2), (0x3B, 3)] {
        for (value, result) in [
            (0x0000u16, 0xFFFFu16),
            (0x0001, 0x0000),
            (0x0010, 0x000F),
            (0x0100, 0x00FF),
            (0x1000, 0x0FFF),
            (0x8000, 0x7FFF),
            (0xFFFF, 0xFFFE),
        ] {
            for pc in [0x8000u16, 0xFFFE, 0xFFFF] {
                for flags in 0..16 {
                    let mut cpu = seeded_cpu(pc, flags);
                    let mut bus = Bus::new();
                    match destination {
                        0 => [cpu.b, cpu.c] = value.to_be_bytes(),
                        1 => [cpu.d, cpu.e] = value.to_be_bytes(),
                        2 => [cpu.h, cpu.l] = value.to_be_bytes(),
                        3 => cpu.sp = value,
                        _ => unreachable!(),
                    }
                    bus.write(pc, opcode);
                    bus.write(0xC030, 0x66);

                    let mut expected = seeded_cpu(pc, flags);
                    match destination {
                        0 => [expected.b, expected.c] = result.to_be_bytes(),
                        1 => [expected.d, expected.e] = result.to_be_bytes(),
                        2 => [expected.h, expected.l] = result.to_be_bytes(),
                        3 => expected.sp = result,
                        _ => unreachable!(),
                    }
                    expected.pc = pc.wrapping_add(1);

                    cpu.step(&mut bus);

                    assert_eq!(
                        cpu, expected,
                        "destination={destination}, value={value:04X}, PC={pc:04X}, initial flags={flags:X}"
                    );
                    assert_eq!(bus.read(pc), opcode);
                    assert_eq!(bus.read(0xC030), 0x66);
                }
            }
        }
    }
}

#[test]
fn add_hl_r16() {
    for (opcode, source) in [(0x09, 0), (0x19, 1), (0x29, 2), (0x39, 3)] {
        for (hl, operand, result, result_flags) in [
            (0x0000u16, 0x0000u16, 0x0000u16, Flags::empty()),
            (0x1234, 0x0102, 0x1336, Flags::empty()),
            (0x00FF, 0x0001, 0x0100, Flags::empty()),
            (0x0FFF, 0x0001, 0x1000, Flags::HALF),
            (0x0800, 0x0800, 0x1000, Flags::HALF),
            (0x8000, 0x8000, 0x0000, Flags::CARRY),
            (0xFFFF, 0x0001, 0x0000, Flags::HALF | Flags::CARRY),
            (0xFFFF, 0xFFFF, 0xFFFE, Flags::HALF | Flags::CARRY),
        ] {
            for pc in [0x8000u16, 0xFFFE, 0xFFFF] {
                for flags in 0..16 {
                    let mut cpu = seeded_cpu(pc, flags);
                    let mut bus = Bus::new();
                    // ADD HL,HL can only use equal operands.
                    if source == 2 && hl != operand {
                        continue;
                    }
                    [cpu.h, cpu.l] = hl.to_be_bytes();
                    match source {
                        0 => [cpu.b, cpu.c] = operand.to_be_bytes(),
                        1 => [cpu.d, cpu.e] = operand.to_be_bytes(),
                        2 => {}
                        3 => cpu.sp = operand,
                        _ => unreachable!(),
                    }
                    bus.write(pc, opcode);
                    bus.write(0xC030, 0x66);

                    let mut expected = seeded_cpu(pc, flags);
                    match source {
                        0 => [expected.b, expected.c] = operand.to_be_bytes(),
                        1 => [expected.d, expected.e] = operand.to_be_bytes(),
                        2 => {}
                        3 => expected.sp = operand,
                        _ => unreachable!(),
                    }
                    [expected.h, expected.l] = result.to_be_bytes();
                    expected.f = Flags::from_bits_retain(result_flags.bits());
                    expected.f.set(Flags::ZERO, cpu.f.contains(Flags::ZERO));
                    expected.pc = pc.wrapping_add(1);

                    cpu.step(&mut bus);

                    assert_eq!(
                        cpu, expected,
                        "source={source}, HL={hl:04X}, operand={operand:04X}, PC={pc:04X}, initial flags={flags:X}"
                    );
                    assert_eq!(bus.read(pc), opcode);
                    assert_eq!(bus.read(0xC030), 0x66);
                }
            }
        }
    }
}

#[test]
fn add_sp_imm8() {
    for (sp, offset, result, result_flags) in [
        (0xD0FF, 0x01, 0xD100, Flags::HALF | Flags::CARRY),
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
                let operand_address = pc.wrapping_add(1);
                let following_address = pc.wrapping_add(2);
                bus.write(pc, 0xE8);
                bus.write(operand_address, offset);
                bus.write(following_address, 0xD3); // Must not fetch another opcode.
                bus.write(0xC030, 0x66); // [HL] must remain untouched.

                let mut expected = seeded_cpu(pc, flags);
                expected.sp = result;
                expected.f = Flags::from_bits_retain(result_flags.bits());
                expected.pc = following_address;

                cpu.step(&mut bus);

                assert_eq!(
                    cpu, expected,
                    "SP={sp:04X}, offset={offset:02X}, PC={pc:04X}, initial flags={flags:X}"
                );
                assert_eq!(bus.read(pc), 0xE8);
                assert_eq!(bus.read(operand_address), offset);
                assert_eq!(bus.read(following_address), 0xD3);
                assert_eq!(bus.read(0xC030), 0x66);
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
fn and_a_r8() {
    for source in 0..8 {
        for (a, operand, result, result_flags) in [
            (0x00, 0x00, 0x00, Flags::ZERO | Flags::HALF),
            (0x00, 0xFF, 0x00, Flags::ZERO | Flags::HALF),
            (0xFF, 0x00, 0x00, Flags::ZERO | Flags::HALF),
            (0xF0, 0x0F, 0x00, Flags::ZERO | Flags::HALF),
            (0xA5, 0x3C, 0x24, Flags::HALF),
            (0x80, 0x80, 0x80, Flags::HALF),
            (0xFF, 0xFF, 0xFF, Flags::HALF),
        ] {
            for pc in [0x8000u16, 0xFFFE, 0xFFFF] {
                for flags in 0..16 {
                    let mut cpu = seeded_cpu(pc, flags);
                    let mut bus = Bus::new();
                    // AND A,A can only use equal operands.
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
                    bus.write(pc, 0xA0 + source);
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
fn and_a_imm8() {
    for (a, operand, result, result_flags) in [
        (0x00, 0x00, 0x00, Flags::ZERO | Flags::HALF),
        (0x00, 0xFF, 0x00, Flags::ZERO | Flags::HALF),
        (0xFF, 0x00, 0x00, Flags::ZERO | Flags::HALF),
        (0xF0, 0x0F, 0x00, Flags::ZERO | Flags::HALF),
        (0xA5, 0x3C, 0x24, Flags::HALF),
        (0x80, 0x80, 0x80, Flags::HALF),
        (0xFF, 0xFF, 0xFF, Flags::HALF),
    ] {
        for pc in [0x8000u16, 0xFFFE, 0xFFFF] {
            for flags in 0..16 {
                let mut cpu = seeded_cpu(pc, flags);
                let mut bus = Bus::new();
                cpu.a = a;
                let operand_address = pc.wrapping_add(1);
                let following_address = pc.wrapping_add(2);
                bus.write(pc, 0xE6);
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
                assert_eq!(bus.read(pc), 0xE6);
                assert_eq!(bus.read(operand_address), operand);
                assert_eq!(bus.read(following_address), 0xD3);
                assert_eq!(bus.read(0xC030), 0x66);
            }
        }
    }
}

#[test]
fn xor_a_r8() {
    for source in 0..8 {
        for (a, operand, result, result_flags) in [
            (0x00, 0x00, 0x00, Flags::ZERO),
            (0x80, 0x80, 0x00, Flags::ZERO),
            (0xFF, 0xFF, 0x00, Flags::ZERO),
            (0x00, 0xFF, 0xFF, Flags::empty()),
            (0xFF, 0x00, 0xFF, Flags::empty()),
            (0xF0, 0x0F, 0xFF, Flags::empty()),
            (0xA5, 0x3C, 0x99, Flags::empty()),
        ] {
            for pc in [0x8000u16, 0xFFFE, 0xFFFF] {
                for flags in 0..16 {
                    let mut cpu = seeded_cpu(pc, flags);
                    let mut bus = Bus::new();
                    // XOR A,A can only use equal operands.
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
                    bus.write(pc, 0xA8 + source);
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
fn xor_a_imm8() {
    for (a, operand, result, result_flags) in [
        (0x00, 0x00, 0x00, Flags::ZERO),
        (0x80, 0x80, 0x00, Flags::ZERO),
        (0xFF, 0xFF, 0x00, Flags::ZERO),
        (0x00, 0xFF, 0xFF, Flags::empty()),
        (0xFF, 0x00, 0xFF, Flags::empty()),
        (0xF0, 0x0F, 0xFF, Flags::empty()),
        (0xA5, 0x3C, 0x99, Flags::empty()),
    ] {
        for pc in [0x8000u16, 0xFFFE, 0xFFFF] {
            for flags in 0..16 {
                let mut cpu = seeded_cpu(pc, flags);
                let mut bus = Bus::new();
                cpu.a = a;
                let operand_address = pc.wrapping_add(1);
                let following_address = pc.wrapping_add(2);
                bus.write(pc, 0xEE);
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
                assert_eq!(bus.read(pc), 0xEE);
                assert_eq!(bus.read(operand_address), operand);
                assert_eq!(bus.read(following_address), 0xD3);
                assert_eq!(bus.read(0xC030), 0x66);
            }
        }
    }
}

#[test]
fn or_a_r8() {
    for source in 0..8 {
        for (a, operand, result, result_flags) in [
            (0x00, 0x00, 0x00, Flags::ZERO),
            (0x00, 0xFF, 0xFF, Flags::empty()),
            (0xFF, 0x00, 0xFF, Flags::empty()),
            (0xF0, 0x0F, 0xFF, Flags::empty()),
            (0xA5, 0x3C, 0xBD, Flags::empty()),
            (0x80, 0x80, 0x80, Flags::empty()),
            (0xFF, 0xFF, 0xFF, Flags::empty()),
        ] {
            for pc in [0x8000u16, 0xFFFE, 0xFFFF] {
                for flags in 0..16 {
                    let mut cpu = seeded_cpu(pc, flags);
                    let mut bus = Bus::new();
                    // OR A,A can only use equal operands.
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
                    bus.write(pc, 0xB0 + source);
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
fn or_a_imm8() {
    for (a, operand, result, result_flags) in [
        (0x00, 0x00, 0x00, Flags::ZERO),
        (0x00, 0xFF, 0xFF, Flags::empty()),
        (0xFF, 0x00, 0xFF, Flags::empty()),
        (0xF0, 0x0F, 0xFF, Flags::empty()),
        (0xA5, 0x3C, 0xBD, Flags::empty()),
        (0x80, 0x80, 0x80, Flags::empty()),
        (0xFF, 0xFF, 0xFF, Flags::empty()),
    ] {
        for pc in [0x8000u16, 0xFFFE, 0xFFFF] {
            for flags in 0..16 {
                let mut cpu = seeded_cpu(pc, flags);
                let mut bus = Bus::new();
                cpu.a = a;
                let operand_address = pc.wrapping_add(1);
                let following_address = pc.wrapping_add(2);
                bus.write(pc, 0xF6);
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
                assert_eq!(bus.read(pc), 0xF6);
                assert_eq!(bus.read(operand_address), operand);
                assert_eq!(bus.read(following_address), 0xD3);
                assert_eq!(bus.read(0xC030), 0x66);
            }
        }
    }
}

#[test]
fn cp_a_r8() {
    for source in 0..8 {
        for (a, operand, result_flags) in [
            (0x00, 0x00, Flags::ZERO | Flags::SUB),
            (0x80, 0x80, Flags::ZERO | Flags::SUB),
            (0xFF, 0xFF, Flags::ZERO | Flags::SUB),
            (0x10, 0x01, Flags::SUB | Flags::HALF),
            (0x10, 0x20, Flags::SUB | Flags::CARRY),
            (0x00, 0xFF, Flags::SUB | Flags::HALF | Flags::CARRY),
            (0xFF, 0x00, Flags::SUB),
            (0x7F, 0x80, Flags::SUB | Flags::CARRY),
        ] {
            for pc in [0x8000u16, 0xFFFE, 0xFFFF] {
                for flags in 0..16 {
                    let mut cpu = seeded_cpu(pc, flags);
                    let mut bus = Bus::new();
                    // CP A,A can only use equal operands.
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
                    bus.write(pc, 0xB8 + source);
                    let mut expected = seeded_cpu(pc, flags);
                    expected.a = a;
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
fn cp_a_imm8() {
    for (a, operand, result_flags) in [
        (0x00, 0x00, Flags::ZERO | Flags::SUB),
        (0x80, 0x80, Flags::ZERO | Flags::SUB),
        (0xFF, 0xFF, Flags::ZERO | Flags::SUB),
        (0x10, 0x01, Flags::SUB | Flags::HALF),
        (0x10, 0x20, Flags::SUB | Flags::CARRY),
        (0x00, 0xFF, Flags::SUB | Flags::HALF | Flags::CARRY),
        (0xFF, 0x00, Flags::SUB),
        (0x7F, 0x80, Flags::SUB | Flags::CARRY),
    ] {
        for pc in [0x8000u16, 0xFFFE, 0xFFFF] {
            for flags in 0..16 {
                let mut cpu = seeded_cpu(pc, flags);
                let mut bus = Bus::new();
                cpu.a = a;
                let operand_address = pc.wrapping_add(1);
                let following_address = pc.wrapping_add(2);
                bus.write(pc, 0xFE);
                bus.write(operand_address, operand);
                bus.write(following_address, 0xD3); // Must not fetch another opcode.
                bus.write(0xC030, 0x66); // [HL] must remain untouched.

                let mut expected = seeded_cpu(pc, flags);
                expected.a = a;
                expected.f = Flags::from_bits_retain(result_flags.bits());
                expected.pc = following_address;

                cpu.step(&mut bus);

                assert_eq!(
                    cpu, expected,
                    "A={a:02X}, operand={operand:02X}, PC={pc:04X}, initial flags={flags:X}"
                );
                assert_eq!(bus.read(pc), 0xFE);
                assert_eq!(bus.read(operand_address), operand);
                assert_eq!(bus.read(following_address), 0xD3);
                assert_eq!(bus.read(0xC030), 0x66);
            }
        }
    }
}

#[test]
fn inc_r8() {
    for destination in 0..8 {
        for (value, result, result_flags) in [
            (0x00, 0x01, Flags::empty()),
            (0x0E, 0x0F, Flags::empty()),
            (0x0F, 0x10, Flags::HALF),
            (0x7F, 0x80, Flags::HALF),
            (0xFE, 0xFF, Flags::empty()),
            (0xFF, 0x00, Flags::ZERO | Flags::HALF),
        ] {
            for pc in [0x8000u16, 0xFFFE, 0xFFFF] {
                for flags in 0..16 {
                    let mut cpu = seeded_cpu(pc, flags);
                    let mut bus = Bus::new();
                    match destination {
                        0 => cpu.b = value,
                        1 => cpu.c = value,
                        2 => cpu.d = value,
                        3 => cpu.e = value,
                        4 => cpu.h = value,
                        5 => cpu.l = value,
                        6 => {}
                        7 => cpu.a = value,
                        _ => unreachable!(),
                    }
                    let address = u16::from_be_bytes([cpu.h, cpu.l]);
                    let opcode = 0x04 + destination * 8;
                    bus.write(pc, opcode);
                    bus.write(address, if destination == 6 { value } else { 0x66 });

                    let mut expected = seeded_cpu(pc, flags);
                    match destination {
                        0 => expected.b = result,
                        1 => expected.c = result,
                        2 => expected.d = result,
                        3 => expected.e = result,
                        4 => expected.h = result,
                        5 => expected.l = result,
                        6 => {}
                        7 => expected.a = result,
                        _ => unreachable!(),
                    }
                    expected.f = Flags::from_bits_retain(result_flags.bits());
                    expected.f.set(Flags::CARRY, cpu.f.contains(Flags::CARRY));
                    expected.pc = pc.wrapping_add(1);

                    cpu.step(&mut bus);

                    assert_eq!(
                        cpu, expected,
                        "destination={destination}, value={value:02X}, PC={pc:04X}, initial flags={flags:X}"
                    );
                    assert_eq!(bus.read(pc), opcode);
                    assert_eq!(
                        bus.read(address),
                        if destination == 6 { result } else { 0x66 }
                    );
                }
            }
        }
    }
}

#[test]
fn dec_r8() {
    for destination in 0..8 {
        for (value, result, result_flags) in [
            (0x00, 0xFF, Flags::SUB | Flags::HALF),
            (0x01, 0x00, Flags::ZERO | Flags::SUB),
            (0x10, 0x0F, Flags::SUB | Flags::HALF),
            (0x11, 0x10, Flags::SUB),
            (0x80, 0x7F, Flags::SUB | Flags::HALF),
            (0xFF, 0xFE, Flags::SUB),
        ] {
            for pc in [0x8000u16, 0xFFFE, 0xFFFF] {
                for flags in 0..16 {
                    let mut cpu = seeded_cpu(pc, flags);
                    let mut bus = Bus::new();
                    match destination {
                        0 => cpu.b = value,
                        1 => cpu.c = value,
                        2 => cpu.d = value,
                        3 => cpu.e = value,
                        4 => cpu.h = value,
                        5 => cpu.l = value,
                        6 => {}
                        7 => cpu.a = value,
                        _ => unreachable!(),
                    }
                    let address = u16::from_be_bytes([cpu.h, cpu.l]);
                    let opcode = 0x05 + destination * 8;
                    bus.write(pc, opcode);
                    bus.write(address, if destination == 6 { value } else { 0x66 });

                    let mut expected = seeded_cpu(pc, flags);
                    match destination {
                        0 => expected.b = result,
                        1 => expected.c = result,
                        2 => expected.d = result,
                        3 => expected.e = result,
                        4 => expected.h = result,
                        5 => expected.l = result,
                        6 => {}
                        7 => expected.a = result,
                        _ => unreachable!(),
                    }
                    expected.f = Flags::from_bits_retain(result_flags.bits());
                    expected.f.set(Flags::CARRY, cpu.f.contains(Flags::CARRY));
                    expected.pc = pc.wrapping_add(1);

                    cpu.step(&mut bus);

                    assert_eq!(
                        cpu, expected,
                        "destination={destination}, value={value:02X}, PC={pc:04X}, initial flags={flags:X}"
                    );
                    assert_eq!(bus.read(pc), opcode);
                    assert_eq!(
                        bus.read(address),
                        if destination == 6 { result } else { 0x66 }
                    );
                }
            }
        }
    }
}
