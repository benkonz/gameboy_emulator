#[macro_use]
extern crate c_str_macro;
extern crate gameboy_core;
extern crate gl;
extern crate glutin;

mod shader;
mod screen;

use screen::Screen;
use gameboy_core::emulator::Emulator;
use std::time::SystemTime;

const FRAME_RATE: f32 = 60f32;

pub fn start(rom: Vec<u8>) {
    let mut screen = Screen::new();
    let mut emulator = Emulator::new();
    emulator.load_rom(rom);

    let mut last_time = SystemTime::now();

    while screen.is_running {
        let current_time = SystemTime::now();
        let duration = current_time.duration_since(last_time).unwrap()
            .subsec_nanos() as f32 / 10.0_f32.powf(9.0);

        if duration >= 1.0 / FRAME_RATE {
            last_time = current_time;
            let pixels = emulator.update(&mut screen);
            screen.draw(pixels);
        }
    }
}
