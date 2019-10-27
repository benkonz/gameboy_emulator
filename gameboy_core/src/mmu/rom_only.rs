use super::mbc::Mbc;
use std::cmp;

pub struct RomOnly {
    rom_banks: [u8; 0x8000],
    eram_banks: [u8; 0x2000],
}

impl Mbc for RomOnly {
    fn read_byte(&self, index: u16) -> u8 {
        match index {
            0x0000..=0x7FFF => self.rom_banks[index as usize],
            0xA000..=0xBFFF => self.eram_banks[index as usize - 0xA000],
            _ => panic!("index out of range: {:04X}", index),
        }
    }
    fn write_byte(&mut self, index: u16, value: u8) {
        match index {
            0x0000..=0x7FFF => self.rom_banks[index as usize] = value,
            0xA000..=0xBFFF => self.eram_banks[index as usize - 0xA000] = value,
            _ => panic!("index out of range: {:04X}", index),
        }
    }
}

impl RomOnly {
    pub fn new(rom: &[u8]) -> RomOnly {
        let mut rom_banks = [0; 0x8000];
        let end = cmp::min(0x8000, rom.len());
        rom_banks.copy_from_slice(&rom[0..end]);

        RomOnly {
            rom_banks,
            eram_banks: [0xFF; 0x2000],
        }
    }
}
