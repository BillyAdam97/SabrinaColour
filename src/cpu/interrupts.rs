use super::Cpu;

impl Cpu {
    pub(super) fn ei(&mut self) {
        self.ime_pending = true;
    }

    pub(super) fn di(&mut self) {
        self.ime = false;
        self.ime_pending = false;
    }
}
