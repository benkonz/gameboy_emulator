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

pub fn start(rom: Vec<u8>) {
    let screen= Screen::new();
    let mut emulator = Emulator::new();

    emulator.load_rom(rom);

    main_loop(screen, emulator);

}

fn main_loop(mut system: Screen, mut emulator: Emulator) {

    emulator.emulate(&mut system);

    window().request_animation_frame(|_| {
        emulator.emulate(&mut system);
        main_loop(system, emulator);
    });
}
