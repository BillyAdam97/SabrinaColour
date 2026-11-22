use super::seeded_cpu;
use crate::bus::Bus;

#[test]
fn call_and_ret_round_trip() {
    let mut cpu = seeded_cpu(0x8123, 0xF);
    let mut bus = Bus::new();
    bus.write(0x8123, 0xCD); // CALL 0xC146
    bus.write(0x8124, 0x46);
    bus.write(0x8125, 0xC1);
    bus.write(0xC146, 0xC9); // RET

    cpu.step(&mut bus);

    let mut expected = seeded_cpu(0xC146, 0xF);
    expected.sp = 0xD133;
    assert_eq!(cpu, expected);
    assert_eq!(bus.read(0xD133), 0x26);
    assert_eq!(bus.read(0xD134), 0x81);

    cpu.step(&mut bus);

    assert_eq!(cpu, seeded_cpu(0x8126, 0xF));
}

#[test]
fn ret_pops_the_return_address() {
    let mut cpu = seeded_cpu(0x8123, 0xF);
    let mut bus = Bus::new();
    bus.write(0x8123, 0xC9);
    bus.write(0xD135, 0x46);
    bus.write(0xD136, 0xC1);

    cpu.step(&mut bus);

    let mut expected = seeded_cpu(0xC146, 0xF);
    expected.sp = 0xD137;
    assert_eq!(cpu, expected);
    assert_eq!(bus.read(0xD135), 0x46);
    assert_eq!(bus.read(0xD136), 0xC1);
}

#[test]
fn call_cond_imm16() {
    // Flag masks use the same ZNHC nibble passed to seeded_cpu.
    for (opcode, flag_mask, when_set) in [
        (0xC4, 0x8, false), // NZ
        (0xCC, 0x8, true),  // Z
        (0xD4, 0x1, false), // NC
        (0xDC, 0x1, true),  // C
    ] {
        for pc in [0x8123u16, 0xFFFE] {
            for flags in 0..16 {
                let mut cpu = seeded_cpu(pc, flags);
                let mut bus = Bus::new();
                bus.write(pc, opcode);
                bus.write(pc.wrapping_add(1), 0x46);
                bus.write(pc.wrapping_add(2), 0xC1);
                bus.write(0xD133, 0x5A);
                bus.write(0xD134, 0xA5);

                let return_pc = pc.wrapping_add(3);
                let taken = (flags & flag_mask != 0) == when_set;
                let mut expected = seeded_cpu(return_pc, flags);
                let mut stack_bytes = [0x5A, 0xA5];
                if taken {
                    expected.pc = 0xC146;
                    expected.sp = 0xD133;
                    stack_bytes = return_pc.to_le_bytes();
                }

                cpu.step(&mut bus);

                let case = format!("opcode={opcode:02X}, PC={pc:04X}, flags={flags:X}");
                assert_eq!(cpu, expected, "{case}");
                assert_eq!([bus.read(0xD133), bus.read(0xD134)], stack_bytes, "{case}");
            }
        }
    }
}

#[test]
fn ret_cond() {
    for (opcode, flag_mask, when_set) in [
        (0xC0, 0x8, false), // NZ
        (0xC8, 0x8, true),  // Z
        (0xD0, 0x1, false), // NC
        (0xD8, 0x1, true),  // C
    ] {
        for sp in [0xD135u16, 0xFFFF] {
            for flags in 0..16 {
                let mut cpu = seeded_cpu(0x8123, flags);
                let mut bus = Bus::new();
                cpu.sp = sp;
                let high_address = sp.wrapping_add(1);
                bus.write(0x8123, opcode);
                bus.write(sp, 0x46);
                bus.write(high_address, 0xC1);

                let taken = (flags & flag_mask != 0) == when_set;
                let mut expected = seeded_cpu(0x8124, flags);
                expected.sp = sp;
                if taken {
                    expected.pc = 0xC146;
                    expected.sp = sp.wrapping_add(2);
                }

                cpu.step(&mut bus);

                let case = format!("opcode={opcode:02X}, SP={sp:04X}, flags={flags:X}");
                assert_eq!(cpu, expected, "{case}");
                assert_eq!(bus.read(sp), 0x46, "{case}");
                assert_eq!(bus.read(high_address), 0xC1, "{case}");
            }
        }
    }
}

#[test]
fn rst_pushes_the_return_address_and_jumps_to_its_vector() {
    for (opcode, vector) in [
        (0xC7, 0x0000),
        (0xCF, 0x0008),
        (0xD7, 0x0010),
        (0xDF, 0x0018),
        (0xE7, 0x0020),
        (0xEF, 0x0028),
        (0xF7, 0x0030),
        (0xFF, 0x0038),
    ] {
        let mut cpu = seeded_cpu(0x8123, 0xF);
        let mut bus = Bus::new();
        bus.write(0x8123, opcode);

        cpu.step(&mut bus);

        let mut expected = seeded_cpu(vector, 0xF);
        expected.sp = 0xD133;
        assert_eq!(bus.read(0xD133), 0x24, "opcode={opcode:02X}");
        assert_eq!(bus.read(0xD134), 0x81, "opcode={opcode:02X}");
        assert_eq!(cpu, expected, "opcode={opcode:02X}");
    }
}

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
