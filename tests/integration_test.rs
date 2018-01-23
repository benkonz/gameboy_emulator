extern crate gameboy_core;

use gameboy_core::emulator::Emulator;
use gameboy_core::emulator::traits::Io;
use gameboy_core::joypad::Joypad;

struct MockIo;

impl Io for MockIo {
    fn draw(&self, _pixels: &[u8; 144 * 160]) {
    }

    fn update_joypad(&mut self, _joypad: &mut Joypad) {
    }
}

#[test]
fn test_bios() {
    let mut emulator = Emulator::new();
    let mut io = MockIo;

    let rom = include_bytes!("tetris.gb").to_vec();

    emulator.load_rom(rom);

    loop {
        emulator.cycle(&mut io);
    }
}