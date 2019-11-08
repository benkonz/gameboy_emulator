pub struct Cartridge {
    cartridge_type: i32,
    rom_banks: i32,
    ram_banks: i32,
    ram_size: i32,
    has_rtc: bool,
    has_battery: bool,
    rom: Vec<u8>,
    ram: Vec<u8>,
    name: String,
}

impl Cartridge {
    pub fn from_rom(rom: Vec<u8>) -> Cartridge {
        let cartridge_type = i32::from(rom[0x0147]);

        let rom_size = rom[0x0148];
        let rom_banks = match rom_size {
            0x0 => 2,
            0x1 => 4,
            0x2 => 8,
            0x3 => 16,
            0x4 => 32,
            0x5 => 64,
            0x6 => 128,
            0x52 => 72,
            0x53 => 80,
            0x54 => 96,
            _ => panic!("Unknown number of ROM banks"),
        };

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

        let ram = vec![0xFF; 0x2000 * ram_size as usize];

        let mut name = String::new();
        let mut name_index = 0x0134;
        while rom[name_index] != 0x00 && name_index < 0x0143 {
            let c = rom[name_index] as char;
            name.push(c);
            name_index += 1;
        }

        Cartridge {
            cartridge_type,
            rom_banks,
            ram_banks,
            ram_size,
            has_rtc,
            has_battery,
            rom,
            ram,
            name,
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

    pub fn get_cartridge_type(&self) -> i32 {
        self.cartridge_type
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }
}
