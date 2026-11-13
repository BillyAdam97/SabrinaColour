use super::{Cpu, DEST_R8, DEST_R16, Flags, R16_HL, SRC_R8};
use crate::bus::Bus;

impl Cpu {
    // Loads
    pub(super) fn ld_r8_r8(&mut self, bus: &mut Bus, opcode: u8) {
        self.set_r8(
            bus,
            (opcode & DEST_R8) >> 3,
            self.get_r8(bus, opcode & SRC_R8),
        );
    }

    pub(super) fn ld_r16_imm16(&mut self, bus: &mut Bus, opcode: u8) {
        let imm16 = self.fetch_bytes(bus);
        self.set_r16((opcode & DEST_R16) >> 4, imm16);
    }

    pub(super) fn ld_r16mem_a(&mut self, bus: &mut Bus, opcode: u8) {
        let code = (opcode & DEST_R16) >> 4;
        self.set_r16mem(bus, code, self.a);
        if code == R16_HL {
            let hl = self.get_r16(R16_HL);
            self.set_r16(R16_HL, hl.wrapping_add(1));
        } else if code == 3 {
            let hl = self.get_r16(R16_HL);
            self.set_r16(R16_HL, hl.wrapping_sub(1));
        }
    }

    pub(super) fn ld_a_r16mem(&mut self, bus: &Bus, opcode: u8) {
        let code = (opcode & DEST_R16) >> 4;
        self.a = self.get_r16mem(bus, code);
        if code == R16_HL {
            let hl = self.get_r16(R16_HL);
            self.set_r16(R16_HL, hl.wrapping_add(1));
        } else if code == 3 {
            let hl = self.get_r16(R16_HL);
            self.set_r16(R16_HL, hl.wrapping_sub(1));
        }
    }

    pub(super) fn ld_imm16_sp(&mut self, bus: &mut Bus) {
        let imm16 = self.fetch_bytes(bus);
        self.write(bus, imm16, self.sp as u8);
        self.write(bus, imm16.wrapping_add(1), (self.sp >> 8) as u8)
    }

    pub(super) fn ld_r8_imm8(&mut self, bus: &mut Bus, opcode: u8) {
        let imm8 = self.fetch_byte(bus);
        self.set_r8(bus, (opcode & DEST_R8) >> 3, imm8);
    }

    pub(super) fn ld_mem_a(&mut self, bus: &mut Bus, addr: u16) {
        self.write(bus, addr, self.a);
    }

    pub(super) fn ld_a(&mut self, bus: &Bus, addr: u16) {
        self.a = self.read(bus, addr);
    }

    pub(super) fn ld_hl_sp_imm8(&mut self, bus: &Bus) {
        let imm8 = self.fetch_byte(bus);
        let hc = (self.sp & 0xF) + (imm8 as u16 & 0xF) > 0xF;
        let c = (self.sp & 0xFF) + imm8 as u16 > 0xFF;

        self.set_r16(R16_HL, self.sp.wrapping_add((imm8 as i8 as i16) as u16));

        self.f.set(Flags::ZERO, false);
        self.f.set(Flags::SUB, false);
        self.f.set(Flags::HALF, hc);
        self.f.set(Flags::CARRY, c);
    }

    pub(super) fn ld_sp_hl(&mut self, opcode: u8) {
        self.set_r16((opcode & DEST_R16) >> 4, self.get_r16(R16_HL));
    }

    // Arithmetic
    pub(super) fn add_a_u8(&mut self, u8_val: u8) {
        let hc = (self.a as u16 & 0xF) + (u8_val as u16 & 0xF) > 0xF;
        let c = self.a as u16 + u8_val as u16 > 0xFF;

        self.a = self.a.wrapping_add(u8_val);

        self.f.set(Flags::ZERO, self.a == 0);
        self.f.set(Flags::SUB, false);
        self.f.set(Flags::HALF, hc);
        self.f.set(Flags::CARRY, c);
    }

    pub(super) fn adc_a_u8(&mut self, u8_val: u8) {
        let oc = u8::from(self.f.contains(Flags::CARRY));

        let hc = (self.a as u16 & 0xF) + (u8_val as u16 & 0xF) + oc as u16 > 0xF;
        let c = self.a as u16 + u8_val as u16 + oc as u16 > 0xFF;

        self.a = self.a.wrapping_add(u8_val).wrapping_add(oc);

        self.f.set(Flags::ZERO, self.a == 0);
        self.f.set(Flags::SUB, false);
        self.f.set(Flags::HALF, hc);
        self.f.set(Flags::CARRY, c);
    }

    pub(super) fn sub_a_u8(&mut self, u8_val: u8) {
        let hc = (self.a & 0xF) < (u8_val & 0xF);
        let c = self.a < u8_val;

        self.a = self.a.wrapping_sub(u8_val);

        self.f.set(Flags::ZERO, self.a == 0);
        self.f.set(Flags::SUB, true);
        self.f.set(Flags::HALF, hc);
        self.f.set(Flags::CARRY, c);
    }

    pub(super) fn sbc_a_u8(&mut self, u8_val: u8) {
        let oc = u8::from(self.f.contains(Flags::CARRY));

        let hc = ((self.a & 0xF) as u16) < ((u8_val & 0xF) as u16) + oc as u16;
        let c = (self.a as u16) < u8_val as u16 + oc as u16;

        self.a = self.a.wrapping_sub(u8_val).wrapping_sub(oc);

        self.f.set(Flags::ZERO, self.a == 0);
        self.f.set(Flags::SUB, true);
        self.f.set(Flags::HALF, hc);
        self.f.set(Flags::CARRY, c);
    }
}
