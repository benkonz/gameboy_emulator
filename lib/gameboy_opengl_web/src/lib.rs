#[macro_use]
extern crate stdweb;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate stdweb_derive;
extern crate gameboy_core;

mod screen;
mod system;
mod webgl_rendering_context;

use gameboy_core::Emulator;
use system::System;
use stdweb::web;
use stdweb::web::Date;

pub fn start(rom: Vec<u8>) {
    let mut system = System::new();
    let mut emulator = Emulator::new();

    emulator.load_rom(rom);

    main_loop(system, emulator);

}

fn main_loop(mut system: System, mut emulator: Emulator) {

    let frame_duration = 1f64 / 60f64 * 1000f64;

    let start = Date::now();

    emulator.emulate(&mut system);

    let end = Date::now();

    let duration = end - start;

    if duration > 0f64 {
        let wait_time = duration as u32 - frame_duration as u32;
        web::set_timeout(|| main_loop(system, emulator),
                         wait_time);
    } else {
        main_loop(system, emulator);
    }
}
