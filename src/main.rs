use std::process::abort;

use sabrina_colour::emulator::Emulator;

fn main() {
    let mut emu: Emulator = Emulator::new();

    if emu.run().is_err() {
        abort()
    }
}
