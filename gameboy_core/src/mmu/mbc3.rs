use super::cartridge::Cartridge;
use super::mbc::Mbc;

pub struct Mbc3 {
    cartridge: Cartridge,
    selected_rom_bank: u8,
    selected_eram_bank: u8,
    external_ram_enabled: bool,
    ram_change_callback: Box<dyn FnMut(usize, u8)>,
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
                    let ram = self.cartridge.get_ram();
                    let offset = self.selected_eram_bank as usize * 0x2000;
                    ram[index as usize - 0xA000 + offset]
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
                    0x00..=0x03 => {
                        self.selected_eram_bank = value;
                        self.selected_eram_bank &= (self.cartridge.get_ram_banks() - 1) as u8;
                    }
                    0x08..=0x0C => {
                        if self.cartridge.has_rtc() {
                            // TODO: ADD RTC FUNCTIONALITY
                        }
                    }
                    _ => (),
                };
            }
            0x6000..=0x7FFF => (), // also used for the RTC
            0xA000..=0xBFFF => {
                if self.external_ram_enabled && self.cartridge.get_ram_size() > 0 {
                    let ram = self.cartridge.get_ram_mut();
                    let offset = self.selected_eram_bank as usize * 0x2000;
                    let address = index as usize - 0xA000 + offset;
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

impl Mbc3 {
    pub fn new(cartridge: Cartridge) -> Mbc3 {
        Mbc3 {
            cartridge,
            selected_rom_bank: 1,
            selected_eram_bank: 0,
            external_ram_enabled: false,
            ram_change_callback: Box::new(|_, _| {}),
        }
    }
}
