#[cfg(test)]
mod tests;

use crate::bus::*;

use bitflags::bitflags;

bitflags! {
    #[derive(Debug,PartialEq)]
    struct Flags: u8 {
        const ZERO  = 0b1000_0000;
        const SUB   = 0b0100_0000;
        const HALF  = 0b0010_0000;
        const CARRY = 0b0001_0000;
    }
}

impl Flags {
    fn check_cond(&self, code: u8) -> bool {
        match code {
            0 => !self.contains(Flags::ZERO),
            1 => self.contains(Flags::ZERO),
            2 => !self.contains(Flags::CARRY),
            3 => self.contains(Flags::CARRY),
            _ => panic!("Outside expected range of 0-3 for condition decoding"),
        }
    }
}

const DEST_R8: u8 = 0b00111000;
const SRC_R8: u8 = 0b00000111;

#[derive(Debug, PartialEq)]
pub struct Cpu {
    a: u8,
    f: Flags,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    pc: u16,
    sp: u16,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            a: 0,
            f: Flags::empty(),
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            pc: 0,
            sp: 0,
        }
    }

    fn read(&self, bus: &Bus, addr: u16) -> u8 {
        bus.read(addr)
    }

    fn write(&self, bus: &mut Bus, addr: u16, val: u8) {
        bus.write(addr, val);
    }

    fn fetch_byte(&mut self, bus: &Bus) -> u8 {
        let b = self.read(bus, self.pc);
        self.pc = self.pc.wrapping_add(1);
        b
    }

    fn fetch_bytes(&mut self, bus: &Bus) -> u16 {
        let lo = self.fetch_byte(bus);
        let hi = self.fetch_byte(bus);
        ((hi as u16) << 8) | lo as u16
    }

    // Move register methods to register.rs sub module?
    fn get_r8(&self, bus: &Bus, code: u8) -> u8 {
        match code {
            0 => self.b,
            1 => self.c,
            2 => self.d,
            3 => self.e,
            4 => self.h,
            5 => self.l,
            6 => self.read(bus, self.get_r16(2)), // [HL]
            7 => self.a,
            _ => panic!("Outside expected range of 0-7 for r8 decoding"),
        }
    }

    fn set_r8(&mut self, bus: &mut Bus, code: u8, val: u8) {
        match code {
            0 => self.b = val,
            1 => self.c = val,
            2 => self.d = val,
            3 => self.e = val,
            4 => self.h = val,
            5 => self.l = val,
            6 => self.write(bus, self.get_r16(2), val), // [HL]
            7 => self.a = val,
            _ => panic!("Outside expected range of 0-7 for r8 decoding"),
        }
    }

    fn get_r16(&self, code: u8) -> u16 {
        match code {
            0 => ((self.b as u16) << 8) | self.c as u16,
            1 => ((self.d as u16) << 8) | self.e as u16,
            2 => ((self.h as u16) << 8) | self.l as u16,
            3 => self.sp,
            _ => panic!("Outside expected range of 0-3 for r16 decoding"),
        }
    }

    fn set_r16(&mut self, code: u8, val: u16) {
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

    fn get_r16stk(&self, code: u8) -> u16 {
        match code {
            0 => ((self.b as u16) << 8) | self.c as u16,
            1 => ((self.d as u16) << 8) | self.e as u16,
            2 => ((self.h as u16) << 8) | self.l as u16,
            3 => ((self.a as u16) << 8) | (self.f.bits() as u16 & 0xF0),
            _ => panic!("Outside expected range of 0-3 for r16stk decoding"),
        }
    }

    fn set_r16stk(&mut self, code: u8, val: u16) {
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

    fn get_r16mem(&mut self, bus: &Bus, code: u8) -> u8 {
        let res = match code {
            0 | 1 => self.read(bus, self.get_r16(code)),
            2 => {
                let r16 = self.get_r16(2);
                let res = self.read(bus, r16);
                self.set_r16(2, r16.wrapping_add(1));
                res
            }
            3 => {
                let r16 = self.get_r16(2);
                let res = self.read(bus, r16);
                self.set_r16(2, r16.wrapping_sub(1));
                res
            }
            _ => panic!("Outside expected range of 0-3 for r16bus decoding"),
        };

        return res;
    }

    // Instructions - Should move to instructions.rs sub module
    fn ld_r8_r8(&mut self, bus: &mut Bus, opcode: u8) {
        println!("{opcode:02X}");
        self.set_r8(bus, opcode & DEST_R8, self.get_r8(bus, opcode & SRC_R8));
    }

    pub fn step(&mut self, bus: &mut Bus) {
        let opcode: u8 = self.fetch_byte(bus);
        match opcode {
            // Implement instructions
            // LD r8 r8
            0x40..0x7F => {
                if opcode == 0x76 {
                    // Impl HALT
                }
                // call ld
                self.ld_r8_r8(bus, opcode);
            }
            _ => {
                todo!("Unknown opcode")
            }
        };
    }
}
