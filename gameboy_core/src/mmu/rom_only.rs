use super::mbc::Mbc;
use cartridge::Cartridge;

pub struct RomOnly {
    cartridge: Cartridge,
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
                    let ram = self.cartridge.get_ram();
                    ram[index as usize - 0xA000]
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
                    let ram = self.cartridge.get_ram_mut();
                    ram[index as usize - 0xA000] = value
                }
            }
            _ => panic!("index out of range: {:04X}", index),
        }
    }

    fn get_cartridge(&self) -> &Cartridge {
        &self.cartridge
    }
}

impl RomOnly {
    pub fn new(cartridge: Cartridge) -> RomOnly {
        RomOnly { cartridge }
    }
}
