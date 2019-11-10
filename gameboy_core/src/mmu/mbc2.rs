use super::cartridge::Cartridge;
use super::mbc::Mbc;

pub struct Mbc2 {
    cartridge: Cartridge,
    selected_rom_bank: u8,
    external_ram_enabled: bool,
}

impl Mbc for Mbc2 {
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
            0xA000..=0xA1FF => {
                if self.external_ram_enabled {
                    let ram = self.cartridge.get_ram();
                    ram[index as usize - 0xA000] & 0x0F
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
            0x0000..=0x1FFF => {
                if (index & 0x0100) == 0 {
                    self.external_ram_enabled = (value & 0x0F) == 0x0A;
                }
            }
            0x2000..=0x3FFF => {
                if (index & 0x0100) != 0 {
                    self.selected_rom_bank = value & 0x0F;
                    if self.selected_rom_bank == 0 {
                        self.selected_rom_bank = 1;
                    }
                    self.selected_rom_bank &= (self.cartridge.get_rom_banks() - 1) as u8;
                }
            }
            0x4000..=0x7FFF => (),
            0xA000..=0xA1FF => {
                if self.external_ram_enabled {
                    let address = index as usize - 0xA000;
                    let ram = self.cartridge.get_ram_mut();
                    ram[address] = value & 0x0F;
                }
            }
            0xA200..=0xBFFF => (),
            _ => panic!("index out of range: {:04X}", index),
        }
    }

    fn get_cartridge(&self) -> &Cartridge {
        &self.cartridge
    }
}

impl Mbc2 {
    pub fn new(mut cartridge: Cartridge) -> Mbc2 {
        // MBC2 has a fixed amount of RAM
        cartridge.set_ram(vec![0x0F; 0x200]);

        Mbc2 {
            cartridge,
            selected_rom_bank: 1,
            external_ram_enabled: false,
        }
    }
}
