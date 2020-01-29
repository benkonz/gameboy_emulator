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
pub mod sound;
mod timer;

pub use crate::button::Button;
pub use crate::controller_event::ControllerEvent;
pub use crate::emulator::step_result::StepResult;
pub use crate::emulator::traits::{PixelMapper, RTC};
pub use crate::emulator::Emulator;
pub use crate::gpu::cgb_color::CGBColor;
pub use crate::gpu::color::Color;
pub use crate::joypad::Controller;
pub use crate::mmu::cartridge::Cartridge;
pub use crate::rtc::Rtc;
