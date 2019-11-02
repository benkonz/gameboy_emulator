use super::cartridge::Cartridge;
use super::mbc::Mbc;

pub struct Mbc3 {
    cartridge: Cartridge,
    eram_banks: Vec<[u8; 0x2000]>,
    selected_rom_bank: u8,
    selected_eram_bank: u8,
    external_ram_enabled: bool,
}

impl Mbc for Mbc3 {
    fn read_byte(&self, index: u16) -> u8 {
        match index {
            0x0000..=0x3FFF => {
                let rom = self.cartridge.get_rom();
                rom[index as usize]
            }
            0x4000..=0x7FFF => {
                let rom = self.cartridge.get_rom();
                let offset = self.selected_rom_bank as usize * 0x4000;
                rom[index as usize - 0x4000 + offset]
            }
            0xA000..=0xBFFF => {
                if self.external_ram_enabled && self.cartridge.get_ram_size() > 0 {
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
            0x0000..=0x1FFF => self.external_ram_enabled = (value & 0x0F) == 0x0A,
            0x2000..=0x3FFF => {
                let mut value = value & 0b0111_1111;
                if value == 0 {
                    value = 1;
                }

                self.selected_rom_bank = value;
                self.selected_rom_bank &= (self.cartridge.get_rom_banks() - 1) as u8
            }
            0x4000..=0x5FFF => {
                match value {
                    0x00..=0x07 => {
                        self.selected_eram_bank = value;
                        self.selected_eram_bank &= (self.cartridge.get_ram_banks() - 1) as u8;
                    }
                    0x08..=0x0C => {
                        if self.cartridge.has_rtc() {
                            panic!("RTC not implemented!")
                        }
                    }
                    _ => panic!("selecting unknown register: {:02X}", value),
                };
            }
            0x6000..=0x7FFF => (), // also used for the RTC
            0xA000..=0xBFFF => {
                if self.external_ram_enabled && self.cartridge.get_ram_size() > 0 {
                    self.eram_banks[self.selected_eram_bank as usize][index as usize - 0xA000] =
                        value;
                }
            }
            _ => panic!("index out of range: {:04X}", index),
        }
    }
}

impl Mbc3 {
    pub fn new(cartridge: Cartridge) -> Mbc3 {
        let eram_banks = vec![[0xFF; 0x2000]; cartridge.get_ram_banks() as usize];

        Mbc3 {
            cartridge,
            eram_banks,
            selected_rom_bank: 1,
            selected_eram_bank: 0,
            external_ram_enabled: false,
        }
    }
}
