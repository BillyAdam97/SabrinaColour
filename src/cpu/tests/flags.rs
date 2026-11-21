use super::seeded_cpu;
use crate::bus::Bus;
use crate::cpu::registers::Flags;

fn assert_flag_instruction(opcode: u8, a: u8, flags: u8, result: u8, result_flags: Flags) {
    for pc in [0x8000u16, 0xFFFE, 0xFFFF] {
        let mut cpu = seeded_cpu(pc, flags);
        let mut bus = Bus::new();
        cpu.a = a;
        bus.write(pc, opcode);
        bus.write(0xC030, 0x66);

        let mut expected = seeded_cpu(pc, flags);
        expected.a = result;
        expected.f = Flags::from_bits_retain(result_flags.bits());
        expected.pc = pc.wrapping_add(1);

        cpu.step(&mut bus);

        assert_eq!(
            cpu, expected,
            "opcode={opcode:02X}, A={a:02X}, PC={pc:04X}, initial flags={flags:X}"
        );
        assert_eq!(bus.read(pc), opcode);
        assert_eq!(bus.read(0xC030), 0x66);
    }
}

#[test]
fn rlca() {
    for (a, result, result_flags) in [
        (0x80, 0x01, Flags::CARRY),
        (0x00, 0x00, Flags::empty()),
        (0x41, 0x82, Flags::empty()),
        (0xFF, 0xFF, Flags::CARRY),
    ] {
        for flags in 0..16 {
            assert_flag_instruction(
                0x07,
                a,
                flags,
                result,
                Flags::from_bits_retain(result_flags.bits()),
            );
        }
    }
}

#[test]
fn rrca() {
    for (a, result, result_flags) in [
        (0x01, 0x80, Flags::CARRY),
        (0x00, 0x00, Flags::empty()),
        (0x82, 0x41, Flags::empty()),
        (0xFF, 0xFF, Flags::CARRY),
    ] {
        for flags in 0..16 {
            assert_flag_instruction(
                0x0F,
                a,
                flags,
                result,
                Flags::from_bits_retain(result_flags.bits()),
            );
        }
    }
}

#[test]
fn rla() {
    for (a, shifted, carry_out) in [
        (0x80, 0x00, true),
        (0x00, 0x00, false),
        (0x41, 0x82, false),
        (0xFF, 0xFE, true),
    ] {
        let result_flags = if carry_out {
            Flags::CARRY
        } else {
            Flags::empty()
        };
        for flags in 0..16 {
            let result = shifted | (flags & 1);
            assert_flag_instruction(
                0x17,
                a,
                flags,
                result,
                Flags::from_bits_retain(result_flags.bits()),
            );
        }
    }
}

#[test]
fn rra() {
    for (a, shifted, carry_out) in [
        (0x01, 0x00, true),
        (0x00, 0x00, false),
        (0x82, 0x41, false),
        (0xFF, 0x7F, true),
    ] {
        let result_flags = if carry_out {
            Flags::CARRY
        } else {
            Flags::empty()
        };
        for flags in 0..16 {
            let result = shifted | ((flags & 1) << 7);
            assert_flag_instruction(
                0x1F,
                a,
                flags,
                result,
                Flags::from_bits_retain(result_flags.bits()),
            );
        }
    }
}

#[test]
fn daa_after_addition() {
    for (a, flags, result, result_flags) in [
        (0x9A, 0x0, 0x00, Flags::ZERO | Flags::CARRY), // Both digits need adjustment.
        (0x99, 0x8, 0x99, Flags::empty()),
        (0x00, 0x0, 0x00, Flags::ZERO),
        (0x0A, 0x0, 0x10, Flags::empty()),
        (0x12, 0x2, 0x18, Flags::empty()),
        (0xA0, 0x0, 0x00, Flags::ZERO | Flags::CARRY),
        (0x32, 0x1, 0x92, Flags::CARRY),
        (0x32, 0x3, 0x98, Flags::CARRY),
    ] {
        assert_flag_instruction(0x27, a, flags, result, result_flags);
    }
}

#[test]
fn daa_after_subtraction() {
    for (a, flags, result, result_flags) in [
        (0xFF, 0x7, 0x99, Flags::SUB | Flags::CARRY), // Both borrows need adjustment.
        (0x99, 0xC, 0x99, Flags::SUB),
        (0x00, 0x4, 0x00, Flags::ZERO | Flags::SUB),
        (0x0F, 0x6, 0x09, Flags::SUB),
        (0xF0, 0x5, 0x90, Flags::SUB | Flags::CARRY),
    ] {
        assert_flag_instruction(0x27, a, flags, result, result_flags);
    }
}

#[test]
fn cpl() {
    for (a, result) in [(0x00, 0xFF), (0xA5, 0x5A), (0xFF, 0x00)] {
        for flags in 0..16 {
            let result_flags =
                Flags::from_bits_retain((flags & 0x9) << 4) | Flags::SUB | Flags::HALF;
            assert_flag_instruction(0x2F, a, flags, result, result_flags);
        }
    }
}

#[test]
fn scf() {
    for flags in 0..16 {
        let result_flags = Flags::from_bits_retain((flags & 0x8) << 4) | Flags::CARRY;
        assert_flag_instruction(0x37, 0xA5, flags, 0xA5, result_flags);
    }
}

#[test]
fn ccf() {
    for flags in 0..16 {
        let result_flags = Flags::from_bits_retain(((flags & 0x9) ^ 1) << 4);
        assert_flag_instruction(0x3F, 0xA5, flags, 0xA5, result_flags);
    }
}
