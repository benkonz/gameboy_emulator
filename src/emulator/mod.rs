mod traits;

use cpu::Cpu;
use gpu::GPU;
use mmu::Memory;
use mmu::interrupt::Interrupt;
use joypad::Joypad;


pub struct Emulator {
    cpu: Cpu,
    gpu: GPU,
    memory: Memory,
    controller: Joypad
}

impl Emulator {
    pub fn new() -> Emulator {
        Emulator {
            cpu: Cpu::new(),
            gpu: GPU::new(),
            memory: Memory::new(),
            controller: Joypad::new()
        }
    }

    pub fn power_on(&mut self, drawer: &Drawer, controller: &mut Controller) {
        loop {
            let cycles = self.cpu.step(&mut self.memory);
            self.gpu.step(cycles, &mut self.memory, &drawer);
            self.handle_input(&mut controller);
            self.handle_interrupts();
        }
    }

    fn handle_input(&mut self, controller: &Controller) {
        controller.update_controller(&mut self.controller);
    }

    fn handle_interrupts(&mut self) {
        if let Some(interrupt) = self.memory.get_interrupt() {
            self.process_interrupt(&interrupt);
        }
    }

    fn process_interrupt(&mut self, interrupt: &Interrupt) {
        match interrupt {
            Interrupt::Vblank => self.cpu.rst_40(&mut self.memory),
            Interrupt::Lcd => self.cpu.rst_48(&mut self.memory),
            Interrupt::Timer => self.cpu.rst_50(&mut self.memory),
            Interrupt::Serial => self.cpu.rst_58(&mut self.memory),
            Interrupt::Joypad => self.cpu.rst_60(&mut self.memory),
        }
    }
}