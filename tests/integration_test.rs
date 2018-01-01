extern crate gameboy_core;

use gameboy_core::emulator::Emulator;
use gameboy_core::emulator::traits::Io;
use gameboy_core::joypad::Joypad;

struct MockIo;

impl Io for MockIo {
    fn draw(&self, pixels: &[u8; 144 * 160]) {
    }

    fn update_joypad(&mut self, joypad: &mut Joypad) {
    }
}

#[test]
fn test_bios() {
    let mut emulator = Emulator::new();
    let mut io = MockIo;

    loop {
        emulator.cycle(&mut io);
    }
}