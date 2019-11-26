pub mod traits;

use cpu::Cpu;
use emulator::traits::PixelMapper;
use gpu::GPU;
use joypad::Controller;
use mmu::cartridge::Cartridge;
use mmu::interrupt::Interrupt;
use mmu::Memory;
use serial::Serial;
use timer::Timer;

pub struct Emulator {
    cpu: Cpu,
    gpu: GPU,
    timer: Timer,
    serial: Serial,
    memory: Memory,
}

impl Emulator {
    pub fn from_cartridge(cartridge: Cartridge) -> Emulator {
        let is_cgb = cartridge.is_cgb();
        Emulator {
            cpu: Cpu::new(is_cgb),
            gpu: GPU::new(is_cgb),
            timer: Timer::new(),
            serial: Serial::new(),
            memory: Memory::from_cartridge(cartridge, is_cgb),
        }
    }

    pub fn emulate<T: PixelMapper>(&mut self, system: &mut T, controller: &mut Controller) -> bool {
        let cycles = self.cpu.step(&mut self.memory);
        self.timer.update(cycles, &mut self.memory);
        self.serial.update(cycles, &mut self.memory);
        let vblank = self.gpu.step(cycles, &mut self.memory, system);
        controller.update(&mut self.memory);
        self.handle_interrupts();
        vblank
    }

    fn handle_interrupts(&mut self) {
        if self.cpu.are_interrupts_enabled() {
            if let Some(interrupt) = self.memory.get_interrupts() {
                self.process_interrupt(interrupt);
            }
        }
    }

    fn process_interrupt(&mut self, interrupt: Interrupt) {
        self.cpu.disable_interrupts();

        match interrupt {
            Interrupt::Vblank => self.cpu.rst_40(&mut self.memory),
            Interrupt::Lcd => self.cpu.rst_48(&mut self.memory),
            Interrupt::Timer => self.cpu.rst_50(&mut self.memory),
            Interrupt::Serial => self.cpu.rst_58(&mut self.memory),
            Interrupt::Joypad => self.cpu.rst_60(&mut self.memory),
        }
        self.memory.remove_interrupt(interrupt);
        self.cpu.unhalt();
    }

    pub fn get_cartridge(&self) -> &Cartridge {
        &self.memory.get_cartridge()
    }

    pub fn set_ram_change_callback(&mut self, f: Box<dyn FnMut(usize, u8)>) {
        self.memory.set_ram_change_callback(f);
    }
}
