use mmu::interrupt::Interrupt;
use mmu::Memory;

// TODO: move these to the mmu as a pub const
const SELECTABLE_TIMER_INDEX: u16 = 0xFF05;
const TIMER_RESET_INDEX: u16 = 0xFF06;
const TIMER_CONTROL_INDEX: u16 = 0xFF07;

// this might make sense as a static module, there isn't any state to manage
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
            let mut div = memory.get_div_from_memory();
            div = div.wrapping_add(1);
            memory.set_div_from_memory(div);
        }

        let tac = memory.read_byte(TIMER_CONTROL_INDEX);

        if tac & 0x04 != 0 {
            memory.tima_cycles += cycles;

            let freq = match tac & 0x03 {
                0b00 => 1024,
                0b01 => 16,
                0b10 => 64,
                0b11 => 256,
                _ => panic!("impossible"),
            };

            while memory.tima_cycles >= freq {
                memory.tima_cycles -= freq;
                let mut tima = memory.read_byte(SELECTABLE_TIMER_INDEX);

                if tima == 0xFF {
                    tima = memory.read_byte(TIMER_RESET_INDEX);
                    memory.request_interrupt(Interrupt::Timer);
                } else {
                    tima += 1;
                }

                memory.write_byte(SELECTABLE_TIMER_INDEX, tima);
            }
        }
    }
}
