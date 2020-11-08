use super::mbc_type::MbcType;
use crate::rtc::Rtc;

pub struct Cartridge {
    rom_banks: usize,
    ram_banks: usize,
    ram_size: usize,
    has_rtc: bool,
    has_battery: bool,
    rom: Vec<u8>,
    ram: Vec<u8>,
    name: String,
    mbc_type: MbcType,
    is_cgb: bool,
    rtc: Rtc,
    last_time: u64,
}

impl Cartridge {
    pub fn from_rom(rom: Vec<u8>) -> Result<Cartridge, String> {
        let cartridge_type = i32::from(rom[0x0147]);
        let mbc_type = match cartridge_type {
            0x00 | 0x08 | 0x09 => Ok(MbcType::RomOnly),
            0x01 | 0x02 | 0x03 | 0xEA | 0xFF => Ok(MbcType::Mbc1),
            0x05 | 0x06 => Ok(MbcType::Mbc2),
            0x0F | 0x10 | 0x11 | 0x12 | 0x13 | 0xFC => Ok(MbcType::Mbc3),
            0x19 | 0x1A | 0x1B | 0x1C | 0x1D | 0x1E => Ok(MbcType::Mbc5),
            _ => Err(format!(
                "Unsupported cartridge type: 0x{:02X}",
                cartridge_type
            )),
        }?;

        let rom_banks = std::cmp::max(Cartridge::pow2ceil(rom.len() / 0x4000), 2);

        let ram_size = usize::from(rom[0x0149]);
        let ram_banks = match ram_size {
            0x0 => Ok(0),
            0x1 => Ok(1),
            0x2 => Ok(1),
            0x3 => Ok(4),
            0x4 => Ok(16),
            _ => Err(format!("Unknown number of RAM banks: 0x{:02X}", ram_size)),
        }?;

        let has_rtc = matches!(cartridge_type, 0x0F | 0x10);
        let has_battery = matches!(
            cartridge_type,
            0x03 | 0x06
                | 0x09
                | 0x0D
                | 0x0F
                | 0x10
                | 0x13
                | 0x17
                | 0x1E
                | 0x1B
                | 0x22
                | 0xFD
                | 0xFF
        );
        let mut name = String::new();
        let mut name_index = 0x0134;
        while rom[name_index] != 0x00 && name_index < 0x0143 {
            let c = rom[name_index] as char;
            name.push(c);
            name_index += 1;
        }

        let is_cgb = rom[0x0143] == 0xC0 || rom[0x0143] == 0x80;

        let ram = match mbc_type {
            MbcType::Mbc2 => vec![0x0F; 0x200],
            MbcType::Mbc5 => vec![0xFF; 0x20000],
            _ => vec![0xFF; 0x8000],
        };

        Ok(Cartridge {
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
            rtc: Rtc::new(),
            last_time: 0,
        })
    }

    fn pow2ceil(i: usize) -> usize {
        let mut i = i - 1;
        i |= i >> 1;
        i |= i >> 2;
        i |= i >> 4;
        i |= i >> 8;
        i += 1;
        i
    }

    pub fn get_rom_banks(&self) -> usize {
        self.rom_banks
    }

    pub fn get_ram_banks(&self) -> usize {
        self.ram_banks
    }

    pub fn get_ram_size(&self) -> usize {
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

    pub fn get_last_timestamp(&self) -> (Rtc, u64) {
        (self.rtc, self.last_time)
    }

    pub fn set_last_timestamp(&mut self, rtc: Rtc, last_time: u64) {
        self.rtc.seconds = rtc.seconds;
        self.rtc.minutes = rtc.minutes;
        self.rtc.hours = rtc.hours;
        self.rtc.days_low = rtc.days_low;
        self.rtc.days_high = rtc.days_high;
        self.last_time = last_time;
    }
}
