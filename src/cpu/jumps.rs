use super::Cpu;

impl Cpu {
    pub(super) fn jp_u16(&mut self, addr: u16) {
        self.pc = addr;
    }

    pub(super) fn jr_i8(&mut self, addr: i8) {
        self.pc = self.pc.wrapping_add_signed(addr as i16);
    }
}
