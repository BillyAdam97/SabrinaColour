use super::seeded_cpu;
use crate::bus::Bus;
use crate::cpu::registers::Flags;

#[test]
fn get_r8() {
    let mut cpu = seeded_cpu(0x8000, 0xF);
    let mut bus = Bus::new();
    [cpu.b, cpu.d] = [0x12, 0x34];
    bus.write(0xC030, 0x66);

    for (code, value) in [
        (0, 0x12),
        (1, 0x10),
        (2, 0x34),
        (3, 0x20),
        (4, 0xC0),
        (5, 0x30),
        (6, 0x66),
        (7, 0xA5),
    ] {
        assert_eq!(cpu.get_r8(&bus, code), value, "code={code}");
    }
}

#[test]
fn set_r8() {
    for code in 0..8 {
        for value in [0x00, 0x5A, 0xFF] {
            let mut cpu = seeded_cpu(0x8000, 0xF);
            let mut bus = Bus::new();
            bus.write(0xC030, 0x66);

            let mut expected = seeded_cpu(0x8000, 0xF);
            match code {
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

            cpu.set_r8(&mut bus, code, value);

            assert_eq!(cpu, expected, "code={code}, value={value:02X}");
            assert_eq!(bus.read(0xC030), if code == 6 { value } else { 0x66 });
        }
    }
}

#[test]
fn get_r16() {
    let cpu = seeded_cpu(0x8000, 0xF);

    for (code, value) in [(0, 0xC010), (1, 0xC020), (2, 0xC030), (3, 0xD135)] {
        assert_eq!(cpu.get_r16(code), value, "code={code}");
    }
}

#[test]
fn set_r16() {
    for code in 0..4 {
        for value in [0x0000u16, 0x1234, 0xFFFF] {
            let mut cpu = seeded_cpu(0x8000, 0xF);
            let mut expected = seeded_cpu(0x8000, 0xF);
            match code {
                0 => [expected.b, expected.c] = value.to_be_bytes(),
                1 => [expected.d, expected.e] = value.to_be_bytes(),
                2 => [expected.h, expected.l] = value.to_be_bytes(),
                3 => expected.sp = value,
                _ => unreachable!(),
            }

            cpu.set_r16(code, value);

            assert_eq!(cpu, expected, "code={code}, value={value:04X}");
        }
    }
}

#[test]
fn get_r16stk() {
    for flags in 0..16 {
        let mut cpu = seeded_cpu(0x8000, flags);
        cpu.f = Flags::from_bits_retain((flags << 4) | 0x0F);

        let af = 0xA500 | (u16::from(flags) << 4);
        for (code, value) in [(0, 0xC010), (1, 0xC020), (2, 0xC030), (3, af)] {
            assert_eq!(cpu.get_r16stk(code), value, "code={code}, flags={flags:X}");
        }
    }
}

#[test]
fn set_r16stk() {
    for code in 0..4 {
        for (value, flag_bits) in [(0x0000u16, 0x00), (0x12AF, 0xA0), (0xFFFF, 0xF0)] {
            let mut cpu = seeded_cpu(0x8000, 0x5);
            let mut expected = seeded_cpu(0x8000, 0x5);
            match code {
                0 => [expected.b, expected.c] = value.to_be_bytes(),
                1 => [expected.d, expected.e] = value.to_be_bytes(),
                2 => [expected.h, expected.l] = value.to_be_bytes(),
                3 => {
                    expected.a = value.to_be_bytes()[0];
                    expected.f = Flags::from_bits_retain(flag_bits);
                }
                _ => unreachable!(),
            }

            cpu.set_r16stk(code, value);

            assert_eq!(cpu, expected, "code={code}, value={value:04X}");
        }
    }
}

#[test]
fn get_r16mem() {
    let mut cpu = seeded_cpu(0x8000, 0xF);
    let mut bus = Bus::new();
    for (address, value) in [(0xC010, 0x12), (0xC020, 0x34), (0xC030, 0x56)] {
        bus.write(address, value);
    }

    for (code, value) in [(0, 0x12), (1, 0x34), (2, 0x56), (3, 0x56)] {
        assert_eq!(cpu.get_r16mem(&bus, code), value, "code={code}");
        assert_eq!(cpu, seeded_cpu(0x8000, 0xF), "code={code}");
    }
}

#[test]
fn set_r16mem() {
    for (code, target) in [(0, 0xC010), (1, 0xC020), (2, 0xC030), (3, 0xC030)] {
        for value in [0x00, 0x5A, 0xFF] {
            let mut cpu = seeded_cpu(0x8000, 0xF);
            let mut bus = Bus::new();
            let memory = [(0xC010, 0x12), (0xC020, 0x34), (0xC030, 0x56)];
            for (address, initial) in memory {
                bus.write(address, initial);
            }

            cpu.set_r16mem(&mut bus, code, value);

            assert_eq!(
                cpu,
                seeded_cpu(0x8000, 0xF),
                "code={code}, value={value:02X}"
            );
            for (address, initial) in memory {
                assert_eq!(
                    bus.read(address),
                    if address == target { value } else { initial },
                    "code={code}, address={address:04X}, value={value:02X}"
                );
            }
        }
    }
}

#[test]
fn check_cond() {
    for (code, mask, when_set) in [
        (0, 0x8, false),
        (1, 0x8, true),
        (2, 0x1, false),
        (3, 0x1, true),
    ] {
        for flags in 0..16 {
            let cpu = seeded_cpu(0x8000, flags);
            assert_eq!(
                cpu.f.check_cond(code),
                (flags & mask != 0) == when_set,
                "code={code}, flags={flags:X}"
            );
        }
    }
}

#[test]
#[should_panic(expected = "Outside expected range of 0-3 for condition decoding")]
fn invalid_condition_code() {
    Flags::empty().check_cond(4);
}
