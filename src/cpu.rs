mod arithmetic;
mod flags;
mod jumps;
mod loads;
mod registers;
mod stack;
#[cfg(test)]
mod tests;

use crate::bus::*;
use crate::cpu::registers::Flags;

const DEST_R8: u8 = 0b00111000;
const SRC_R8: u8 = 0b00000111;
const DEST_R16: u8 = 0b00110000;
const R16_HL: u8 = 2;

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

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
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

    pub fn step(&mut self, bus: &mut Bus) {
        let opcode: u8 = self.fetch_byte(bus);
        match opcode {
            0x01 | 0x11 | 0x21 | 0x31 => {
                self.ld_r16_imm16(bus, opcode);
            }
            0x02 | 0x12 | 0x22 | 0x32 => {
                self.ld_r16mem_a(bus, opcode);
            }
            0x03 | 0x13 | 0x23 | 0x33 => {
                self.inc_r16(opcode);
            }
            0x04 | 0x0C | 0x14 | 0x1C | 0x24 | 0x2C | 0x34 | 0x3C => {
                self.inc_r8(bus, opcode);
            }
            0x05 | 0x0D | 0x15 | 0x1D | 0x25 | 0x2D | 0x35 | 0x3D => {
                self.dec_r8(bus, opcode);
            }
            0x06 | 0x16 | 0x26 | 0x36 | 0x0E | 0x1E | 0x2E | 0x3E => {
                self.ld_r8_imm8(bus, opcode);
            }
            0x07 => {
                self.rlca();
            }
            0x08 => {
                self.ld_imm16_sp(bus);
            }
            0x09 | 0x19 | 0x29 | 0x39 => {
                let u16 = self.get_r16((opcode & DEST_R16) >> 4);
                self.add_hl_u16(u16);
            }
            0x0A | 0x1A | 0x2A | 0x3A => {
                self.ld_a_r16mem(bus, opcode);
            }
            0x0B | 0x1B | 0x2B | 0x3B => {
                self.dec_r16(opcode);
            }
            0x0F => {
                self.rrca();
            }
            0x17 => {
                self.rla();
            }
            0x18 => {
                let imm8 = self.fetch_byte(bus) as i8;
                self.jr_i8(imm8);
            }
            0x1F => {
                self.rra();
            }
            0x20 => {
                let imm8 = self.fetch_byte(bus) as i8;
                let cond = self.f.contains(Flags::ZERO);
                if !cond {
                    self.jr_i8(imm8);
                }
            }
            0x27 => {
                self.daa();
            }
            0x28 => {
                let imm8 = self.fetch_byte(bus) as i8;
                let cond = self.f.contains(Flags::ZERO);
                if cond {
                    self.jr_i8(imm8);
                }
            }
            0x2F => {
                self.cpl();
            }
            0x30 => {
                let imm8 = self.fetch_byte(bus) as i8;
                let cond = self.f.contains(Flags::CARRY);
                if !cond {
                    self.jr_i8(imm8);
                }
            }
            0x37 => {
                self.scf();
            }
            0x38 => {
                let imm8 = self.fetch_byte(bus) as i8;
                let cond = self.f.contains(Flags::CARRY);
                if cond {
                    self.jr_i8(imm8);
                }
            }
            0x3F => {
                self.ccf();
            }
            0x40..=0x7F => {
                if opcode == 0x76 {
                    // Impl HALT
                    todo!("Implement");
                }
                self.ld_r8_r8(bus, opcode);
            }
            0x80..=0x87 => {
                let r8 = self.get_r8(bus, opcode & SRC_R8);
                self.add_a_u8(r8);
            }
            0x88..=0x8F => {
                let r8 = self.get_r8(bus, opcode & SRC_R8);
                self.adc_a_u8(r8);
            }
            0x90..=0x97 => {
                let r8 = self.get_r8(bus, opcode & SRC_R8);
                self.sub_a_u8(r8);
            }
            0x98..=0x9F => {
                let r8 = self.get_r8(bus, opcode & SRC_R8);
                self.sbc_a_u8(r8);
            }
            0xA0..=0xA7 => {
                let r8 = self.get_r8(bus, opcode & SRC_R8);
                self.and_a_u8(r8);
            }
            0xA8..=0xAF => {
                let r8 = self.get_r8(bus, opcode & SRC_R8);
                self.xor_a_u8(r8);
            }
            0xB0..=0xB7 => {
                let r8 = self.get_r8(bus, opcode & SRC_R8);
                self.or_a_u8(r8);
            }
            0xB8..=0xBF => {
                let r8 = self.get_r8(bus, opcode & SRC_R8);
                self.cp_a_u8(r8);
            }
            0xC1 | 0xD1 | 0xE1 | 0xF1 => {
                self.pop_r16stk(bus, (opcode & DEST_R16) >> 4);
            }
            // Should make a helper for the condition check and then group
            0xC2 => {
                let imm16 = self.fetch_bytes(bus);
                let cond = self.f.contains(Flags::ZERO);
                if !cond {
                    self.jp_u16(imm16);
                }
            }
            0xC3 => {
                let imm16 = self.fetch_bytes(bus);
                self.jp_u16(imm16);
            }
            0xC5 | 0xD5 | 0xE5 | 0xF5 => {
                self.push_r16stk(bus, (opcode & DEST_R16) >> 4);
            }
            0xC6 => {
                let imm8 = self.fetch_byte(bus);
                self.add_a_u8(imm8);
            }
            0xCA => {
                let imm16 = self.fetch_bytes(bus);
                let cond = self.f.contains(Flags::ZERO);
                if cond {
                    self.jp_u16(imm16);
                }
            }
            0xCE => {
                let imm8 = self.fetch_byte(bus);
                self.adc_a_u8(imm8);
            }
            0xD2 => {
                let imm16 = self.fetch_bytes(bus);
                let cond = self.f.contains(Flags::CARRY);
                if !cond {
                    self.jp_u16(imm16);
                }
            }
            0xD6 => {
                let imm8 = self.fetch_byte(bus);
                self.sub_a_u8(imm8);
            }
            0xDA => {
                let imm16 = self.fetch_bytes(bus);
                let cond = self.f.contains(Flags::CARRY);
                if cond {
                    self.jp_u16(imm16);
                }
            }
            0xDE => {
                let imm8 = self.fetch_byte(bus);
                self.sbc_a_u8(imm8);
            }
            0xE6 => {
                let imm8 = self.fetch_byte(bus);
                self.and_a_u8(imm8);
            }
            0xE8 => {
                let imm8 = self.fetch_byte(bus);
                self.add_sp_imm8(imm8);
            }
            0xE9 => {
                self.jp_u16(self.get_r16(R16_HL));
            }
            0xEE => {
                let imm8 = self.fetch_byte(bus);
                self.xor_a_u8(imm8);
            }
            0xF6 => {
                let imm8 = self.fetch_byte(bus);
                self.or_a_u8(imm8);
            }
            0xFE => {
                let imm8 = self.fetch_byte(bus);
                self.cp_a_u8(imm8);
            }
            0xE0 => {
                let imm8 = self.fetch_byte(bus);
                self.ld_mem_a(bus, 0xFF00 + imm8 as u16);
            }
            0xE2 => {
                self.ld_mem_a(bus, 0xFF00 + self.c as u16);
            }
            0xEA => {
                let imm16 = self.fetch_bytes(bus);
                self.ld_mem_a(bus, imm16);
            }
            0xF0 => {
                let imm8 = self.fetch_byte(bus);
                self.ld_a(bus, 0xFF00 + imm8 as u16);
            }
            0xF2 => {
                self.ld_a(bus, 0xFF00 + self.c as u16);
            }
            0xF8 => {
                self.ld_hl_sp_imm8(bus);
            }
            0xF9 => {
                self.ld_sp_hl(opcode);
            }
            0xFA => {
                let imm16 = self.fetch_bytes(bus);
                self.ld_a(bus, imm16);
            }
            _ => {
                todo!("Unknown opcode")
            }
        };
    }
}
