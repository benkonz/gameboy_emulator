#[macro_use]
extern crate c_str_macro;
extern crate gameboy_core;
extern crate gl;
extern crate glutin;

mod shader;
mod screen;

use screen::Screen;
use gameboy_core::emulator::Emulator;

pub fn start(rom: Vec<u8>) {
    let mut screen = Screen::new();
    let mut emulator = Emulator::new();

    emulator.load_rom(rom);

    emulator.emulate(&mut screen);
}
