const ADDRESS_SPACE: usize = 1 << 16;
pub struct Bus {
    mem: [u8; ADDRESS_SPACE],
}

impl Default for Bus {
    fn default() -> Self {
        Self::new()
    }
}

impl Bus {
    pub fn new() -> Self {
        Self {
            mem: [0; ADDRESS_SPACE],
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        self.mem[addr as usize]
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        self.mem[addr as usize] = val;
    }
}
