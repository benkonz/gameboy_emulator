#[macro_use]
extern crate c_str_macro;
extern crate gameboy_core;
extern crate gl;
extern crate glutin;

mod shader;
mod screen;

use screen::Screen;
use gameboy_core::emulator::Emulator;
use std::thread;
use std::time::{SystemTime, Duration};

const FRAME_RATE: f64 = 60f64;

pub fn start(rom: Vec<u8>) {
    let mut screen = Screen::new();
    let mut emulator = Emulator::new();

    emulator.load_rom(rom);

    let frame_duration = Duration::from_millis((1000f64 * (1f64 / FRAME_RATE)) as u64);

    while screen.is_running {
        let start_time = SystemTime::now();

        let pixels = emulator.update(&mut screen);
        screen.draw(pixels);

        let end_time = SystemTime::now();

        let last_frame_duration = end_time.duration_since(start_time).unwrap();

        if frame_duration >= last_frame_duration {
            let sleep_duration = frame_duration - last_frame_duration;
            thread::sleep(sleep_duration);
        }
    }
}
