use crate::mmu::interrupt::Interrupt;
use crate::mmu::{self, Memory};

pub struct Timer {}

impl Timer {
    pub fn new() -> Timer {
        Timer {}
    }

    pub fn update(&mut self, cycles: i32, memory: &mut Memory) {
        memory.div_cycles += cycles;

        let div_cycles = 256;

        while memory.div_cycles >= div_cycles {
            memory.div_cycles -= div_cycles;
            let mut div = memory.load(mmu::DIVIDER_INDEX);
            div = div.wrapping_add(1);
            memory.store(mmu::DIVIDER_INDEX, div);
        }

        let tac = memory.load(mmu::TIMER_CONTROL_INDEX);

        if (tac & 0x4) != 0 {
            memory.tima_cycles += cycles;

            let freq = match tac & 0x03 {
                0 => 1024,
                1 => 16,
                2 => 64,
                3 => 256,
                _ => unreachable!(),
            };

            while memory.tima_cycles >= freq {
                memory.tima_cycles -= freq;
                let mut tima = memory.load(mmu::SELECTABLE_TIMER_INDEX);

                tima = tima.wrapping_add(1);

                if tima == 0 {
                    tima = memory.load(mmu::TIMER_RESET_INDEX);
                    memory.request_interrupt(Interrupt::Timer);
                }

                memory.store(mmu::SELECTABLE_TIMER_INDEX, tima);
            }
        }
    }
}
