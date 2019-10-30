use super::mbc::Mbc;
use std::cmp;

pub struct Mbc2 {
    rom_banks: Vec<[u8; 0x4000]>,
    eram: [u8; 0x200],
    num_rom_banks: usize,
    selected_rom_bank: u8,
    external_ram_enabled: bool,
}

impl Mbc for Mbc2 {
    fn read_byte(&self, index: u16) -> u8 {
        match index {
            0x0000..=0x3FFF => self.rom_banks[0][index as usize],
            0x4000..=0x7FFF => {
                self.rom_banks[self.selected_rom_bank as usize][index as usize - 0x4000]
            }
            0xA000..=0xA1FF => {
                if self.external_ram_enabled {
                    self.eram[index as usize - 0xA000] & 0x0F
                } else {
                    0xFF
                }
            }
            0xA200..=0xBFFF => 0x00,
            _ => panic!("index out of range: {:04X}", index),
        }
    }

    fn write_byte(&mut self, index: u16, value: u8) {
        match index {
            0x0000..=0x1FFF => self.external_ram_enabled = (value & 0x0F) == 0x0A,
            0x2000..=0x3FFF => {
                if index & 0x0100 != 0 {
                    self.selected_rom_bank = value & 0x0F;
                    if self.selected_rom_bank == 0 {
                        self.selected_rom_bank = 1;
                    }
                    self.selected_rom_bank &= (self.num_rom_banks - 1) as u8;
                }
            }
            0x4000..=0x7FFF => (),
            0xA000..=0xA1FF => {
                if self.external_ram_enabled {
                    self.eram[index as usize - 0xA000] =
                        value & 0x0F;
                }
            }
            0xA200..=0xBFFF => (),
            _ => panic!("index out of range: {:04X}", index),
        }
    }
}

impl Mbc2 {
    pub fn new(num_rom_banks: usize, rom: &[u8]) -> Mbc2 {
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

        Mbc2 {
            rom_banks,
            eram: [0xFF; 0x200],
            num_rom_banks,
            selected_rom_bank: 1,
            external_ram_enabled: false,
        }
    }
}
