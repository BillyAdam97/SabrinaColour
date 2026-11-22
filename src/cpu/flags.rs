use super::Cpu;
use crate::cpu::registers::Flags;

impl Cpu {
    pub(super) fn rlca(&mut self) -> u8 {
        let c: u8 = (self.a >> 7) & 0x01;
        self.a = self.a.rotate_left(1);

        self.f.set(Flags::ZERO, false);
        self.f.set(Flags::SUB, false);
        self.f.set(Flags::HALF, false);
        self.f.set(Flags::CARRY, c == 1);
        4
    }

    pub(super) fn rrca(&mut self) {
        let c: u8 = self.a & 0x1;
        self.a = self.a.rotate_right(1);

        self.f.set(Flags::ZERO, false);
        self.f.set(Flags::SUB, false);
        self.f.set(Flags::HALF, false);
        self.f.set(Flags::CARRY, c == 1);
    }

    pub(super) fn rla(&mut self) {
        let c: u8 = (self.a >> 7) & 0x01;
        self.a <<= 1;
        self.a |= self.f.contains(Flags::CARRY) as u8;

        self.f.set(Flags::ZERO, false);
        self.f.set(Flags::SUB, false);
        self.f.set(Flags::HALF, false);
        self.f.set(Flags::CARRY, c == 1);
    }

    pub(super) fn rra(&mut self) {
        let c: u8 = self.a & 0x01;
        self.a >>= 1;
        self.a |= (self.f.contains(Flags::CARRY) as u8) << 7;

        self.f.set(Flags::ZERO, false);
        self.f.set(Flags::SUB, false);
        self.f.set(Flags::HALF, false);
        self.f.set(Flags::CARRY, c == 1);
    }

    pub(super) fn daa(&mut self) {
        let mut adj: u8 = 0;
        if self.f.contains(Flags::SUB) {
            if self.f.contains(Flags::HALF) {
                adj += 0x6;
            }
            if self.f.contains(Flags::CARRY) {
                adj += 0x60;
            }
            self.a = self.a.wrapping_sub(adj);
        } else {
            if self.f.contains(Flags::HALF) || (self.a & 0xF) > 0x9 {
                adj += 0x6;
            }
            if self.f.contains(Flags::CARRY) || self.a > 0x99 {
                adj += 0x60;
                self.f.set(Flags::CARRY, true);
            }
            self.a = self.a.wrapping_add(adj);
        }
        self.f.set(Flags::ZERO, self.a == 0);
        self.f.set(Flags::HALF, false);
    }

    pub(super) fn cpl(&mut self) {
        self.a = !self.a;
        self.f.set(Flags::SUB, true);
        self.f.set(Flags::HALF, true);
    }

    pub(super) fn scf(&mut self) {
        self.f.set(Flags::SUB, false);
        self.f.set(Flags::HALF, false);
        self.f.set(Flags::CARRY, true);
    }

    pub(super) fn ccf(&mut self) {
        self.f.set(Flags::CARRY, !self.f.contains(Flags::CARRY));
        self.f.set(Flags::SUB, false);
        self.f.set(Flags::HALF, false);
    }
}
