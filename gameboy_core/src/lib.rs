#[macro_use]
extern crate bitflags;

mod bit_utils;
pub mod button;
pub mod controller_event;
mod cpu;
pub mod emulator;
mod gpu;
mod joypad;
mod mmu;
pub mod rtc;
mod timer;

pub use button::Button;
pub use controller_event::ControllerEvent;
pub use emulator::traits::{PixelMapper, RTC};
pub use emulator::Emulator;
pub use gpu::cgb_color::CGBColor;
pub use gpu::color::Color;
pub use joypad::Controller;
pub use mmu::cartridge::Cartridge;
pub use rtc::Rtc;
