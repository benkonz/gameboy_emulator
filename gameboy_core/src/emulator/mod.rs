pub mod step_result;
pub mod traits;

use self::step_result::StepResult;
use self::traits::{PixelMapper, RTC};
use crate::cpu::Cpu;
use crate::gpu::GPU;
use crate::joypad::Controller;
use crate::mmu::cartridge::Cartridge;
use crate::mmu::interrupt::Interrupt;
use crate::mmu::Memory;
use crate::timer::Timer;
use crate::transfer::ByteTransfer;

pub struct Emulator {
    cpu: Cpu,
    gpu: GPU,
    timer: Timer,
    memory: Memory,
}

impl Emulator {
    pub fn from_cartridge(cartridge: Cartridge, rtc: Box<dyn RTC>) -> Emulator {
        let is_cgb = cartridge.is_cgb();
        Emulator {
            cpu: Cpu::new(is_cgb),
            gpu: GPU::new(is_cgb),
            timer: Timer::new(),
            memory: Memory::from_cartridge(cartridge, rtc, is_cgb),
        }
    }

    pub fn emulate(
        &mut self,
        system: &mut impl PixelMapper,
        controller: &mut Controller,
        link_cable: &mut dyn ByteTransfer,
    ) -> StepResult {
        let cycles = self.cpu.step(&mut self.memory);
        self.timer.update(cycles, &mut self.memory);
        let audio_buffer_full = self.memory.get_sound_mut().step(cycles);
        let vblank = self.gpu.step(cycles, &mut self.memory, system);
        controller.update(&mut self.memory);
        link_cable.update(cycles, &mut self.memory);
        self.handle_interrupts();

        if audio_buffer_full {
            StepResult::AudioBufferFull
        } else if vblank {
            StepResult::VBlank
        } else {
            StepResult::Nothing
        }
    }

    fn handle_interrupts(&mut self) {
        if let Some(interrupt) = self.memory.get_interrupts() {
            self.process_interrupt(interrupt);
        }
    }
    fn process_interrupt(&mut self, interrupt: Interrupt) {
        if self.cpu.are_interrupts_enabled() {
            self.cpu.disable_interrupts();
            match interrupt {
                Interrupt::Vblank => self.cpu.rst_40(&mut self.memory),
                Interrupt::Lcd => self.cpu.rst_48(&mut self.memory),
                Interrupt::Timer => self.cpu.rst_50(&mut self.memory),
                Interrupt::Serial => self.cpu.rst_58(&mut self.memory),
                Interrupt::Joypad => self.cpu.rst_60(&mut self.memory),
            }
            self.memory.remove_interrupt(interrupt);
        }
        self.cpu.unhalt();
    }

    pub fn get_cartridge(&self) -> &Cartridge {
        &self.memory.get_cartridge()
    }

    pub fn set_ram_change_callback(&mut self, f: Box<dyn FnMut(usize, u8)>) {
        self.memory.set_ram_change_callback(f);
    }

    pub fn get_cartridge_mut(&mut self) -> &mut Cartridge {
        self.memory.get_cartridge_mut()
    }

    pub fn get_audio_buffer(&self) -> &[f32] {
        self.memory.get_sound().get_audio_buffer()
    }
}
