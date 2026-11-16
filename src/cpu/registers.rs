use super::{Cpu, R16_HL};
use super::Bus;

use bitflags::bitflags;

bitflags! {
    #[derive(Debug,PartialEq)]
    pub(super) struct Flags: u8 {
        const ZERO  = 0b1000_0000;
        const SUB   = 0b0100_0000;
        const HALF  = 0b0010_0000;
        const CARRY = 0b0001_0000;
    }
}

impl Flags {
    #[allow(dead_code)]
    pub(super) fn check_cond(&self, code: u8) -> bool {
        match code {
            0 => !self.contains(Flags::ZERO),
            1 => self.contains(Flags::ZERO),
            2 => !self.contains(Flags::CARRY),
            3 => self.contains(Flags::CARRY),
            _ => panic!("Outside expected range of 0-3 for condition decoding"),
        }
    }
}

impl Cpu {
 
    pub(super) fn get_r8(&self, bus: &Bus, code: u8) -> u8 {
        match code {
            0 => self.b,
            1 => self.c,
            2 => self.d,
            3 => self.e,
            4 => self.h,
            5 => self.l,
            6 => self.read(bus, self.get_r16(R16_HL)), // [HL]
            7 => self.a,
            _ => panic!("Outside expected range of 0-7 for r8 decoding"),
        }
    }

    pub(super) fn set_r8(&mut self, bus: &mut Bus, code: u8, val: u8) {
        match code {
            0 => self.b = val,
            1 => self.c = val,
            2 => self.d = val,
            3 => self.e = val,
            4 => self.h = val,
            5 => self.l = val,
            6 => self.write(bus, self.get_r16(R16_HL), val), // [HL]
            7 => self.a = val,
            _ => panic!("Outside expected range of 0-7 for r8 decoding"),
        }
    }

    pub(super) fn get_r16(&self, code: u8) -> u16 {
        match code {
            0 => ((self.b as u16) << 8) | self.c as u16,
            1 => ((self.d as u16) << 8) | self.e as u16,
            2 => ((self.h as u16) << 8) | self.l as u16,
            3 => self.sp,
            _ => panic!("Outside expected range of 0-3 for r16 decoding"),
        }
    }

    pub(super) fn set_r16(&mut self, code: u8, val: u16) {
        let hi: u8 = (val >> 8) as u8;
        let lo: u8 = val as u8;
        match code {
            0 => {
                self.b = hi;
                self.c = lo;
            }
            1 => {
                self.d = hi;
                self.e = lo;
            }
            2 => {
                self.h = hi;
                self.l = lo;
            }
            3 => self.sp = val,
            _ => panic!("Outside expected range of 0-3 for r16 decoding"),
        }
    }

    #[allow(dead_code)]
    pub(super) fn get_r16stk(&self, code: u8) -> u16 {
        match code {
            0 => ((self.b as u16) << 8) | self.c as u16,
            1 => ((self.d as u16) << 8) | self.e as u16,
            2 => ((self.h as u16) << 8) | self.l as u16,
            3 => ((self.a as u16) << 8) | (self.f.bits() as u16 & 0xF0),
            _ => panic!("Outside expected range of 0-3 for r16stk decoding"),
        }
    }

    #[allow(dead_code)]
    pub(super) fn set_r16stk(&mut self, code: u8, val: u16) {
        let hi: u8 = (val >> 8) as u8;
        let lo: u8 = val as u8;
        match code {
            0 => {
                self.b = hi;
                self.c = lo;
            }
            1 => {
                self.d = hi;
                self.e = lo;
            }
            2 => {
                self.h = hi;
                self.l = lo;
            }
            3 => {
                self.a = hi;
                self.f = Flags::from_bits_retain(lo & 0xF0);
            }
            _ => panic!("Outside expected range of 0-3 for r16stk decoding"),
        }
    }

    pub(super) fn get_r16mem(&mut self, bus: &Bus, code: u8) -> u8 {
        match code {
            0 | 1 => self.read(bus, self.get_r16(code)),
            2 | 3 => self.read(bus, self.get_r16(R16_HL)),
            _ => panic!("Outside expected range of 0-3 for r16mem decoding"),
        }
    }

    pub(super) fn set_r16mem(&mut self, bus: &mut Bus, code: u8, val: u8) {
        match code {
            0 | 1 => self.write(bus, self.get_r16(code), val),
            2 | 3 => self.write(bus, self.get_r16(R16_HL), val),
            _ => panic!("Outside expected range of 0-3 for r16mem decoding"),
        }
    }
}
