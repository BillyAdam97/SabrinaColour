use super::seeded_cpu;
use crate::bus::Bus;
use crate::cpu::registers::Flags;

#[test]
fn push_r16stk() {
    let pc = 0x8000;
    for flags in 0..16 {
        let af = 0xA500 | (u16::from(flags) << 4);
        for (opcode, value) in [(0xC5, 0xC010), (0xD5, 0xC020), (0xE5, 0xC030), (0xF5, af)] {
            for sp in [0xD135u16, 0x0000, 0x0001] {
                let mut cpu = seeded_cpu(pc, flags);
                let mut bus = Bus::new();
                cpu.sp = sp;
                cpu.f = Flags::from_bits_retain((flags << 4) | 0x0F);
                let low_address = sp.wrapping_sub(2);
                let high_address = sp.wrapping_sub(1);
                let before_stack = sp.wrapping_sub(3);
                bus.write(pc, opcode);
                bus.write(before_stack, 0x5A);
                bus.write(low_address, 0x22);
                bus.write(high_address, 0x33);
                bus.write(sp, 0x66);

                let mut expected = seeded_cpu(pc, flags);
                expected.f = Flags::from_bits_retain(cpu.f.bits());
                expected.sp = low_address;
                expected.pc = pc.wrapping_add(1);
                let [low, high] = value.to_le_bytes();

                cpu.step(&mut bus);

                let case = format!("opcode={opcode:02X}, SP={sp:04X}, initial flags={flags:X}");
                assert_eq!(cpu, expected, "{case}");
                assert_eq!(bus.read(low_address), low, "{case}");
                assert_eq!(bus.read(high_address), high, "{case}");
                assert_eq!(bus.read(before_stack), 0x5A, "{case}");
                assert_eq!(bus.read(sp), 0x66, "{case}");
                assert_eq!(bus.read(pc), opcode, "{case}");
            }
        }
    }
}

#[test]
fn pop_r16stk() {
    let pc = 0x8000;
    for opcode in [0xC1, 0xD1, 0xE1, 0xF1] {
        for sp in [0xD133u16, 0xFFFE, 0xFFFF] {
            for flags in 0..16 {
                let mut cpu = seeded_cpu(pc, flags);
                let mut bus = Bus::new();
                cpu.sp = sp;
                let high_address = sp.wrapping_add(1);
                let high = 0x96;
                // Flip every flag and include nonzero bits in the unused nibble for POP AF.
                let low = ((flags ^ 0xF) << 4) | 0x0B;
                bus.write(pc, opcode);
                bus.write(sp, low);
                bus.write(high_address, high);

                let mut expected = seeded_cpu(pc, flags);
                match opcode {
                    0xC1 => [expected.b, expected.c] = [high, low],
                    0xD1 => [expected.d, expected.e] = [high, low],
                    0xE1 => [expected.h, expected.l] = [high, low],
                    0xF1 => {
                        expected.a = high;
                        expected.f = Flags::from_bits_retain(low & 0xF0);
                    }
                    _ => unreachable!(),
                }
                expected.sp = sp.wrapping_add(2);
                expected.pc = pc.wrapping_add(1);

                cpu.step(&mut bus);

                let case = format!("opcode={opcode:02X}, SP={sp:04X}, initial flags={flags:X}");
                assert_eq!(cpu, expected, "{case}");
                assert_eq!(bus.read(sp), low, "{case}");
                assert_eq!(bus.read(high_address), high, "{case}");
                assert_eq!(bus.read(pc), opcode, "{case}");
            }
        }
    }
}
