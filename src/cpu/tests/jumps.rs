use super::seeded_cpu;
use crate::bus::Bus;

#[test]
fn jp_imm16() {
    for target in [0x0000u16, 0x1234, 0x8000, 0xFFFE, 0xFFFF] {
        for pc in [0x0000u16, 0x8000, 0xFFFD, 0xFFFE, 0xFFFF] {
            for flags in 0..16 {
                let mut cpu = seeded_cpu(pc, flags);
                let mut bus = Bus::new();
                let low_address = pc.wrapping_add(1);
                let high_address = pc.wrapping_add(2);
                bus.write(pc, 0xC3);
                bus.write(low_address, target as u8);
                bus.write(high_address, (target >> 8) as u8);
                bus.write(pc.wrapping_add(3), 0xD3); // Must not execute another instruction.
                bus.write(0xC030, 0x66);

                let mut expected = seeded_cpu(pc, flags);
                expected.pc = target;

                cpu.step(&mut bus);

                assert_eq!(
                    cpu.pc, target,
                    "target={target:04X}, PC={pc:04X}, initial flags={flags:X}"
                );
                assert_eq!(cpu, expected, "PC={pc:04X}, initial flags={flags:X}");
                assert_eq!(bus.read(pc), 0xC3);
                assert_eq!(bus.read(low_address), target as u8);
                assert_eq!(bus.read(high_address), (target >> 8) as u8);
                assert_eq!(bus.read(pc.wrapping_add(3)), 0xD3);
                assert_eq!(bus.read(0xC030), 0x66);
            }
        }
    }
}

#[test]
fn jp_cond_imm16() {
    // Flag masks use the same ZNHC nibble passed to seeded_cpu.
    for (opcode, flag_mask, when_set) in [
        (0xC2, 0x8, false), // NZ
        (0xCA, 0x8, true),  // Z
        (0xD2, 0x1, false), // NC
        (0xDA, 0x1, true),  // C
    ] {
        for target in [0x0000u16, 0x1234, 0x8000, 0xFFFE, 0xFFFF] {
            for pc in [0x0000u16, 0x8000, 0xFFFD, 0xFFFE, 0xFFFF] {
                for flags in 0..16 {
                    let mut cpu = seeded_cpu(pc, flags);
                    let mut bus = Bus::new();
                    let low_address = pc.wrapping_add(1);
                    let high_address = pc.wrapping_add(2);
                    let following_address = pc.wrapping_add(3);
                    bus.write(pc, opcode);
                    bus.write(low_address, target as u8);
                    bus.write(high_address, (target >> 8) as u8);
                    bus.write(following_address, 0xD3); // Must not execute the fallthrough instruction.
                    bus.write(0xC030, 0x66);

                    let taken = (flags & flag_mask != 0) == when_set;
                    let mut expected = seeded_cpu(pc, flags);
                    expected.pc = if taken { target } else { following_address };

                    cpu.step(&mut bus);

                    assert_eq!(
                        cpu.pc, expected.pc,
                        "opcode={opcode:02X}, target={target:04X}, taken={taken}, PC={pc:04X}, initial flags={flags:X}"
                    );
                    assert_eq!(cpu, expected, "PC={pc:04X}, initial flags={flags:X}");
                    assert_eq!(bus.read(pc), opcode);
                    assert_eq!(bus.read(low_address), target as u8);
                    assert_eq!(bus.read(high_address), (target >> 8) as u8);
                    assert_eq!(bus.read(following_address), 0xD3);
                    assert_eq!(bus.read(0xC030), 0x66);
                }
            }
        }
    }
}

#[test]
fn jp_hl() {
    for target in [0x0000u16, 0x1234, 0x8000, 0xFFFE, 0xFFFF] {
        for pc in [0x0000u16, 0x8000, 0xFFFE, 0xFFFF] {
            for flags in 0..16 {
                let mut cpu = seeded_cpu(pc, flags);
                let mut bus = Bus::new();
                [cpu.h, cpu.l] = target.to_be_bytes();
                bus.write(target, 0xD3); // JP HL uses the address itself, not the byte at [HL].
                bus.write(pc, 0xE9);

                let mut expected = seeded_cpu(pc, flags);
                [expected.h, expected.l] = target.to_be_bytes();
                expected.pc = target;

                cpu.step(&mut bus);

                assert_eq!(
                    cpu.pc, target,
                    "HL={target:04X}, PC={pc:04X}, initial flags={flags:X}"
                );
                assert_eq!(cpu, expected, "PC={pc:04X}, initial flags={flags:X}");
                assert_eq!(bus.read(pc), 0xE9);
                assert_eq!(bus.read(target), if target == pc { 0xE9 } else { 0xD3 });
            }
        }
    }
}

#[test]
fn jr_imm8() {
    // Targets are relative to PC after both instruction bytes have been fetched.
    for (pc, offset, target) in [
        (0x8000u16, 0x00, 0x8002),
        (0x8000, 0x01, 0x8003),
        (0x8000, 0x7F, 0x8081),
        (0x8000, 0x80, 0x7F82),
        (0x8000, 0xFF, 0x8001),
        (0x8000, 0xFE, 0x8000),
        (0x0000, 0x80, 0xFF82),
        (0x0000, 0xFD, 0xFFFF),
        (0x0000, 0xFE, 0x0000),
        (0x007E, 0x80, 0x0000),
        (0xFF7F, 0x7F, 0x0000),
        (0xFFFE, 0x00, 0x0000),
        (0xFFFE, 0xFF, 0xFFFF),
        (0xFFFF, 0x00, 0x0001),
        (0xFFFF, 0x7F, 0x0080),
        (0xFFFF, 0x80, 0xFF81),
        (0xFFFF, 0xFF, 0x0000),
        (0xFFFF, 0xFE, 0xFFFF),
    ] {
        for flags in 0..16 {
            let mut cpu = seeded_cpu(pc, flags);
            let mut bus = Bus::new();
            let operand_address = pc.wrapping_add(1);
            let following_address = pc.wrapping_add(2);
            bus.write(pc, 0x18);
            bus.write(operand_address, offset);
            bus.write(following_address, 0xD3); // Must not execute another instruction.
            bus.write(0xC030, 0x66);

            let mut expected = seeded_cpu(pc, flags);
            expected.pc = target;

            cpu.step(&mut bus);

            assert_eq!(
                cpu.pc, target,
                "offset={offset:02X}, target={target:04X}, PC={pc:04X}, initial flags={flags:X}"
            );
            assert_eq!(cpu, expected, "PC={pc:04X}, initial flags={flags:X}");
            assert_eq!(bus.read(pc), 0x18);
            assert_eq!(bus.read(operand_address), offset);
            assert_eq!(bus.read(following_address), 0xD3);
            assert_eq!(bus.read(0xC030), 0x66);
        }
    }
}

#[test]
fn jr_cond_imm8() {
    // Flag masks use the same ZNHC nibble passed to seeded_cpu.
    for (opcode, flag_mask, when_set) in [
        (0x20, 0x8, false), // NZ
        (0x28, 0x8, true),  // Z
        (0x30, 0x1, false), // NC
        (0x38, 0x1, true),  // C
    ] {
        for (pc, offset, target) in [
            (0x8000u16, 0x00, 0x8002),
            (0x8000, 0x01, 0x8003),
            (0x8000, 0x7F, 0x8081),
            (0x8000, 0x80, 0x7F82),
            (0x8000, 0xFF, 0x8001),
            (0x8000, 0xFE, 0x8000),
            (0x0000, 0x80, 0xFF82),
            (0x0000, 0xFD, 0xFFFF),
            (0x0000, 0xFE, 0x0000),
            (0x007E, 0x80, 0x0000),
            (0xFF7F, 0x7F, 0x0000),
            (0xFFFE, 0x00, 0x0000),
            (0xFFFE, 0xFF, 0xFFFF),
            (0xFFFF, 0x00, 0x0001),
            (0xFFFF, 0x7F, 0x0080),
            (0xFFFF, 0x80, 0xFF81),
            (0xFFFF, 0xFF, 0x0000),
            (0xFFFF, 0xFE, 0xFFFF),
        ] {
            for flags in 0..16 {
                let mut cpu = seeded_cpu(pc, flags);
                let mut bus = Bus::new();
                let operand_address = pc.wrapping_add(1);
                let following_address = pc.wrapping_add(2);
                bus.write(pc, opcode);
                bus.write(operand_address, offset);
                bus.write(following_address, 0xD3); // Must not execute the fallthrough instruction.
                bus.write(0xC030, 0x66);

                let taken = (flags & flag_mask != 0) == when_set;
                let mut expected = seeded_cpu(pc, flags);
                expected.pc = if taken { target } else { following_address };

                cpu.step(&mut bus);

                assert_eq!(
                    cpu.pc, expected.pc,
                    "opcode={opcode:02X}, offset={offset:02X}, target={target:04X}, taken={taken}, PC={pc:04X}, initial flags={flags:X}"
                );
                assert_eq!(cpu, expected, "PC={pc:04X}, initial flags={flags:X}");
                assert_eq!(bus.read(pc), opcode);
                assert_eq!(bus.read(operand_address), offset);
                assert_eq!(bus.read(following_address), 0xD3);
                assert_eq!(bus.read(0xC030), 0x66);
            }
        }
    }
}
