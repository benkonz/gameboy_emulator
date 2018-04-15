pub mod traits;

use self::traits::*;
use cpu::Cpu;
use gpu::GPU;
use mmu::interrupt::Interrupt;
use mmu::Memory;
use std::thread;
use std::time::{Duration, SystemTime};
use timer::Timer;

pub struct Emulator {
    cpu: Cpu,
    gpu: GPU,
    timer: Timer,
    memory: Memory,
}

impl Emulator {
    pub fn new() -> Emulator {
        Emulator {
            cpu: Cpu::new(),
            gpu: GPU::new(),
            timer: Timer::new(),
            memory: Memory::new(),
        }
    }

    pub fn load_rom(&mut self, rom: Vec<u8>) {
        self.memory.load_rom(rom);
    }

    pub fn emulate<T: Render + Input + Running + PixelMapper>(&mut self, system: &mut T) {
        let frame_rate = 60f64;
        let frame_duration = Duration::from_millis((1000f64 * (1f64 / frame_rate)) as u64);
        let max_cycles = 69905;

        while system.should_run() {
            let start_time = SystemTime::now();

            let mut cycles_this_update = 0;

            while cycles_this_update < max_cycles {
                let cycles = self.cpu.step(&mut self.memory);
                cycles_this_update += cycles;
                self.timer.update(cycles, &mut self.memory);
                self.gpu.step(cycles, &mut self.memory, system);
                system.get_input().update(&mut self.memory);
                self.handle_interrupts();
            }

            system.render();

            let end_time = SystemTime::now();

            let last_frame_duration = end_time.duration_since(start_time).unwrap();

            if frame_duration >= last_frame_duration {
                let sleep_duration = frame_duration - last_frame_duration;
                thread::sleep(sleep_duration);
            }
        }
    }

    fn handle_interrupts(&mut self) {
        if self.cpu.interrupt_enabled {
            if let Some(interrupt) = self.memory.get_interrupts() {
                self.process_interrupt(interrupt);
            }
        }
    }

    fn process_interrupt(&mut self, interrupt: Interrupt) {
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
