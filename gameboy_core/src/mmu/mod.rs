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
use sound::Sound;

pub const SPRITES_START_INDEX: u16 = 0xFE00;
pub const JOYPAD_INDEX: u16 = 0xFF00;
pub const DIVIDER_INDEX: u16 = 0xFF04;
pub const SELECTABLE_TIMER_INDEX: u16 = 0xFF05;
pub const TIMER_RESET_INDEX: u16 = 0xFF06;
pub const TIMER_CONTROL_INDEX: u16 = 0xFF07;
pub const INTERRUPT_FLAGS_INDEX: u16 = 0xFF0F;
const APU_INDEX_START: u16 = 0xFF10;
const APU_INDEX_END: u16 = 0xFF3F;
pub const LCD_CONTROL_INDEX: u16 = 0xFF40;
pub const LCD_INDEX: u16 = 0xFF41;
pub const SCROLL_Y_INDEX: u16 = 0xFF42;
pub const SCROLL_X_INDEX: u16 = 0xFF43;
pub const LY_INDEX: u16 = 0xFF44;
pub const LYC_INDEX: u16 = 0xFF45;
pub const BACKGROUND_PALETTE_INDEX: u16 = 0xFF47;
pub const OBJECT_PALETTE_0_INDEX: u16 = 0xFF48;
pub const OBJECT_PALETTE_1_INDEX: u16 = 0xFF49;
pub const WINDOW_Y_INDEX: u16 = 0xFF4A;
pub const WINDOW_X_INDEX: u16 = 0xFF4B;
pub const VRAM_BANK_INDEX: u16 = 0xFF4F;
pub const CGB_BACKGROUND_PALETTE_INDEX_INDEX: u16 = 0xFF68;
pub const CGB_BACKGROUND_PALETTE_DATA_INDEX: u16 = 0xFF69;
pub const CGB_SPRITE_PALETTE_INDEX_INDEX: u16 = 0xFF6A;
pub const CGB_SPRITE_PALETTE_DATA_INDEX: u16 = 0xFF6B;
pub const INTERRUPT_ENABLE_INDEX: u16 = 0xFFFF;

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
    sound: Sound,
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
            // TODO: make all of these MMU constants
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

        // setup initial values for the sound module
        let mut sound = Sound::new();
        for i in 0xFF10..=0xFF3F {
            let value = if is_cgb {
                INITIAL_VALUES_FOR_COLOR_FFXX[i - 0xFF00]
            } else {
                INITIAL_VALUES_FOR_FFXX[i - 0xFF00]
            };
            sound.write_byte(i as u16, value);
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
            sound,
        }
    }

    pub fn read_byte(&self, index: u16) -> u8 {
        match index {
            0x0000..=0x7FFF => self.mbc.read_byte(index),
            0x8000..=0x9FFF => self.read_cgb_lcd_ram(index, self.vram_bank),
            0xA000..=0xBFFF => self.mbc.read_byte(index),
            0xC000..=0xCFFF => self.read_cgb_wram(index - 0xC000, 0),
            0xD000..=0xDFFF => self.read_cgb_wram(index - 0xD000, self.wram_bank),
            0xE000..=0xFDFF => self.read_byte(index - 0x2000),
            0xFE00..=0xFEFF => self.oam[index as usize - 0xFE00],
            0xFF00..=0xFFFF => match index {
                JOYPAD_INDEX => self.get_joypad_state(),
                DIVIDER_INDEX => self.load(index),
                SELECTABLE_TIMER_INDEX => self.load(index),
                TIMER_RESET_INDEX => self.load(index),
                TIMER_CONTROL_INDEX => self.load(index) | 0xF8,
                INTERRUPT_FLAGS_INDEX => self.load(index) | 0xE0,
                APU_INDEX_START..=APU_INDEX_END => self.sound.read_byte(index),
                LCD_CONTROL_INDEX => self.load(index),
                LCD_INDEX => self.load(index) | 0x80,
                SCROLL_Y_INDEX => self.load(index),
                SCROLL_X_INDEX => self.load(index),
                LY_INDEX => {
                    if !self.screen_disabled {
                        self.scan_line
                    } else {
                        0x00
                    }
                }
                LYC_INDEX => self.load(index),
                WINDOW_Y_INDEX => self.load(index),
                WINDOW_X_INDEX => self.load(index),
                BACKGROUND_PALETTE_INDEX => self.load(index),
                OBJECT_PALETTE_0_INDEX => self.load(index),
                OBJECT_PALETTE_1_INDEX => self.load(index),
                VRAM_BANK_INDEX => self.load(index),
                CGB_BACKGROUND_PALETTE_INDEX_INDEX | CGB_SPRITE_PALETTE_INDEX_INDEX => {
                    if self.is_cgb {
                        self.load(index) | 0x40
                    } else {
                        0xC0
                    }
                }
                CGB_BACKGROUND_PALETTE_DATA_INDEX | CGB_SPRITE_PALETTE_DATA_INDEX => {
                    if self.is_cgb {
                        self.load(index) | 0xF8
                    } else {
                        0xFF
                    }
                }
                _ => self.load(index),
            },
        }
    }

    fn get_joypad_state(&self) -> u8 {
        let joypad_control = self.load(JOYPAD_INDEX);

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
        let joypad_control = self.load(0xFF00);
        joypad_control & 0x30 != 0x20
    }

    pub fn are_direction_keys_enabled(&self) -> bool {
        let joypad_control = self.load(0xFF00);
        joypad_control & 0x30 != 0x10
    }

    pub fn write_byte(&mut self, index: u16, value: u8) {
        match index {
            0x0000..=0x7FFF => self.mbc.write_byte(index, value),
            0x8000..=0x9FFF => self.write_cgb_lcd_ram(index, value, self.vram_bank),
            0xA000..=0xBFFF => self.mbc.write_byte(index, value),
            0xC000..=0xCFFF => self.write_cgb_wram(index - 0xC000, value, 0),
            0xD000..=0xDFFF => self.write_cgb_wram(index - 0xD000, value, self.wram_bank),
            0xE000..=0xFDFF => self.write_byte(index - 0x2000, value),
            0xFE00..=0xFEFF => self.oam[index as usize - 0xFE00] = value,
            0xFF00..=0xFFFF => match index {
                DIVIDER_INDEX => self.reset_div_cycles(),
                SELECTABLE_TIMER_INDEX => self.store(index, value),
                TIMER_RESET_INDEX => self.store(index, value),
                TIMER_CONTROL_INDEX => self.store(index, value),
                INTERRUPT_FLAGS_INDEX => self.store(index, value & 0x1F),
                APU_INDEX_START..=APU_INDEX_END => self.sound.write_byte(index, value),
                LCD_CONTROL_INDEX => self.do_lcd_control_write(value),
                LCD_INDEX => self.do_lcd_status_write(value),
                SCROLL_Y_INDEX => self.store(index, value),
                SCROLL_X_INDEX => self.store(index, value),
                LY_INDEX => self.do_scanline_write(value),
                LYC_INDEX => self.do_lyc_write(value),
                0xFF46 => {
                    self.store(index, value);
                    self.do_dma_transfer(value)
                }
                BACKGROUND_PALETTE_INDEX => self.store(index, value),
                OBJECT_PALETTE_0_INDEX => self.store(index, value),
                OBJECT_PALETTE_1_INDEX => self.store(index, value),
                WINDOW_Y_INDEX => self.store(index, value),
                WINDOW_X_INDEX => self.store(index, value),
                0xFF4D if self.is_cgb => {
                    let current_key1 = self.load(index);
                    self.store(index, (current_key1 & 0x80) | (value & 1) | 0x7E);
                }
                VRAM_BANK_INDEX if self.is_cgb => {
                    let value = value & 1;
                    self.vram_bank = value as i32;
                    self.store(index, value);
                }
                0xFF51 if self.is_cgb => {
                    let value = if value > 0x7F && value < 0xC0 {
                        0
                    } else {
                        value
                    };
                    self.hdma_source = ((value as u16) << 8) | (self.hdma_source & 0xF0);
                    self.store(index, value);
                }
                0xFF52 if self.is_cgb => {
                    let value = value & 0xF0;
                    self.hdma_source = (self.hdma_source & 0xFF00) | (value as u16);
                    self.store(index, value);
                }
                0xFF53 if self.is_cgb => {
                    let value = value & 0x1F;
                    self.hdma_destination = ((value as u16) << 8) | (self.hdma_destination & 0xF0);
                    self.hdma_destination |= 0x8000;
                    self.store(index, value);
                }
                0xFF54 if self.is_cgb => {
                    let value = value & 0xF0;
                    self.hdma_destination = (self.hdma_destination & 0x1F00) | (value as u16);
                    self.hdma_destination |= 0x8000;
                    self.store(index, value);
                }
                0xFF55 if self.is_cgb => self.do_cgb_dma(value),
                CGB_BACKGROUND_PALETTE_INDEX_INDEX if self.is_cgb => {
                    self.store(index, value);
                    self.update_color_palette(true, value);
                }
                CGB_BACKGROUND_PALETTE_DATA_INDEX if self.is_cgb => {
                    self.store(index, value);
                    self.set_color_palette(true, value);
                }
                CGB_SPRITE_PALETTE_INDEX_INDEX if self.is_cgb => {
                    self.store(index, value);
                    self.update_color_palette(false, value);
                }
                CGB_SPRITE_PALETTE_DATA_INDEX if self.is_cgb => {
                    self.store(index, value);
                    self.set_color_palette(false, value);
                }
                0xFF70 if self.is_cgb => {
                    let value = value & 0x07;
                    self.wram_bank = value as i32;
                    if self.wram_bank == 0 {
                        self.wram_bank = 1;
                    }
                    self.store(index, value);
                }
                INTERRUPT_ENABLE_INDEX => self.store(index, value & 0x1F),
                _ => self.store(index, value),
            },
        };
    }

    pub fn do_lcd_control_write(&mut self, value: u8) {
        let current_lcdc = LcdControlFlag::from_bits_truncate(self.load(LCD_CONTROL_INDEX));
        let new_lcdc = LcdControlFlag::from_bits_truncate(value);
        self.store(LCD_CONTROL_INDEX, value);

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
        let current_stat = self.load(LCD_INDEX) & 0x07;
        let new_stat = (value & 0x78) | (current_stat & 0x07);
        self.store(LCD_INDEX, new_stat);
        let lcd_control = LcdControlFlag::from_bits_truncate(self.load(LCD_CONTROL_INDEX));
        let mut signal = self.irq48_signal;
        let mode = self.lcd_status_mode;
        signal &= (new_stat >> 3) & 0x0F;
        self.irq48_signal = signal;

        if lcd_control.contains(LcdControlFlag::DISPLAY) {
            if bit_utils::is_set(new_stat, 3) && mode == 0 {
                if signal == 0 {
                    self.request_interrupt(Interrupt::Lcd);
                }
                signal = bit_utils::set_bit(signal, 0);
            }

            if bit_utils::is_set(new_stat, 4) && mode == 1 {
                if signal == 0 {
                    self.request_interrupt(Interrupt::Lcd);
                }
                signal = bit_utils::set_bit(signal, 1);
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
        let current_lyc = self.load(LYC_INDEX);
        if current_lyc != value {
            self.store(LYC_INDEX, value);
            let lcd_control = LcdControlFlag::from_bits_truncate(self.load(LCD_CONTROL_INDEX));
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
                self.write_byte(SPRITES_START_INDEX + i, value);
            }
        }
    }

    pub fn compare_ly_to_lyc(&mut self) {
        if !self.screen_disabled {
            let lyc = self.load(LYC_INDEX);
            let mut stat = self.load(LCD_INDEX);

            if lyc == self.scan_line {
                stat = bit_utils::set_bit(stat, 2);
                if bit_utils::is_set(stat, 6) {
                    if self.irq48_signal == 0 {
                        self.request_interrupt(Interrupt::Lcd);
                    }
                    self.irq48_signal = bit_utils::set_bit(self.irq48_signal, 3);
                }
            } else {
                stat = bit_utils::unset_bit(stat, 2);
                self.irq48_signal = bit_utils::unset_bit(self.irq48_signal, 3);
            }
            self.store(LCD_INDEX, stat);
        }
    }

    pub fn enable_screen(&mut self) {
        if self.screen_disabled {
            self.gpu_cycles.screen_enable_delay_cycles = 244;
        }
    }

    pub fn disable_screen(&mut self) {
        self.screen_disabled = true;
        let mut stat = self.load(LCD_INDEX);
        stat &= 0x7C;
        self.store(LCD_INDEX, stat);
        self.lcd_status_mode = 0;
        self.gpu_cycles.cycles_counter = 0;
        self.gpu_cycles.aux_cycles_counter = 0;
        self.scan_line = 0;
        self.irq48_signal = 0;
    }

    pub fn reset_window_line(&mut self) {
        let wy = self.load(WINDOW_Y_INDEX);

        if (self.gpu_cycles.window_line == 0) && (self.scan_line < 144) && (self.scan_line > wy) {
            self.gpu_cycles.window_line = 144;
        }
    }

    fn do_cgb_dma(&mut self, value: u8) {
        self.hdma_bytes = 16 + ((value & 0x7F) as i32 * 16);

        if self.hdma_enabled {
            if bit_utils::is_set(value, 7) {
                self.store(0xFF55, value & 0x7F);
            } else {
                self.store(0xFF55, 0xFF);
                self.hdma_enabled = false;
            }
        } else if bit_utils::is_set(value, 7) {
            self.hdma_enabled = true;
            self.store(0xFF55, value & 0x7F);
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

        self.store(0xFF51, (self.hdma_source >> 8) as u8);
        self.store(0xFF52, (self.hdma_source & 0xFF) as u8);

        self.store(0xFF53, (self.hdma_destination >> 8) as u8);
        self.store(0xFF54, (self.hdma_destination & 0xFF) as u8);

        self.hdma_bytes -= 0x10;
        self.store(0xFF55, self.load(0xFF55).wrapping_sub(1));

        if self.load(0xFF55) == 0xFF {
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
            self.store(0xFF51 + i, 0xFF);
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
            self.store(CGB_BACKGROUND_PALETTE_DATA_INDEX, final_value);
        } else {
            self.store(CGB_SPRITE_PALETTE_DATA_INDEX, final_value);
        }
    }

    fn set_color_palette(&mut self, background: bool, value: u8) {
        let mut ps = if background {
            self.load(CGB_BACKGROUND_PALETTE_INDEX_INDEX)
        } else {
            self.load(CGB_SPRITE_PALETTE_INDEX_INDEX)
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
                self.store(CGB_BACKGROUND_PALETTE_INDEX_INDEX, ps);
            } else {
                self.store(CGB_SPRITE_PALETTE_INDEX_INDEX, ps);
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

    fn write_cgb_lcd_ram(&mut self, index: u16, value: u8, bank: i32) {
        let offset = 0x2000 * bank as usize;
        let address = index as usize - 0x8000 + offset;
        self.vram[address] = value;
    }

    fn read_cgb_wram(&self, index: u16, bank: i32) -> u8 {
        let offset = 0x1000 * bank as usize;
        let address = index as usize + offset;
        self.wram[address]
    }

    fn write_cgb_wram(&mut self, index: u16, value: u8, bank: i32) {
        let offset = 0x1000 * bank as usize;
        let address = index as usize + offset;
        self.wram[address] = value;
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
        interrupt_flag = bit_utils::set_bit(interrupt_flag, interrupt);
        self.write_byte(INTERRUPT_FLAGS_INDEX, interrupt_flag);
    }

    pub fn remove_interrupt(&mut self, interrupt: Interrupt) {
        let mut interrupt_flag = self.read_byte(INTERRUPT_FLAGS_INDEX);
        let interrupt = interrupt as u8;
        interrupt_flag = bit_utils::unset_bit(interrupt_flag, interrupt);
        self.write_byte(INTERRUPT_FLAGS_INDEX, interrupt_flag);
    }

    fn reset_div_cycles(&mut self) {
        self.store(DIVIDER_INDEX, 0);
    }

    pub fn load(&self, index: u16) -> u8 {
        self.high_ram[index as usize - 0xFF00]
    }

    pub fn store(&mut self, index: u16, value: u8) {
        self.high_ram[index as usize - 0xFF00] = value
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

    pub fn get_sound(&self) -> &Sound {
        &self.sound
    }

    pub fn get_sound_mut(&mut self) -> &mut Sound {
        &mut self.sound
    }
}
