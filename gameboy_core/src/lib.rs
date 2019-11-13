#[macro_use]
extern crate bitflags;

pub mod button;
pub mod controller_event;
mod cpu;
pub mod emulator;
mod gpu;
mod joypad;
mod mmu;
mod serial;
mod timer;

pub use button::Button;
pub use controller_event::ControllerEvent;
pub use emulator::traits::PixelMapper;
pub use emulator::Emulator;
pub use gpu::color::Color;
pub use joypad::Controller;
pub use mmu::cartridge::Cartridge;
