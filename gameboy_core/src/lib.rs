#[macro_use]
extern crate bitflags;

mod cpu;
pub mod emulator;
mod gpu;
mod joypad;
mod mmu;
mod timer;
mod serial;
pub mod button;

pub use emulator::Emulator;
pub use gpu::color::Color;
pub use joypad::Controller;
pub use button::Button;
pub use emulator::traits::PixelMapper;
