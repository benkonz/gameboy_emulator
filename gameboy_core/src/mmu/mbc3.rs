use super::mbc::Mbc;
use std::cmp;

pub struct Mbc3 {
    rom_banks: Vec<[u8; 0x4000]>,
    eram_banks: Vec<[u8; 0x2000]>,
    num_rom_banks: usize,
    num_ram_banks: usize,
    selected_rom_bank: u8,
    selected_eram_bank: u8,
    external_ram_enabled: bool,
}

impl Mbc for Mbc3 {
    fn read_byte(&self, index: u16) -> u8 {
        match index {
            0x0000..=0x3FFF => self.rom_banks[0][index as usize],
            0x4000..=0x7FFF => {
                self.rom_banks[self.selected_rom_bank as usize][index as usize - 0x4000]
            }
            0xA000..=0xBFFF => {
                if self.external_ram_enabled {
                    self.eram_banks[self.selected_eram_bank as usize][index as usize - 0xA000]
                } else {
                    0xFF
                }
            }
            _ => panic!("index out of range: {:04X}", index),
        }
    }

    fn write_byte(&mut self, index: u16, value: u8) {
        match index {
            0x0000..=0x1FFF => self.external_ram_enabled = value & 0b0000_1010 == 0b0000_1010,
            0x2000..=0x3FFF => {
                let mut value = value & 0b0111_1111;
                if value == 0 {
                    value = 1;
                }

                self.selected_rom_bank = value;
                self.selected_rom_bank &= (self.num_rom_banks - 1) as u8
            }
            0x4000..=0x5FFF => {
                match value {
                    0x00..=0x07 => {
                        self.selected_eram_bank = value;
                        self.selected_eram_bank &= (self.num_ram_banks - 1) as u8;
                    }
                    0x08..=0x0C => panic!("RTC not implemented!"),
                    _ => panic!("selecting unknown register: {:02X}", value),
                };
            }
            0x6000..=0x7FFF => (), // also used for the RTC
            0xA000..=0xBFFF => {
                if self.external_ram_enabled {
                    self.eram_banks[self.selected_eram_bank as usize][index as usize - 0xA000] =
                        value;
                }
            }
            _ => panic!("index out of range: {:04X}", index),
        }
    }
}

impl Mbc3 {
    pub fn new(num_rom_banks: usize, num_ram_banks: usize, rom: &[u8]) -> Mbc3 {
        let mut rom_banks: Vec<[u8; 0x4000]> = vec![[0; 0x4000]; num_rom_banks];
        for (i, bank) in rom_banks.iter_mut().enumerate() {
            let start = i * 0x4000;
            let end = cmp::min(start + 0x4000, rom.len());
            bank.copy_from_slice(&rom[start..end]);

            // this, along with the call to min above, prevents us from copying past the length of the rom
            if end == rom.len() {
                break;
            }
        }

        let eram_banks = vec![[0xFF; 0x2000]; num_ram_banks];

        Mbc3 {
            rom_banks,
            eram_banks,
            num_rom_banks,
            num_ram_banks,
            selected_rom_bank: 1,
            selected_eram_bank: 0,
            external_ram_enabled: false,
        }
    }
}
