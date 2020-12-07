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

use crate::emulator::Emulator;
pub use crate::gpu::cgb_color::CGBColor;
pub use crate::gpu::color::Color;
pub use crate::joypad::Controller;
pub use crate::mmu::cartridge::Cartridge;
pub use crate::rtc::Rtc;
pub struct Gameboy {
    emulator: Emulator,
    controller: Controller,
}
impl Gameboy {
    /// Loads game from rom. Needs a Real Time Clock
    pub fn from_rom(rom: Vec<u8>, rtc: Box<dyn RTC>) -> Result<Gameboy, String> {
        let cartridge = Cartridge::from_rom(rom)?;
        Ok(Gameboy {
            emulator: Emulator::from_cartridge(cartridge, rtc),
            controller: Controller::new(),
        })
    }
    /// Run emulation step
    pub fn emulate(&mut self, system: &mut impl PixelMapper) -> emulator::step_result::StepResult {
        self.emulator.emulate(system, &mut self.controller)
    }
    pub fn get_audio_buffer(&self) -> &[f32] {
        self.emulator.get_audio_buffer()
    }
    pub fn set_cartridge_ram(&mut self, ram: Vec<u8>) {
        self.emulator.get_cartridge_mut().set_ram(ram)
    }
    pub fn has_battery(&self) -> bool {
        self.emulator.get_cartridge().has_battery()
    }
    pub fn has_rtc(&self) -> bool {
        self.emulator.get_cartridge().has_rtc()
    }
    pub fn get_cartridge_ram(&self) -> &[u8] {
        self.emulator.get_cartridge().get_ram()
    }
    pub fn get_cartridge_name(&self) -> &str {
        self.emulator.get_cartridge().get_name()
    }
    pub fn get_last_timestamp(&self) -> (rtc::Rtc, u64) {
        self.emulator.get_cartridge().get_last_timestamp()
    }
    pub fn set_last_timestamp(&mut self, rtc: Rtc, timestamp: u64) {
        self.emulator
            .get_cartridge_mut()
            .set_last_timestamp(rtc, timestamp)
    }
    pub fn set_ram_change_callback(&mut self, f: Box<dyn FnMut(usize, u8)>) {
        self.emulator.set_ram_change_callback(f)
    }
    pub fn press_button(&mut self, button: Button) {
        self.controller.press(button)
    }
    pub fn release_button(&mut self, button: Button) {
        self.controller.release(button)
    }
}
