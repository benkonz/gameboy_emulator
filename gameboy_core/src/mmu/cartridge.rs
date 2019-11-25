use super::mbc_type::MbcType;

pub struct Cartridge {
    rom_banks: i32,
    ram_banks: i32,
    ram_size: i32,
    has_rtc: bool,
    has_battery: bool,
    rom: Vec<u8>,
    ram: Vec<u8>,
    name: String,
    mbc_type: MbcType,
    is_cgb: bool,
}

fn pow2ceil(i: i32) -> i32 {
    let mut i = i - 1;
    i |= i >> 1;
    i |= i >> 2;
    i |= i >> 4;
    i |= i >> 8;
    i += 1;
    i
}

impl Cartridge {
    pub fn from_rom(rom: Vec<u8>) -> Cartridge {
        let cartridge_type = i32::from(rom[0x0147]);

        let rom_banks = std::cmp::max(pow2ceil(rom.len() as i32 / 0x4000), 2);

        let ram_size = i32::from(rom[0x0149]);
        let ram_banks = match ram_size {
            0x0 => 0,
            0x1 => 1,
            0x2 => 1,
            0x3 => 4,
            0x4 => 16,
            _ => panic!("Unknown number of RAM banks"),
        };

        let has_rtc = match cartridge_type {
            0x0F | 0x10 => true,
            _ => false,
        };
        let has_battery = match cartridge_type {
            0x03 | 0x06 | 0x09 | 0x0D | 0x0F | 0x10 | 0x13 | 0x17 | 0x1E | 0x1B | 0x22 | 0xFD
            | 0xFF => true,
            _ => false,
        };

        let mut name = String::new();
        let mut name_index = 0x0134;
        while rom[name_index] != 0x00 && name_index < 0x0143 {
            let c = rom[name_index] as char;
            name.push(c);
            name_index += 1;
        }

        let mbc_type = match cartridge_type {
            0x00 | 0x08 | 0x09 => MbcType::RomOnly,
            0x01 | 0x02 | 0x03 | 0xEA | 0xFF => MbcType::Mbc1,
            0x05 | 0x06 => MbcType::Mbc2,
            0x0F | 0x10 | 0x11 | 0x12 | 0x13 | 0xFC => MbcType::Mbc3,
            0x19 | 0x1A | 0x1B | 0x1C | 0x1D | 0x1E => MbcType::Mbc5,
            _ => panic!("Unsupported cartridge type: {:?}", cartridge_type),
        };

        let is_cgb = rom[0x0143] == 0xC0 || rom[0x0143] == 0x80;

        let ram = match mbc_type {
            MbcType::Mbc2 => vec![0x0F; 0x200],
            MbcType::Mbc5 => vec![0xFF; 0x20000],
            _ => vec![0xFF; 0x8000]
        };

        Cartridge {
            rom_banks,
            ram_banks,
            ram_size,
            has_rtc,
            has_battery,
            rom,
            ram,
            name,
            mbc_type,
            is_cgb,
        }
    }

    pub fn get_rom_banks(&self) -> i32 {
        self.rom_banks
    }

    pub fn get_ram_banks(&self) -> i32 {
        self.ram_banks
    }

    pub fn get_ram_size(&self) -> i32 {
        self.ram_size
    }

    pub fn has_rtc(&self) -> bool {
        self.has_rtc
    }

    pub fn has_battery(&self) -> bool {
        self.has_battery
    }

    pub fn get_rom(&self) -> &[u8] {
        self.rom.as_ref()
    }

    pub fn get_ram_mut(&mut self) -> &mut [u8] {
        self.ram.as_mut()
    }

    pub fn get_ram(&self) -> &[u8] {
        self.ram.as_ref()
    }

    pub fn set_ram(&mut self, ram: Vec<u8>) {
        self.ram = ram;
    }

    pub fn get_mbc_type(&self) -> MbcType {
        self.mbc_type
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn is_cgb(&self) -> bool {
        self.is_cgb
    }
}
