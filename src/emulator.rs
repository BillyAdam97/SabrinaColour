use crate::bus::Bus;
use crate::cpu::Cpu;
use std::io;

pub struct Emulator {
    cpu: Cpu,
    bus: Bus,
}

impl Emulator {
    pub fn new() -> Self {
        Self {
            cpu: Cpu::new(),
            bus: Bus::new(),
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        let mut buffer = String::new();
        loop {
            buffer.clear();
            io::stdin().read_line(&mut buffer)?;
            if buffer.as_str().trim_end() == "q" {
                break;
            }
            self.cpu.step(&mut self.bus);
        }
        Ok(())
    }
}
