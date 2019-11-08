#[macro_use]
extern crate bitflags;

pub mod button;
pub mod cartridge;
mod cpu;
pub mod emulator;
mod gpu;
mod joypad;
mod mmu;
mod serial;
mod timer;

pub use button::Button;
pub use cartridge::Cartridge;
pub use emulator::traits::PixelMapper;
pub use emulator::Emulator;
pub use gpu::color::Color;
pub use joypad::Controller;
