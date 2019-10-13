#[macro_use]
extern crate stdweb;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate stdweb_derive;
extern crate gameboy_core;

mod screen;
mod webgl_rendering_context;

use gameboy_core::Emulator;
use screen::Screen;
use stdweb::web::window;
use screen::CONTROLLER;

pub fn start(rom: Vec<u8>) {
    let screen = Screen::new();
    let emulator = Emulator::from_rom(rom);

    main_loop(screen, emulator);
}

fn main_loop(mut system: Screen, mut emulator: Emulator) {
    loop {
        let vblank = unsafe { emulator.emulate(&mut system, &mut CONTROLLER) };
        if vblank {
            break;
        }
    }

    system.render();
    window().request_animation_frame(|_| {
        main_loop(system, emulator);
    });
}
