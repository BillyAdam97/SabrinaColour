use super::{Cpu, DEST_R8, DEST_R16, Flags, R16_HL};
use crate::bus::Bus;

impl Cpu {
    pub(super) fn add_a_u8(&mut self, u8_val: u8) {
        let hc = (self.a as u16 & 0xF) + (u8_val as u16 & 0xF) > 0xF;
        let c = self.a as u16 + u8_val as u16 > 0xFF;

        self.a = self.a.wrapping_add(u8_val);

        self.f.set(Flags::ZERO, self.a == 0);
        self.f.set(Flags::SUB, false);
        self.f.set(Flags::HALF, hc);
        self.f.set(Flags::CARRY, c);
    }

    pub(super) fn add_hl_u16(&mut self, u16_val: u16) {
        let hl = self.get_r16(R16_HL);
        let hc = (hl & 0xFFF) + (u16_val & 0xFFF) > 0xFFF;
        let c = hl as u32 + u16_val as u32 > 0xFFFF;

        self.set_r16(R16_HL, hl.wrapping_add(u16_val));

        self.f.set(Flags::SUB, false);
        self.f.set(Flags::HALF, hc);
        self.f.set(Flags::CARRY, c);
    }

    pub(super) fn add_sp_imm8(&mut self, u8_val: u8) {
        let hc = (self.sp & 0xF) + (u8_val as u16 & 0xF) > 0xF;
        let c = (self.sp & 0xFF) + u8_val as u16 > 0xFF;

        let imm8 = u8_val as i8 as i16 as u16;
        self.sp = self.sp.wrapping_add(imm8);

        self.f.set(Flags::ZERO, false);
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

    pub(super) fn and_a_u8(&mut self, u8_val: u8) {
        self.a &= u8_val;

        self.f.set(Flags::ZERO, self.a == 0);
        self.f.set(Flags::SUB, false);
        self.f.set(Flags::HALF, true);
        self.f.set(Flags::CARRY, false);
    }

    pub(super) fn xor_a_u8(&mut self, u8_val: u8) {
        self.a ^= u8_val;

        self.f.set(Flags::ZERO, self.a == 0);
        self.f.set(Flags::SUB, false);
        self.f.set(Flags::HALF, false);
        self.f.set(Flags::CARRY, false);
    }

    pub(super) fn or_a_u8(&mut self, u8_val: u8) {
        self.a |= u8_val;

        self.f.set(Flags::ZERO, self.a == 0);
        self.f.set(Flags::SUB, false);
        self.f.set(Flags::HALF, false);
        self.f.set(Flags::CARRY, false);
    }

    pub(super) fn cp_a_u8(&mut self, u8_val: u8) {
        let hc = (self.a & 0xF) < (u8_val & 0xF);
        let c = self.a < u8_val;

        let res = self.a.wrapping_sub(u8_val);

        self.f.set(Flags::ZERO, res == 0);
        self.f.set(Flags::SUB, true);
        self.f.set(Flags::HALF, hc);
        self.f.set(Flags::CARRY, c);
    }

    pub(super) fn inc_r8(&mut self, bus: &mut Bus, opcode: u8) {
        let code = (opcode & DEST_R8) >> 3;
        let r8 = self.get_r8(bus, code);
        let hc = (r8 & 0xF) + 1 > 0xF;

        let r8 = r8.wrapping_add(1);
        self.set_r8(bus, code, r8);

        self.f.set(Flags::ZERO, r8 == 0);
        self.f.set(Flags::SUB, false);
        self.f.set(Flags::HALF, hc);
    }

    pub(super) fn inc_r16(&mut self, opcode: u8) {
        let code = (opcode & DEST_R16) >> 4;
        let r16 = self.get_r16(code);
        self.set_r16(code, r16.wrapping_add(1));
    }

    pub(super) fn dec_r8(&mut self, bus: &mut Bus, opcode: u8) {
        let code = (opcode & DEST_R8) >> 3;
        let r8 = self.get_r8(bus, code);
        let hc = (r8 & 0xF) < 1;

        let r8 = r8.wrapping_sub(1);
        self.set_r8(bus, code, r8);

        self.f.set(Flags::ZERO, r8 == 0);
        self.f.set(Flags::SUB, true);
        self.f.set(Flags::HALF, hc);
    }

    pub(super) fn dec_r16(&mut self, opcode: u8) {
        let code = (opcode & DEST_R16) >> 4;
        let r16 = self.get_r16(code);
        self.set_r16(code, r16.wrapping_sub(1));
    }
}
