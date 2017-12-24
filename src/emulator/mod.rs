pub mod traits;

use cpu::Cpu;
use gpu::GPU;
use mmu::Memory;
use mmu::interrupt::Interrupt;
use emulator::traits::*;

//TODO: make this the right index
const KEYS_INDEX: u16 = 0xFF80;

pub struct Emulator {
    cpu: Cpu,
    gpu: GPU,
    memory: Memory
}

impl Emulator {
    pub fn new() -> Emulator {
        Emulator {
            cpu: Cpu::new(),
            gpu: GPU::new(),
            memory: Memory::new()
        }
    }

    pub fn cycle<T, V>(&mut self, drawer: &T, controller: &V) where
        T: Drawer, V: Controller {
        let cycles = self.cpu.step(&mut self.memory) as u32;
        self.gpu.step(cycles, &mut self.memory, drawer);
        self.handle_input(controller);
        self.handle_interrupts();
    }

    fn handle_input<T>(&mut self, controller: &T) where T: Controller {
        //TODO: idk about emulator having the responsibility of knowing how to go from keys -> memory
        //TODO: possibly refactor this into the memory struct, like memory.serialize_keys(controller.get_keys())
        let (action_keys, direction_keys) = controller.get_keys();
        if self.use_direction_keys() {
            self.memory[KEYS_INDEX] = direction_keys.bits();
        } else {
            self.memory[KEYS_INDEX] = action_keys.bits();
        }
    }

    fn use_direction_keys(&self) -> bool {
        false
    }

    fn handle_interrupts(&mut self) {
        if let Some(interrupt) = self.memory.get_interrupt() {
            self.process_interrupt(&interrupt);
        }
    }

    fn process_interrupt(&mut self, interrupt: &Interrupt) {
        match *interrupt {
            Interrupt::Vblank => self.cpu.rst_40(&mut self.memory),
            Interrupt::Lcd => self.cpu.rst_48(&mut self.memory),
            Interrupt::Timer => self.cpu.rst_50(&mut self.memory),
            Interrupt::Serial => self.cpu.rst_58(&mut self.memory),
            Interrupt::Joypad => self.cpu.rst_60(&mut self.memory),
        }
    }


}