pub mod traits;

use emulator::traits::PixelMapper;
use cpu::Cpu;
use gpu::GPU;
use joypad::Joypad;
use mmu::interrupt::Interrupt;
use mmu::Memory;
use std::cell::RefCell;
use std::rc::Rc;
use timer::Timer;

pub struct Emulator {
    cpu: Cpu,
    gpu: GPU,
    timer: Timer,
    memory: Memory,
    joypad: Rc<RefCell<Joypad>>,
}

impl Emulator {
    pub fn new(joypad: Rc<RefCell<Joypad>>) -> Emulator {
        Emulator {
            cpu: Cpu::new(),
            gpu: GPU::new(),
            timer: Timer::new(),
            memory: Memory::new(),
            joypad,
        }
    }

    pub fn from_rom(rom: Vec<u8>, joypad: Rc<RefCell<Joypad>>) -> Emulator {
        Emulator {
            cpu: Cpu::new(),
            gpu: GPU::new(),
            timer: Timer::new(),
            memory: Memory::from_rom(rom),
            joypad,
        }
    }

    pub fn emulate<T: PixelMapper>(&mut self, system: &mut T) -> bool {
        let cycles = self.cpu.step(&mut self.memory);
        self.timer.update(cycles, &mut self.memory);
        let vblank = self.gpu.step(cycles, &mut self.memory, system);
        self.joypad.borrow_mut().update(&mut self.memory);
        self.handle_interrupts();
        vblank
    }

    fn handle_interrupts(&mut self) {
        if self.cpu.interrupt_enabled {
            if let Some(interrupt) = self.memory.get_interrupts() {
                self.process_interrupt(interrupt);
            }
        }
    }

    fn process_interrupt(&mut self, interrupt: Interrupt) {
        println!("processing interrupt: {:?}", interrupt);
        self.cpu.interrupt_enabled = false;

        match interrupt {
            Interrupt::Vblank => self.cpu.rst_40(&mut self.memory),
            Interrupt::Lcd => self.cpu.rst_48(&mut self.memory),
            Interrupt::Timer => self.cpu.rst_50(&mut self.memory),
            Interrupt::Serial => self.cpu.rst_58(&mut self.memory),
            Interrupt::Joypad => self.cpu.rst_60(&mut self.memory),
        }

        self.memory.remove_interrupt(interrupt);
        self.cpu.halted = false;
    }
}
