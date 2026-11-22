use crate::bus::Bus;

use super::Cpu;

impl Cpu {
    pub(super) fn push_r16(&mut self, bus: &mut Bus, val: u16) {
        self.sp = self.sp.wrapping_sub(1);
        self.write(bus, self.sp, (val >> 8) as u8);
        self.sp = self.sp.wrapping_sub(1);
        self.write(bus, self.sp, val as u8);
    }

    pub(super) fn push_r16stk(&mut self, bus: &mut Bus, code: u8) {
        let r16: u16 = self.get_r16stk(code);
        self.push_r16(bus, r16);
    }

    pub(super) fn pop_r16(&mut self, bus: &Bus) -> u16 {
        let r16: u16 = self.read(bus, self.sp) as u16;
        self.sp = self.sp.wrapping_add(1);
        let r16: u16 = ((self.read(bus, self.sp) as u16) << 8) | r16;
        self.sp = self.sp.wrapping_add(1);
        r16
    }

    pub(super) fn pop_r16stk(&mut self, bus: &Bus, code: u8) {
        let r16: u16 = self.pop_r16(bus);
        self.set_r16stk(code, r16);
    }
}
