use super::seeded_cpu;
use crate::bus::Bus;
use crate::cpu::registers::Flags;

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
