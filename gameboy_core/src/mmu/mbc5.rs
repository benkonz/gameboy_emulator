use super::mbc::Mbc;
use std::cmp;

pub struct Mbc5 {
    rom_banks: Vec<[u8; 0x4000]>,
    eram_banks: Vec<[u8; 0x2000]>,
    num_rom_banks: usize,
    num_ram_banks: usize,
    selected_rom_bank: usize,
    selected_rom_bank_high: usize,
    selected_eram_bank: usize,
    external_ram_enabled: bool,
}

impl Mbc for Mbc5 {
    fn read_byte(&self, index: u16) -> u8 {
        match index {
            0x0000..=0x3FFF => self.rom_banks[0][index as usize],
            0x4000..=0x7FFF => self.rom_banks[self.selected_rom_bank][index as usize - 0x4000],
            0xA000..=0xBFFF => {
                if self.external_ram_enabled {
                    self.eram_banks[self.selected_eram_bank][index as usize - 0xA000]
                } else {
                    0xFF
                }
            }
            _ => panic!("index out of range: {:04X}", index),
        }
    }

    fn write_byte(&mut self, index: u16, value: u8) {
        match index {
            0x0000..=0x1FFF => self.external_ram_enabled = (value & 0x0F) == 0x0A,
            0x2000..=0x2FFF => {
                self.selected_rom_bank = (value as usize) | (self.selected_rom_bank_high << 8);
                self.selected_rom_bank &= self.num_rom_banks - 1
            }
            0x3000..=0x3FFF => {
                self.selected_rom_bank_high = value as usize & 0x01;
                self.selected_rom_bank =
                    (self.selected_rom_bank & 0xFF) | (self.selected_rom_bank_high << 8);
                self.selected_rom_bank &= self.num_rom_banks - 1
            }
            0x4000..=0x5FFF => {
                self.selected_eram_bank = value as usize & 0x0F;
                self.selected_eram_bank &= self.num_ram_banks & 0x01;
            }
            0x6000..=0x7FFF => (),
            0xA000..=0xBFFF => {
                if self.external_ram_enabled {
                    self.eram_banks[self.selected_eram_bank][index as usize - 0xA000] = value;
                }
            }
            _ => panic!("index out of range: {:04X}", index),
        }
    }
}

impl Mbc5 {
    pub fn new(num_rom_banks: usize, num_ram_banks: usize, rom: &[u8]) -> Mbc5 {
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
        Mbc5 {
            rom_banks,
            eram_banks,
            num_rom_banks,
            num_ram_banks,
            selected_rom_bank: 1,
            selected_rom_bank_high: 0,
            selected_eram_bank: 0,
            external_ram_enabled: false,
        }
    }
}
