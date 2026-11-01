use crate::bus::*;
use crate::cpu::*;
use rstest::rstest;

struct CpuTestCase {
    initial: Cpu,
    expected: Cpu,
}

#[rstest]
#[case::basic(
    CpuTestCase {
        initial: Cpu {
            a: 0,
            f: Flags::empty(),
            b: 1,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            pc: 0,
            sp: 0,
        },
        expected: Cpu {
            a: 0,
            f: Flags::empty(),
            b: 1,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            pc: 1,
            sp: 0,
        },
    },
    {
        let mut bus = Bus::new();
        bus.write(0x00, 0x40);
        bus
    })]
fn test_ld_r8_r8(#[case] test_case: CpuTestCase, #[case] mut bus: Bus) {
    let mut cpu = test_case.initial;
    cpu.step(&mut bus);
    assert_eq!(cpu, test_case.expected)
}
