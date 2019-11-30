use super::cartridge::Cartridge;
use super::mbc::Mbc;
use bit_utils;
use emulator::traits::RTC;
use rtc::Rtc;

pub struct Mbc3 {
    cartridge: Cartridge,
    selected_rom_bank: u8,
    selected_eram_bank: u8,
    external_ram_enabled: bool,
    ram_change_callback: Box<dyn FnMut(usize, u8)>,
    rtc: Box<dyn RTC>,
    rtc_last_time: u64,
    rtc_last_time_cache: u64,
    rtc_register_select: u8,
    use_rtc_for_ram: bool,
    rtc_latch_data: u8,
    rtc_latch: Rtc,
    rtc_data: Rtc,
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
                if self.use_rtc_for_ram && self.external_ram_enabled && self.cartridge.has_rtc() {
                    match self.rtc_register_select {
                        0x08 => self.rtc_latch.seconds,
                        0x09 => self.rtc_latch.minutes,
                        0x0A => self.rtc_latch.hours,
                        0x0B => self.rtc_latch.days_low,
                        0x0C => self.rtc_latch.days_high,
                        _ => 0xFF,
                    }
                } else if self.external_ram_enabled && self.cartridge.get_ram_size() > 0 {
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
                self.selected_rom_bank = value & 0x7F;
                if self.selected_rom_bank == 0 {
                    self.selected_rom_bank = 1;
                }
                self.selected_rom_bank &= (self.cartridge.get_rom_banks() - 1) as u8
            }
            0x4000..=0x5FFF => {
                match value {
                    0x00..=0x03 => {
                        self.selected_eram_bank = value;
                        self.selected_eram_bank &= (self.cartridge.get_ram_banks() - 1) as u8;
                        self.use_rtc_for_ram = false;
                    }
                    0x08..=0x0C => {
                        if self.cartridge.has_rtc() && self.external_ram_enabled {
                            self.rtc_register_select = value;
                            self.use_rtc_for_ram = true;
                        }
                    }
                    _ => (),
                };
            }
            0x6000..=0x7FFF => {
                if self.cartridge.has_rtc() {
                    if self.rtc_latch_data == 0 && value == 1 {
                        self.update_rtc_latch();
                        self.rtc_latch.seconds = self.rtc_data.seconds;
                        self.rtc_latch.minutes = self.rtc_data.minutes;
                        self.rtc_latch.hours = self.rtc_data.hours;
                        self.rtc_latch.days_low = self.rtc_data.days_low;
                        self.rtc_latch.days_high = self.rtc_data.days_high;
                    }
                    self.rtc_latch_data = value;
                }
            }
            0xA000..=0xBFFF => {
                if self.use_rtc_for_ram && self.external_ram_enabled && self.cartridge.has_rtc() {
                    match self.rtc_register_select {
                        0x08 => self.rtc_data.seconds = value,
                        0x09 => self.rtc_data.minutes = value,
                        0x0A => self.rtc_data.hours = value,
                        0x0B => self.rtc_data.days_low = value,
                        0x0C => {
                            self.rtc_data.days_high =
                                (self.rtc_data.days_high & 0x80) | (value & 0xC1)
                        }
                        _ => (),
                    }
                } else if self.external_ram_enabled && self.cartridge.get_ram_size() > 0 {
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

    fn get_cartridge_mut(&mut self) -> &mut Cartridge {
        &mut self.cartridge
    }

    fn set_ram_change_callback(&mut self, f: Box<dyn FnMut(usize, u8)>) {
        self.ram_change_callback = f;
    }
}

impl Mbc3 {
    pub fn new(cartridge: Cartridge, rtc: Box<dyn RTC>) -> Mbc3 {
        let (rtc_data, mut rtc_last_time) = cartridge.get_last_timestamp();

        if rtc_last_time == 0 {
            rtc_last_time = rtc.get_current_time();
        }

        let mut mbc3 = Mbc3 {
            cartridge,
            selected_rom_bank: 1,
            selected_eram_bank: 0,
            external_ram_enabled: false,
            ram_change_callback: Box::new(|_, _| {}),
            rtc,
            rtc_last_time,
            rtc_last_time_cache: rtc_last_time,
            use_rtc_for_ram: false,
            rtc_register_select: 0,
            rtc_latch_data: 0,
            rtc_data,
            rtc_latch: rtc_data,
        };
        mbc3.update_rtc_latch();

        mbc3
    }

    fn update_rtc_latch(&mut self) {
        let current_time_secs = self.rtc.get_current_time();
        if !bit_utils::is_set(self.rtc_data.days_high, 6)
            && self.rtc_last_time_cache != current_time_secs
        {
            self.rtc_last_time_cache = current_time_secs;
            let mut difference = current_time_secs - self.rtc_last_time;
            self.rtc_last_time = current_time_secs;
            if difference > 0 {
                self.rtc_data.seconds += (difference % 60) as u8;
                if self.rtc_data.seconds > 59 {
                    self.rtc_data.seconds -= 60;
                    self.rtc_data.minutes += 1;
                }
                difference /= 60;
                self.rtc_data.minutes += (difference % 60) as u8;
                if self.rtc_data.minutes > 59 {
                    self.rtc_data.minutes -= 60;
                    self.rtc_data.hours += 1;
                }
                difference /= 60;
                self.rtc_data.hours += (difference % 24) as u8;
                let mut rtc_days = 0;
                if self.rtc_data.hours > 23 {
                    self.rtc_data.hours -= 24;
                    rtc_days += 1;
                }
                difference /= 24;
                rtc_days +=
                    self.rtc_data.days_low as u16 + ((self.rtc_data.days_high as u16) & 0x01);
                rtc_days += difference as u16;
                if rtc_days > 511 {
                    rtc_days %= 512;
                    // set the carry flag and clear the rest of the bits
                    self.rtc_data.days_high |= 0x80;
                    self.rtc_data.days_high &= 0xC0;
                }
                self.rtc_data.days_low = (rtc_days & 0xFF) as u8;
                self.rtc_data.days_high |= ((rtc_days & 0x100) >> 8) as u8;
            }
        }

        self.cartridge
            .set_last_timestamp(self.rtc_data, self.rtc_last_time);
    }
}
