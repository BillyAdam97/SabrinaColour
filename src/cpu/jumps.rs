use super::Cpu;
use crate::bus::Bus;

impl Cpu {
    pub(super) fn jp_u16(&mut self, addr: u16) {
        self.pc = addr;
    }

    pub(super) fn jp_cc_u16(&mut self, cond: u8, addr: u16) {
        if self.f.check_cond(cond) {
            self.jp_u16(addr);
        }
    }

    pub(super) fn jr_i8(&mut self, addr: i8) {
        self.pc = self.pc.wrapping_add_signed(addr as i16);
    }

    pub(super) fn call_u16(&mut self, bus: &mut Bus, addr: u16) {
        self.push_r16(bus, self.pc);
        self.jp_u16(addr);
    }

    pub(super) fn call_cc_u16(&mut self, bus: &mut Bus, cond: u8, addr: u16) {
        if self.f.check_cond(cond) {
            self.call_u16(bus, addr);
        }
    }

    pub(super) fn ret_u16(&mut self, bus: &Bus) {
        self.pc = self.pop_r16(bus);
    }

    pub(super) fn ret_cc_u16(&mut self, bus: &Bus, cond: u8) {
        if self.f.check_cond(cond) {
            self.ret_u16(bus);
        }
    }

    pub(super) fn reti(&mut self, bus: &Bus) {
        self.ret_u16(bus);
        self.ime = true;
    }

    pub(super) fn rst_vec(&mut self, bus: &mut Bus, addr: u16) {
        self.call_u16(bus, addr);
    }
}
