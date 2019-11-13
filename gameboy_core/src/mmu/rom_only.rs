use super::cartridge::Cartridge;
use super::mbc::Mbc;

pub struct RomOnly {
    cartridge: Cartridge,
    ram_change_callback: Box<dyn FnMut(usize, u8)>,
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
                    let address = index as usize - 0xA000;
                    ram[address] = value;

                    (self.ram_change_callback)(address, value);
                }
            }
            _ => panic!("index out of range: {:04X}", index),
        }
    }

    fn get_cartridge(&self) -> &Cartridge {
        &self.cartridge
    }

    fn set_ram_change_callback(&mut self, f: Box<dyn FnMut(usize, u8)>) {
        self.ram_change_callback = f;
    }
}

impl RomOnly {
    pub fn new(cartridge: Cartridge) -> RomOnly {
        RomOnly {
            cartridge,
            ram_change_callback: Box::new(|_, _| {}),
        }
    }
}
