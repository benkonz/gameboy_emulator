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
    let mut screen= Screen::new();
    let joypad = screen.get_input();
    let emulator = Emulator::from_rom(rom, joypad);

    main_loop(screen, emulator);

}

fn main_loop(mut system: Screen, mut emulator: Emulator) {
    window().request_animation_frame(|_| {
        loop {
            let vblank = emulator.emulate(&mut system);
            if vblank {
                break;
            }
        }

        system.render();
        main_loop(system, emulator);
    });
}
