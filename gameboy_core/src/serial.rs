use mmu::interrupt::Interrupt;
use mmu::Memory;

pub struct Serial {
    serial_cycles: i32,
    serial_bit: i32,
}

impl Serial {
    pub fn new() -> Serial {
        Serial {
            serial_cycles: 0,
            serial_bit: 0,
        }
    }

    pub fn update(&mut self, cycles: i32, memory: &mut Memory) {
        let sc = memory.read_byte(0xFF02);

        if sc & 0b1000_0001 == 0b1000_0001 {
            self.serial_cycles += cycles;

            if self.serial_bit < 0 {
                self.serial_bit = 0;
                self.serial_cycles = 0;
                return;
            }

            let serial_cycles = 512;

            if self.serial_cycles >= serial_cycles {
                if self.serial_bit > 7 {
                    memory.write_byte(0xFF02, 0x7F);
                    memory.request_interrupt(Interrupt::Serial);
                    self.serial_bit -= 1;

                    return;
                }

                let mut sb = memory.read_byte(0xFF01);
                sb <<= 1;
                sb |= 0x01;
                memory.write_byte(0xFF01, sb);

                self.serial_cycles -= serial_cycles;
                self.serial_bit += 1;
            }
        }
    }
}
