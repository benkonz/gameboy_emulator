#![feature(nll)]
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate lazy_static;

mod cpu;
pub mod emulator;
mod gpu;
pub mod joypad;
mod mmu;
mod timer;

pub use emulator::Emulator;
pub use gpu::color::Color;
