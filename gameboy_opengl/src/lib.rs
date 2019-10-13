#[macro_use]
extern crate c_str_macro;
extern crate gameboy_core;
extern crate glutin;

mod screen;
mod shader;
mod opengl_rendering_context;

pub fn start(rom: Vec<u8>) {
    screen::start(rom);
}
