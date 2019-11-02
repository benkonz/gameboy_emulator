use super::cartridge::Cartridge;
use super::mbc::Mbc;

pub struct RomOnly {
    cartridge: Cartridge,
    eram_banks: [u8; 0x2000],
}

impl Mbc for RomOnly {
    fn read_byte(&self, index: u16) -> u8 {
        match index {
            0x0000..=0x7FFF => {
                let rom = self.cartridge.get_rom();
                rom[index as usize]
            }
            0xA000..=0xBFFF => {
                if self.cartridge.get_ram_size() > 0 {
                    self.eram_banks[index as usize - 0xA000]
                } else {
                    0xFF
                }
            }
            _ => panic!("index out of range: {:04X}", index),
        }
    }
    fn write_byte(&mut self, index: u16, value: u8) {
        match index {
            0x0000..=0x7FFF => (),
            0xA000..=0xBFFF => {
                if self.cartridge.get_ram_size() > 0 {
                    self.eram_banks[index as usize - 0xA000] = value
                }
            }
            _ => panic!("index out of range: {:04X}", index),
        }
    }
}

impl RomOnly {
    pub fn new(cartridge: Cartridge) -> RomOnly {
        RomOnly {
            cartridge,
            eram_banks: [0xFF; 0x2000],
        }
    }
}
