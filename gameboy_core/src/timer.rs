use mmu::Memory;
use mmu::interrupt::Interrupt;

const TIMER_INDEX: u16 = 0xFF05;
const TIMER_RESET_INDEX: u16 = 0xFF06;
const TIMER_CONTROL_INDEX: u16 = 0xFF07;

pub struct Timer {
    divide_register_counter: i32,
    timer_counter: i32,
    timer_max_cycles: i32
}

impl Timer {
    pub fn new() -> Timer {
        Timer {
            divide_register_counter: 0,
            timer_counter: 0,
            timer_max_cycles: 1024,
        }
    }

    pub fn update(&mut self, cycles: i32, memory: &mut Memory) {
        self.divider_register(cycles, memory);

        if self.is_clock_enabled(memory) {
            self.update_clock_frequency(memory);

            self.timer_counter += cycles;

            if self.timer_counter <= 0 {
                let mut timer = memory.read_byte(TIMER_INDEX);

                if timer == 0xFF {
                    timer = memory.read_byte(TIMER_RESET_INDEX);
                    memory.request_interrupt(Interrupt::Timer);
                } else {
                    timer += 1;
                }

                memory.write_byte(TIMER_INDEX, timer);
            }
        }
    }

    fn divider_register(&mut self, cycles: i32, memory: &mut Memory) {
        self.divide_register_counter += cycles;
        if self.divide_register_counter >= 255 {
            self.divide_register_counter = 0;
            memory.divider_register = memory.divider_register.wrapping_add(1);
        }
    }

    fn is_clock_enabled(&self, memory: &Memory) -> bool {
        memory.read_byte(TIMER_CONTROL_INDEX) & 2 != 0
    }

    fn update_clock_frequency(&mut self, memory: &Memory) {
        let timer_controller = memory.read_byte(TIMER_CONTROL_INDEX);

        self.timer_max_cycles = match timer_controller & 3 {
            0 => 1024,
            1 => 16,
            2 => 64,
            3 => 256,
            _ => 0
        };
    }
}
