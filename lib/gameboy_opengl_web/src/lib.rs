#[macro_use]
extern crate stdweb;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate stdweb_derive;
extern crate gameboy_core;

mod screen;
mod webgl_rendering_context;

use screen::Screen;
use gameboy_core::Emulator;

pub fn start(rom: Vec<u8>) {
    let mut screen = Screen::new();
    let mut emulator = Emulator::new();

    emulator.load_rom(rom);

    emulator.emulate(&mut screen);
}