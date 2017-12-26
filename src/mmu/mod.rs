pub mod interrupt;

use self::interrupt::Interrupt;
use std::fs::File;
use std::io::prelude::Read;
use std::ops::{Index, IndexMut};

static BIOS: &'static [u8] = include_bytes!("bios.gb");

pub struct Memory {
    pub memory: [u8; 0x10000]
}

impl Memory {
    pub fn new() -> Memory {
        Memory { memory: [0; 0x10000] }
    }

    pub fn read_byte(&self, index: u16) -> u8 {
        self.memory[index as usize]
    }

    pub fn write_byte(&mut self, index: u16, value: u8) {
        if index > 0x8000 {
            self.memory[index as usize] = value;
            if index >= 0xC000 && index < 0xDE00 {
                self.memory[(index + 0x2000) as usize] = value;
            }
        }
    }

    pub fn read_word(&self, index: u16) -> u16 {
        let low = self.read_byte(index) as u16;
        let high = self.read_byte(index + 1) as u16;
        (high << 8) + low
    }

    pub fn write_word(&mut self, index: u16, value: u16) {
        let high = (value >> 8) as u8;
        let low = value as u8;
        self.write_byte(index, low);
        self.write_byte(index + 1, high);
    }

    pub fn get_interrupt(&self) -> Option<Interrupt> {
        let interrupt_enable = self.read_byte(0xFFFF);
        let interrupt_flags = self.read_byte(0xFF0F);

        match interrupt_enable & interrupt_flags {
            01 => Some(Interrupt::Vblank),
            02 => Some(Interrupt::Lcd),
            04 => Some(Interrupt::Timer),
            08 => Some(Interrupt::Joypad),
            _ => None
        }
    }

    pub fn request_interrupt(&mut self, interrupt: Interrupt) {
        //TODO: update address 0xFF0F with the new interrupt
    }

    pub fn load_rom(&mut self, rom_file: &mut File) {
        rom_file.read(&mut self.memory);
    }
}

impl Index<u16> for Memory {
    type Output = u8;

    fn index(&self, index: u16) -> &Self::Output {
        &self.memory[index as usize]
    }
}

impl IndexMut<u16> for Memory {
    fn index_mut(&mut self, index: u16) -> &mut Self::Output {
        &mut self.memory[index as usize]
    }
}

#[cfg(test)]
mod tests {
    use mmu::Memory;

    #[test]
    fn test_write_byte() {
        let mut memory = Memory::new();
        memory.write_byte(0xFF80, 1);

        assert_eq!(memory.read_byte(0xFF80), 1);
    }

    #[test]
    fn test_write_word() {
        let mut memory = Memory::new();
        memory.write_word(0xFF80, 0x1122);
        assert_eq!(memory.read_word(0xFF80), 0x1122);
    }

    #[test]
    fn test_read_only() {
        let mut memory = Memory::new();
        memory.write_byte(0, 1);
        memory.write_byte(0x3FFF, 1);
        assert_eq!(memory.read_byte(0), 0);
        assert_eq!(memory.read_byte(0x3FFF), 0);
    }

    #[test]
    fn test_echo() {
        let mut memory = Memory::new();
        memory.write_byte(0xC000, 1);
        assert_eq!(memory.read_byte(0xC000), 1);
        assert_eq!(memory.read_byte(0xE000), 1);
    }
}
