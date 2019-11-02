use super::cartridge::Cartridge;
use super::mbc::Mbc;

pub struct Mbc1 {
    cartridge: Cartridge,
    eram_banks: Vec<[u8; 0x2000]>,
    selected_rom_bank: u8,
    selected_eram_bank: u8,
    in_ram_banking_mode: bool,
    external_ram_enabled: bool,
    higher_rom_bank_bits: u8,
}

impl Mbc for Mbc1 {
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
                    let selected_bank = if self.in_ram_banking_mode {
                        self.selected_eram_bank as usize
                    } else {
                        0
                    };
                    self.eram_banks[selected_bank][index as usize - 0xA000]
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
                if self.in_ram_banking_mode {
                    self.selected_rom_bank = value & 0x1F;
                } else {
                    self.selected_rom_bank = (value & 0x1F) | (self.higher_rom_bank_bits << 5);
                }

                if self.selected_rom_bank == 0x00
                    || self.selected_rom_bank == 0x20
                    || self.selected_rom_bank == 0x40
                    || self.selected_rom_bank == 0x60
                {
                    self.selected_rom_bank += 1;
                }

                self.selected_rom_bank &= (self.cartridge.get_rom_banks() - 1) as u8;
            }
            0x4000..=0x5FFF => {
                if self.in_ram_banking_mode {
                    self.selected_eram_bank = value & 0x03;
                    self.selected_eram_bank &= (self.cartridge.get_ram_banks() - 1) as u8;
                } else {
                    self.higher_rom_bank_bits = value & 0x03;
                    self.selected_rom_bank = (value & 0x1F) | (self.higher_rom_bank_bits << 5);

                    if self.selected_rom_bank == 0x00
                        || self.selected_rom_bank == 0x20
                        || self.selected_rom_bank == 0x40
                        || self.selected_rom_bank == 0x60
                    {
                        self.selected_rom_bank += 1;
                    }
                    self.selected_rom_bank &= (self.cartridge.get_ram_banks() - 1) as u8;
                }
            }
            0x6000..=0x7FFF => self.in_ram_banking_mode = value & 0x01 == 0x01,
            0xA000..=0xBFFF => {
                if self.external_ram_enabled && self.cartridge.get_ram_size() > 0 {
                    if self.in_ram_banking_mode {
                        self.eram_banks[self.selected_eram_bank as usize]
                            [index as usize - 0xA000] = value;
                    } else {
                        self.eram_banks[0][index as usize - 0xA000] = value;
                    }
                }
            }
            _ => panic!("index out of range: {:04X}", index),
        }
    }
}

impl Mbc1 {
    pub fn new(cartridge: Cartridge) -> Mbc1 {
        let eram_banks = vec![[0xFF; 0x2000]; cartridge.get_ram_size() as usize];

        Mbc1 {
            cartridge,
            eram_banks,
            selected_rom_bank: 1,
            selected_eram_bank: 0,
            in_ram_banking_mode: false,
            external_ram_enabled: false,
            higher_rom_bank_bits: 0,
        }
    }
}