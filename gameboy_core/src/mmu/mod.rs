pub mod cartridge;
pub mod gpu_cycles;
pub mod interrupt;
mod mbc;
mod mbc1;
mod mbc2;
mod mbc3;
mod mbc5;
mod mbc_type;
mod rom_only;

use self::cartridge::Cartridge;
use self::gpu_cycles::GpuCycles;
use self::interrupt::Interrupt;
use self::mbc::Mbc;
use self::mbc1::Mbc1;
use self::mbc2::Mbc2;
use self::mbc3::Mbc3;
use self::mbc5::Mbc5;
use self::mbc_type::MbcType;
use self::rom_only::RomOnly;
use bit_utils;
use emulator::traits::RTC;
use gpu::cgb_color::CGBColor;
use gpu::lcd_control_flag::LcdControlFlag;

const INTERRUPT_ENABLE_INDEX: u16 = 0xFFFF;
const INTERRUPT_FLAGS_INDEX: u16 = 0xFF0F;

const INITIAL_VALUES_FOR_FFXX: [u8; 0x100] = [
    0xCF, 0x00, 0x7E, 0xFF, 0xD3, 0x00, 0x00, 0xF8, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xE1,
    0x80, 0xBF, 0xF3, 0xFF, 0xBF, 0xFF, 0x3F, 0x00, 0xFF, 0xBF, 0x7F, 0xFF, 0x9F, 0xFF, 0xBF, 0xFF,
    0xFF, 0x00, 0x00, 0xBF, 0x77, 0xF3, 0xF1, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0x71, 0x72, 0xD5, 0x91, 0x58, 0xBB, 0x2A, 0xFA, 0xCF, 0x3C, 0x54, 0x75, 0x48, 0xCF, 0x8F, 0xD9,
    0x91, 0x80, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFC, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0x2B, 0x0B, 0x64, 0x2F, 0xAF, 0x15, 0x60, 0x6D, 0x61, 0x4E, 0xAC, 0x45, 0x0F, 0xDA, 0x92, 0xF3,
    0x83, 0x38, 0xE4, 0x4E, 0xA7, 0x6C, 0x38, 0x58, 0xBE, 0xEA, 0xE5, 0x81, 0xB4, 0xCB, 0xBF, 0x7B,
    0x59, 0xAD, 0x50, 0x13, 0x5E, 0xF6, 0xB3, 0xC1, 0xDC, 0xDF, 0x9E, 0x68, 0xD7, 0x59, 0x26, 0xF3,
    0x62, 0x54, 0xF8, 0x36, 0xB7, 0x78, 0x6A, 0x22, 0xA7, 0xDD, 0x88, 0x15, 0xCA, 0x96, 0x39, 0xD3,
    0xE6, 0x55, 0x6E, 0xEA, 0x90, 0x76, 0xB8, 0xFF, 0x50, 0xCD, 0xB5, 0x1B, 0x1F, 0xA5, 0x4D, 0x2E,
    0xB4, 0x09, 0x47, 0x8A, 0xC4, 0x5A, 0x8C, 0x4E, 0xE7, 0x29, 0x50, 0x88, 0xA8, 0x66, 0x85, 0x4B,
    0xAA, 0x38, 0xE7, 0x6B, 0x45, 0x3E, 0x30, 0x37, 0xBA, 0xC5, 0x31, 0xF2, 0x71, 0xB4, 0xCF, 0x29,
    0xBC, 0x7F, 0x7E, 0xD0, 0xC7, 0xC3, 0xBD, 0xCF, 0x59, 0xEA, 0x39, 0x01, 0x2E, 0x00, 0x69, 0x00,
];

const INITIAL_VALUES_FOR_COLOR_FFXX: [u8; 0x100] = [
    0xCF, 0x00, 0x7C, 0xFF, 0x44, 0x00, 0x00, 0xF8, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xE1,
    0x80, 0xBF, 0xF3, 0xFF, 0xBF, 0xFF, 0x3F, 0x00, 0xFF, 0xBF, 0x7F, 0xFF, 0x9F, 0xFF, 0xBF, 0xFF,
    0xFF, 0x00, 0x00, 0xBF, 0x77, 0xF3, 0xF1, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF,
    0x91, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFC, 0x00, 0x00, 0x00, 0x00, 0xFF, 0x7E, 0xFF, 0xFE,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x3E, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xC0, 0xFF, 0xC1, 0x00, 0xFE, 0xFF, 0xFF, 0xFF,
    0xF8, 0xFF, 0x00, 0x00, 0x00, 0x8F, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B, 0x03, 0x73, 0x00, 0x83, 0x00, 0x0C, 0x00, 0x0D,
    0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E, 0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD, 0xD9, 0x99,
    0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC, 0xDD, 0xDC, 0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E,
    0x45, 0xEC, 0x42, 0xFA, 0x08, 0xB7, 0x07, 0x5D, 0x01, 0xF5, 0xC0, 0xFF, 0x08, 0xFC, 0x00, 0xE5,
    0x0B, 0xF8, 0xC2, 0xCA, 0xF4, 0xF9, 0x0D, 0x7F, 0x44, 0x6D, 0x19, 0xFE, 0x46, 0x97, 0x33, 0x5E,
    0x08, 0xFF, 0xD1, 0xFF, 0xC6, 0x8B, 0x24, 0x74, 0x12, 0xFC, 0x00, 0x9F, 0x94, 0xB7, 0x06, 0xD5,
    0x40, 0x7A, 0x20, 0x9E, 0x04, 0x5F, 0x41, 0x2F, 0x3D, 0x77, 0x36, 0x75, 0x81, 0x8A, 0x70, 0x3A,
    0x98, 0xD1, 0x71, 0x02, 0x4D, 0x01, 0xC1, 0xFF, 0x0D, 0x00, 0xD3, 0x05, 0xF9, 0x00, 0x0B, 0x00,
];

pub struct Memory {
    mbc: Box<dyn Mbc>,
    wram: Vec<u8>,
    vram: Vec<u8>,
    oam: [u8; 0x100],
    high_ram: [u8; 0x100],
    joypad_state: u8,
    pub scan_line: u8,
    pub irq48_signal: u8,
    pub screen_disabled: bool,
    pub lcd_status_mode: u8,
    pub gpu_cycles: GpuCycles,
    pub div_cycles: i32,
    pub tima_cycles: i32,
    is_cgb: bool,
    vram_bank: i32,
    wram_bank: i32,
    hdma_source: u16,
    hdma_destination: u16,
    hdma_bytes: i32,
    hdma_enabled: bool,
    pub cgb_background_palettes: [[CGBColor; 4]; 8],
    pub cgb_sprite_palettes: [[CGBColor; 4]; 8],
}

impl Memory {
    pub fn from_cartridge(cartridge: Cartridge, rtc: Box<dyn RTC>, is_cgb: bool) -> Memory {
        // set the initial values for the IO memory into high-ram
        // this is necessary, since we don't load the bios
        let high_ram = if is_cgb {
            INITIAL_VALUES_FOR_COLOR_FFXX
        } else {
            INITIAL_VALUES_FOR_FFXX
        };

        let mbc: Box<dyn Mbc> = match cartridge.get_mbc_type() {
            MbcType::RomOnly => Box::new(RomOnly::new(cartridge)),
            MbcType::Mbc1 => Box::new(Mbc1::new(cartridge)),
            MbcType::Mbc2 => Box::new(Mbc2::new(cartridge)),
            MbcType::Mbc3 => Box::new(Mbc3::new(cartridge, rtc)),
            MbcType::Mbc5 => Box::new(Mbc5::new(cartridge)),
        };

        let vram = if is_cgb {
            vec![0x00; 0x2000 * 2]
        } else {
            vec![0x00; 0x2000]
        };

        let wram = if is_cgb {
            vec![0x00; 0x1000 * 8]
        } else {
            vec![0x00; 0x1000 * 2]
        };

        let mut hdma_source = 0;
        let mut hdma_destination = 0;
        if is_cgb {
            let mut hdma_source_high = high_ram[0xFF51 - 0xFF00] as u16;
            let hdma_source_low = high_ram[0xFF52 - 0xFF00] as u16;
            if hdma_source_high > 0x7F && hdma_source_high < 0xA0 {
                hdma_source_high = 0;
            }
            hdma_source = (hdma_source_high << 8) | (hdma_source_low & 0xF0);
            let hdma_destination_high = high_ram[0xFF53 - 0xFF00] as u16;
            let hdma_destination_low = high_ram[0xFF54 - 0xFF00] as u16;
            hdma_destination =
                ((hdma_destination_high & 0x1F) << 8) | (hdma_destination_low & 0xF0);
            hdma_destination |= 0x8000;
        }

        let white = CGBColor {
            red: 0,
            green: 0,
            blue: 0,
        };

        Memory {
            mbc,
            vram,
            wram,
            oam: [0; 0x100],
            high_ram,
            scan_line: 144,
            joypad_state: 0,
            irq48_signal: 0,
            screen_disabled: false,
            lcd_status_mode: 1,
            gpu_cycles: GpuCycles::new(),
            div_cycles: 0,
            tima_cycles: 0,
            is_cgb,
            vram_bank: 0,
            wram_bank: 1,
            hdma_source,
            hdma_destination,
            hdma_bytes: 0,
            hdma_enabled: false,
            cgb_background_palettes: [[white; 4]; 8],
            cgb_sprite_palettes: [[white; 4]; 8],
        }
    }

    pub fn read_byte(&self, index: u16) -> u8 {
        match index {
            0x0000..=0x7FFF => self.mbc.read_byte(index),
            0x8000..=0x9FFF => {
                let offset = if self.is_cgb {
                    self.vram_bank as usize * 0x2000
                } else {
                    0
                };
                let address = index as usize - 0x8000 + offset;
                self.vram[address]
            }
            0xA000..=0xBFFF => self.mbc.read_byte(index),
            0xC000..=0xCFFF => self.wram[index as usize - 0xC000],
            0xD000..=0xDFFF => {
                let offset = if self.is_cgb {
                    self.wram_bank as usize * 0x1000
                } else {
                    0x1000
                };
                let address = index as usize - 0xD000 + offset;
                self.wram[address]
            }
            0xE000..=0xFDFF => self.read_byte(index - 0x2000),
            0xFE00..=0xFE9F => self.oam[index as usize - 0xFE00],
            0xFF00..=0xFFFF => match index {
                0xFF00 => self.get_joypad_state(),
                0xFF07 => self.high_ram[index as usize - 0xFF00] | 0xF8,
                0xFF0F => self.high_ram[index as usize - 0xFF00] | 0xE0,
                0xFF41 => self.high_ram[index as usize - 0xFF00] | 0x80,
                0xFF44 => {
                    if !self.screen_disabled {
                        self.scan_line
                    } else {
                        0x00
                    }
                }
                0xFF68 | 0xFF6A => {
                    if self.is_cgb {
                        self.high_ram[index as usize - 0xFF00] | 0x40
                    } else {
                        0xC0
                    }
                }
                0xFF69 | 0xFF6B => {
                    if self.is_cgb {
                        self.high_ram[index as usize - 0xFF00] | 0xF8
                    } else {
                        0xFF
                    }
                }
                _ => self.high_ram[index as usize - 0xFF00],
            },
            _ => 0,
        }
    }

    fn get_joypad_state(&self) -> u8 {
        let joypad_control = self.high_ram[0];

        if self.are_direction_keys_enabled() {
            (joypad_control & 0xF0) | (self.joypad_state & 0x0F)
        } else if self.are_action_keys_enabled() {
            (joypad_control & 0xF0) | (self.joypad_state >> 4)
        } else {
            joypad_control
        }
    }

    pub fn set_joypad_state(&mut self, joypad_state: u8) {
        self.joypad_state = joypad_state;
    }

    pub fn are_action_keys_enabled(&self) -> bool {
        let joypad_control = self.high_ram[0];
        joypad_control & 0x30 != 0x20
    }

    pub fn are_direction_keys_enabled(&self) -> bool {
        let joypad_control = self.high_ram[0];
        joypad_control & 0x30 != 0x10
    }

    pub fn write_byte(&mut self, index: u16, value: u8) {
        match index {
            0x0000..=0x7FFF => self.mbc.write_byte(index, value),
            0x8000..=0x9FFF => {
                let offset = if self.is_cgb {
                    self.vram_bank as usize * 0x2000
                } else {
                    0
                };
                let address = index as usize - 0x8000 + offset;
                self.vram[address] = value
            }
            0xA000..=0xBFFF => self.mbc.write_byte(index, value),
            0xC000..=0xCFFF => self.wram[index as usize - 0xC000] = value,
            0xD000..=0xDFFF => {
                let offset = if self.is_cgb {
                    self.wram_bank as usize * 0x1000
                } else {
                    0x1000
                };
                let address = index as usize - 0xD000 + offset;
                self.wram[address] = value;
            }
            0xE000..=0xFDFF => self.write_byte(index - 0x2000, value),
            0xFE00..=0xFE9F => self.oam[index as usize - 0xFE00] = value,
            0xFF00..=0xFFFF => match index {
                0xFF04 => self.reset_div_cycles(),
                0xFF07 => {
                    let value = value & 0x07;
                    let current_tac = self.read_byte(0xFF07);
                    if (current_tac & 0x03) != (value & 0x03) {
                        self.reset_tima_cycles();
                    }
                    self.high_ram[index as usize - 0xFF00] = value;
                }
                0xFF0F => self.high_ram[index as usize - 0xFF00] = value & 0x1F,
                0xFF40 => self.do_lcd_control_write(value),
                0xFF41 => self.do_lcd_status_write(value),
                0xFF44 => self.do_scanline_write(value),
                0xFF45 => self.do_lyc_write(value),
                0xFF46 => {
                    self.high_ram[index as usize - 0xFF00] = value;
                    self.do_dma_transfer(value)
                }
                0xFF4D if self.is_cgb => {
                    let current_key1 = self.get_key1();
                    self.high_ram[index as usize - 0xFF00] =
                        (current_key1 & 0x80) | (value & 1) | 0x7E;
                }
                0xFF4F if self.is_cgb => {
                    let value = value & 1;
                    self.vram_bank = value as i32;
                    self.high_ram[index as usize - 0xFF00] = value;
                }
                0xFF51 if self.is_cgb => {
                    let value = if value > 0x7F && value < 0xC0 {
                        0
                    } else {
                        value
                    };
                    self.hdma_source = ((value as u16) << 8) | (self.hdma_source & 0xF0);
                    self.high_ram[index as usize - 0xFF00] = value;
                }
                0xFF52 if self.is_cgb => {
                    let value = value & 0xF0;
                    self.hdma_source = (self.hdma_source & 0xFF00) | (value as u16);
                    self.high_ram[index as usize - 0xFF00] = value;
                }
                0xFF53 if self.is_cgb => {
                    let value = value & 0x1F;
                    self.hdma_destination = ((value as u16) << 8) | (self.hdma_destination & 0xF0);
                    self.hdma_destination |= 0x8000;
                    self.high_ram[index as usize - 0xFF00] = value;
                }
                0xFF54 if self.is_cgb => {
                    let value = value & 0xF0;
                    self.hdma_destination = (self.hdma_destination & 0x1F00) | (value as u16);
                    self.hdma_destination |= 0x8000;
                    self.high_ram[index as usize - 0xFF00] = value;
                }
                0xFF55 if self.is_cgb => self.do_cgb_dma(value),
                0xFF68 if self.is_cgb => {
                    self.high_ram[index as usize - 0xFF00] = value;
                    self.update_color_palette(true, value);
                }
                0xFF69 if self.is_cgb => {
                    self.high_ram[index as usize - 0xFF00] = value;
                    self.set_color_palette(true, value);
                }
                0xFF6A if self.is_cgb => {
                    self.high_ram[index as usize - 0xFF00] = value;
                    self.update_color_palette(false, value);
                }
                0xFF6B if self.is_cgb => {
                    self.high_ram[index as usize - 0xFF00] = value;
                    self.set_color_palette(false, value);
                }
                0xFF70 if self.is_cgb => {
                    let value = value & 0x07;
                    self.wram_bank = value as i32;
                    if self.wram_bank == 0 {
                        self.wram_bank = 1;
                    }
                    self.high_ram[index as usize - 0xFF00] = value;
                }
                0xFFFF => self.high_ram[index as usize - 0xFF00] = value & 0x1F,
                _ => self.high_ram[index as usize - 0xFF00] = value,
            },
            _ => {}
        };
    }

    pub fn do_lcd_control_write(&mut self, value: u8) {
        let current_lcdc = LcdControlFlag::from_bits_truncate(self.get_lcdc_from_memory());
        let new_lcdc = LcdControlFlag::from_bits_truncate(value);
        self.set_lcdc_from_memory(value);

        if !current_lcdc.contains(LcdControlFlag::WINDOW)
            && new_lcdc.contains(LcdControlFlag::WINDOW)
        {
            self.reset_window_line();
        }

        if new_lcdc.contains(LcdControlFlag::DISPLAY) {
            self.enable_screen();
        } else {
            self.disable_screen();
        }
    }

    pub fn do_lcd_status_write(&mut self, value: u8) {
        let current_stat = self.get_lcdc_from_memory() & 0x07;
        let new_stat = (value & 0x78) | (current_stat & 0x07);
        self.set_lcd_status_from_memory(new_stat);
        let lcd_control = LcdControlFlag::from_bits_truncate(self.get_lcdc_from_memory());
        let mut signal = self.irq48_signal;
        let mode = self.lcd_status_mode;
        signal &= (new_stat >> 3) & 0x0F;
        self.irq48_signal = signal;

        if lcd_control.contains(LcdControlFlag::DISPLAY) {
            if bit_utils::is_set(new_stat, 3) && mode == 0 {
                if signal == 0 {
                    self.request_interrupt(Interrupt::Lcd);
                }
                signal |= 0b01;
            }

            if bit_utils::is_set(new_stat, 4) && mode == 1 {
                if signal == 0 {
                    self.request_interrupt(Interrupt::Lcd);
                }
                signal |= 0b10;
            }

            if bit_utils::is_set(new_stat, 5) && mode == 2 && signal == 0 {
                self.request_interrupt(Interrupt::Lcd);
            }
            self.compare_ly_to_lyc();
        }
    }

    pub fn do_scanline_write(&mut self, value: u8) {
        let current_ly = self.scan_line;
        if bit_utils::is_set(current_ly, 7) && !bit_utils::is_set(value, 7) {
            self.disable_screen();
        }
    }

    pub fn do_lyc_write(&mut self, value: u8) {
        let current_lyc = self.get_lyc_from_memory();
        if current_lyc != value {
            self.set_lyc_from_memory(value);
            let lcd_control = LcdControlFlag::from_bits_truncate(self.get_lcdc_from_memory());
            if lcd_control.contains(LcdControlFlag::DISPLAY) {
                self.compare_ly_to_lyc();
            }
        }
    }

    pub fn do_dma_transfer(&mut self, data: u8) {
        let address = 0x100 * u16::from(data);
        if address >= 0x8000 && address < 0xE000 {
            for i in 0..0xA0 {
                let value = self.read_byte(address + i);
                self.write_byte(0xFE00 + i, value);
            }
        }
    }

    pub fn compare_ly_to_lyc(&mut self) {
        if !self.screen_disabled {
            let lyc = self.get_lyc_from_memory();
            let mut stat = self.get_lcd_status_from_memory();

            if lyc == self.scan_line {
                stat |= 0b0000_0100;
                if bit_utils::is_set(stat, 6) {
                    if self.irq48_signal == 0 {
                        self.request_interrupt(Interrupt::Lcd);
                    }
                    self.irq48_signal |= 0b0000_1000;
                }
            } else {
                stat &= 0b1111_1011;
                self.irq48_signal &= 0b1111_0111;
            }
            self.set_lcd_status_from_memory(stat);
        }
    }

    pub fn enable_screen(&mut self) {
        if self.screen_disabled {
            self.gpu_cycles.screen_enable_delay_cycles = 244;
        }
    }

    pub fn disable_screen(&mut self) {
        self.screen_disabled = true;
        let mut stat = self.get_lcd_status_from_memory();
        stat &= 0x7C;
        self.set_lcd_status_from_memory(stat);
        self.lcd_status_mode = 0;
        self.gpu_cycles.cycles_counter = 0;
        self.gpu_cycles.aux_cycles_counter = 0;
        self.scan_line = 0;
        self.irq48_signal = 0;
    }

    pub fn reset_window_line(&mut self) {
        let wy = self.get_window_line_from_memory();

        if (self.gpu_cycles.window_line == 0) && (self.scan_line < 144) && (self.scan_line > wy) {
            self.gpu_cycles.window_line = 144;
        }
    }

    fn do_cgb_dma(&mut self, value: u8) {
        self.hdma_bytes = 16 + ((value & 0x7F) as i32 * 16);

        if self.hdma_enabled {
            if bit_utils::is_set(value, 7) {
                self.high_ram[0xFF55 - 0xFF00] = value & 0x7F;
            } else {
                self.high_ram[0xFF55 - 0xFF00] = 0xFF;
                self.hdma_enabled = false;
            }
        } else if bit_utils::is_set(value, 7) {
            self.hdma_enabled = true;
            self.high_ram[0xFF55 - 0xFF00] = value & 0x7F;
            if self.lcd_status_mode == 0 {
                let _cycles = self.do_hdma();
            }
        } else {
            let _cycles = self.do_gdma(value);
        }
    }

    pub fn do_hdma(&mut self) -> i32 {
        let source = self.hdma_source & 0xFFF0;
        let destination = (self.hdma_destination & 0x1FF0) | 0x8000;

        for i in 0..0x10 {
            let value = self.read_byte(source + i);
            self.write_byte(destination + i, value);
        }

        self.hdma_destination += 0x10;
        if self.hdma_destination == 0xA000 {
            self.hdma_destination = 0x8000;
        }

        self.hdma_source += 0x10;
        if self.hdma_source == 0x8000 {
            self.hdma_source = 0xA000;
        }

        self.high_ram[0xFF51 - 0xFF00] = (self.hdma_source >> 8) as u8;
        self.high_ram[0xFF52 - 0xFF00] = (self.hdma_source & 0xFF) as u8;

        self.high_ram[0xFF53 - 0xFF00] = (self.hdma_destination >> 8) as u8;
        self.high_ram[0xFF54 - 0xFF00] = (self.hdma_destination & 0xFF) as u8;

        self.hdma_bytes -= 0x10;
        self.high_ram[0xFF55 - 0xFF00] = self.high_ram[0xFF55 - 0xFF00].wrapping_sub(1);

        if self.high_ram[0xFF55 - 0xFF00] == 0xFF {
            self.hdma_enabled = false;
        }

        9 * 4 // TODO: this needs to be the correct timing
    }

    fn do_gdma(&mut self, value: u8) -> i32 {
        let source = self.hdma_source & 0xFFF0;
        let destination = (self.hdma_destination & 0x1FF0) | 0x8000;

        for i in 0..self.hdma_bytes as u16 {
            let value = self.read_byte(source + i);
            self.write_byte(destination + i, value);
        }

        self.hdma_source += self.hdma_bytes as u16;
        self.hdma_destination += self.hdma_bytes as u16;

        for i in 0..5 {
            self.high_ram[0xFF51 - 0xFF00 + i] = 0xFF;
        }

        1 + 8 * ((value & 0x7F) as i32 * 4) // TODO: this needs to be the right timing
    }

    fn update_color_palette(&mut self, background: bool, value: u8) {
        let hl = bit_utils::is_set(value, 0);
        let index = (value >> 1) & 0x03;
        let pal = (value >> 3) & 0x07;
        let color = if background {
            self.cgb_background_palettes[pal as usize][index as usize]
        } else {
            self.cgb_sprite_palettes[pal as usize][index as usize]
        };

        let final_value = if hl {
            let blue = (color.blue & 0x1F) << 2;
            let half_green_hi = (color.green >> 3) & 0x03;
            (blue | half_green_hi) & 0x7F
        } else {
            let half_green_low = (color.green & 0x07) << 5;
            let red = color.red & 0x1F;
            (red | half_green_low)
        };

        if background {
            self.high_ram[0xFF69 - 0xFF00] = final_value;
        } else {
            self.high_ram[0xFF6B - 0xFF00] = final_value;
        }
    }

    fn set_color_palette(&mut self, background: bool, value: u8) {
        let mut ps = if background {
            self.get_background_palette_index()
        } else {
            self.get_sprite_palette_index()
        };
        let hl = bit_utils::is_set(ps, 0);
        let index = (ps >> 1) & 0x03;
        let pal = (ps >> 3) & 0x07;
        let increment = bit_utils::is_set(ps, 7);

        if increment {
            let mut address = ps & 0x3F;
            address += 1;
            address &= 0x3F;
            ps = (ps & 0x80) | address;
            if background {
                self.set_background_palette_index(ps);
            } else {
                self.set_sprite_palette_index(ps);
            }
            self.update_color_palette(background, ps);
        }

        if hl {
            let blue = (value >> 2) & 0x1F;
            let half_green_hi = (value & 0x03) << 3;

            if background {
                self.cgb_background_palettes[pal as usize][index as usize].blue = blue;
                self.cgb_background_palettes[pal as usize][index as usize].green =
                    (self.cgb_background_palettes[pal as usize][index as usize].green & 0x07)
                        | half_green_hi;
            } else {
                self.cgb_sprite_palettes[pal as usize][index as usize].blue = blue;
                self.cgb_sprite_palettes[pal as usize][index as usize].green =
                    (self.cgb_sprite_palettes[pal as usize][index as usize].green & 0x07)
                        | half_green_hi;
            }
        } else {
            let half_green_low = (value >> 5) & 0x07;
            let red = value & 0x1F;

            if background {
                self.cgb_background_palettes[pal as usize][index as usize].red = red;
                self.cgb_background_palettes[pal as usize][index as usize].green =
                    (self.cgb_background_palettes[pal as usize][index as usize].green & 0x18)
                        | half_green_low;
            } else {
                self.cgb_sprite_palettes[pal as usize][index as usize].red = red;
                self.cgb_sprite_palettes[pal as usize][index as usize].green =
                    (self.cgb_sprite_palettes[pal as usize][index as usize].green & 0x18)
                        | half_green_low;
            }
        }
    }

    pub fn read_cgb_lcd_ram(&self, index: u16, bank: i32) -> u8 {
        let offset = 0x2000 * bank as usize;
        let address = index as usize - 0x8000 + offset;
        self.vram[address]
    }

    pub fn read_word(&self, index: u16) -> u16 {
        let low = u16::from(self.read_byte(index));
        let high = u16::from(self.read_byte(index + 1));
        (high << 8) + low
    }

    pub fn write_word(&mut self, index: u16, value: u16) {
        let high = (value >> 8) as u8;
        let low = value as u8;
        self.write_byte(index, low);
        self.write_byte(index + 1, high);
    }

    pub fn get_interrupts(&self) -> Option<Interrupt> {
        let interrupts = [
            Interrupt::Vblank,
            Interrupt::Lcd,
            Interrupt::Timer,
            Interrupt::Serial,
            Interrupt::Joypad,
        ];
        let interrupt_enable = self.read_byte(INTERRUPT_ENABLE_INDEX);
        let interrupt_flags = self.read_byte(INTERRUPT_FLAGS_INDEX);
        let check = interrupt_enable & interrupt_flags;

        for (i, interrupt) in interrupts.iter().enumerate() {
            if bit_utils::is_set(check, i as u8) {
                let interrupt = interrupt.clone();
                return Some(interrupt);
            }
        }

        None
    }

    pub fn request_interrupt(&mut self, interrupt: Interrupt) {
        let mut interrupt_flag = self.read_byte(INTERRUPT_FLAGS_INDEX);
        let interrupt = interrupt as u8;
        interrupt_flag |= interrupt;
        self.write_byte(INTERRUPT_FLAGS_INDEX, interrupt_flag);
    }

    pub fn remove_interrupt(&mut self, interrupt: Interrupt) {
        let mut interrupt_flag = self.read_byte(INTERRUPT_FLAGS_INDEX);
        interrupt_flag &= !(interrupt as u8);
        self.write_byte(INTERRUPT_FLAGS_INDEX, interrupt_flag);
    }

    pub fn get_div_from_memory(&self) -> u8 {
        self.high_ram[0xFF04 - 0xFF00]
    }

    pub fn set_div_from_memory(&mut self, value: u8) {
        self.high_ram[0xFF04 - 0xFF00] = value;
    }

    fn reset_div_cycles(&mut self) {
        self.div_cycles = 0;
        self.high_ram[0xFF04 - 0xFF00] = 0;
    }

    fn reset_tima_cycles(&mut self) {
        self.tima_cycles = 0;
        self.high_ram[0xFF05 - 0xFF00] = self.read_byte(0xFF06);
    }

    pub fn get_key1(&self) -> u8 {
        self.high_ram[0xFF4D - 0xFF00]
    }

    pub fn set_key1(&mut self, value: u8) {
        self.high_ram[0xFF4D - 0xFF00] = value;
    }

    pub fn get_lcd_status_from_memory(&self) -> u8 {
        self.high_ram[0xFF41 - 0xFF00]
    }

    pub fn set_lcd_status_from_memory(&mut self, value: u8) {
        self.high_ram[0xFF41 - 0xFF00] = value;
    }

    pub fn get_lcdc_from_memory(&self) -> u8 {
        self.high_ram[0xFF40 - 0xFF00]
    }

    pub fn set_lcdc_from_memory(&mut self, value: u8) {
        self.high_ram[0xFF40 - 0xFF00] = value;
    }

    pub fn get_window_line_from_memory(&self) -> u8 {
        self.high_ram[0xFF4A - 0xFF00]
    }

    pub fn get_lyc_from_memory(&self) -> u8 {
        self.high_ram[0xFF45 - 0xFF00]
    }

    pub fn set_lyc_from_memory(&mut self, value: u8) {
        self.high_ram[0xFF45 - 0xFF00] = value;
    }

    pub fn get_background_palette_index(&self) -> u8 {
        self.high_ram[0xFF68 - 0xFF00]
    }

    pub fn set_background_palette_index(&mut self, value: u8) {
        self.high_ram[0xFF68 - 0xFF00] = value;
    }

    pub fn get_sprite_palette_index(&self) -> u8 {
        self.high_ram[0xFF6A - 0xFF00]
    }

    pub fn set_sprite_palette_index(&mut self, value: u8) {
        self.high_ram[0xFF6A - 0xFF00] = value;
    }

    pub fn is_hdma_enabled(&self) -> bool {
        self.hdma_enabled
    }

    pub fn get_cartridge(&self) -> &Cartridge {
        &self.mbc.get_cartridge()
    }

    pub fn set_ram_change_callback(&mut self, f: Box<dyn FnMut(usize, u8)>) {
        self.mbc.set_ram_change_callback(f);
    }

    pub fn get_cartridge_mut(&mut self) -> &mut Cartridge {
        self.mbc.get_cartridge_mut()
    }
}
