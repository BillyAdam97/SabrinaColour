use std::process::abort;

use SabrinaColour::emulator::Emulator;

fn main() {
    let mut emu: Emulator = Emulator::new();

    if let Err(_) = emu.run() {
        abort()
    }
}
