extern crate gameboy_core;

use gameboy_core::emulator::Emulator;
use gameboy_core::emulator::traits::Io;
use gameboy_core::joypad::Joypad;

struct MockIo;

impl Io for MockIo {
    fn draw(&self, pixels: &[u8; 144 * 160]) {
//        for i in 0..144 {
//            for j in 0..160 {
//                print!("{:X}", pixels[i * 144 + j]);
//            }
//            println!();
//        }
    }

    fn update_joypad(&mut self, joypad: &mut Joypad) {
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