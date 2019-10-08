use mmu::interrupt::Interrupt;
use mmu::Memory;

const SELECTABLE_TIMER_INDEX: u16 = 0xFF05;
const TIMER_RESET_INDEX: u16 = 0xFF06;
const TIMER_CONTROL_INDEX: u16 = 0xFF07;

struct Clock {
    main: i32,
    sub: i32,
    div: i32,
}

impl Clock {
    fn new() -> Clock {
        Clock {
            main: 0,
            sub: 0,
            div: 0,
        }
    }
}

struct Registers {
    div: u8,
    tima: u8,
}

impl Registers {
    fn new() -> Registers {
        Registers { div: 0, tima: 0 }
    }
}

pub struct Timer {
    clock: Clock,
    registers: Registers,
}

impl Timer {
    pub fn new() -> Timer {
        Timer {
            clock: Clock::new(),
            registers: Registers::new(),
        }
    }

    pub fn update(&mut self, cycles: i32, memory: &mut Memory) {
        self.clock.sub += cycles;

        if self.clock.sub >= 4 {
            self.clock.main += 1;
            self.clock.sub -= 4;

            self.clock.div += 1;

            if self.clock.div == 16 {
                self.registers.div = self.registers.div.wrapping_add(1);
                self.clock.div = 0;
                memory.divider_register = self.registers.div;
            }
        }

        self.check(memory);
    }

    fn check(&mut self, memory: &mut Memory) {
        let tac = memory.read_byte(TIMER_CONTROL_INDEX);

        let threshold = match tac & 0b11 {
            0b00 => 64,
            0b01 => 1,
            0b10 => 4,
            0b11 => 16,
            _ => panic!(),
        };

        if self.clock.main >= threshold {
            self.step(memory);
        }
    }

    fn step(&mut self, memory: &mut Memory) {
        self.clock.main = 0;
        let (result, overflow) = self.registers.tima.overflowing_add(1);

        if overflow {
            let tma = memory.read_byte(TIMER_RESET_INDEX);
            self.registers.tima = tma;
            memory.request_interrupt(Interrupt::Timer);
        } else {
            self.registers.tima = result;
        }

        memory.write_byte(SELECTABLE_TIMER_INDEX, self.registers.tima);
    }
}
