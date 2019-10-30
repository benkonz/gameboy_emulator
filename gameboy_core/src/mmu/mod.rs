pub mod gpu_cycles;
pub mod interrupt;
mod mbc;
mod mbc1;
mod mbc2;
mod mbc3;
mod mbc5;
mod rom_only;

use self::interrupt::Interrupt;
use self::mbc::Mbc;
use self::mbc1::Mbc1;
use self::mbc2::Mbc2;
use self::mbc3::Mbc3;
use self::mbc5::Mbc5;
use self::rom_only::RomOnly;
use gpu::lcd_control_flag::LcdControlFlag;
use mmu::gpu_cycles::GpuCycles;

// TODO: add all IO Register INDEX's here
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

pub struct Memory {
    mbc: Box<dyn Mbc>,
    wram_banks: [u8; 0x2000],
    vram_banks: [u8; 0x2000],
    oam: [u8; 0x100],
    high_ram: [u8; 0x100],
    joypad_state: u8,
    pub scan_line: u8,
    pub irq48_signal: u8,
    pub screen_disabled: bool,
    pub lcd_status_mode: u8,
    // TODO make this private with a getter
    pub gpu_cycles: GpuCycles,
    // TODO make this into a private struct
    pub div_cycles: i32,
    pub tima_cycles: i32,
}

impl Memory {
    pub fn from_rom(rom: Vec<u8>) -> Memory {
        let cartridge_type = rom[0x0147];
        let rom_size = rom[0x0148];
        let num_rom_banks = match rom_size {
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
        let ram_size = rom[0x0149];
        let num_ram_banks = match ram_size {
            0x0 => 0,
            0x1 => 1,
            0x2 => 1,
            0x3 => 4,
            0x4 => 16,
            _ => panic!("Unknown number of RAM banks"),
        };
        println!("cartridge type: {:02X}", cartridge_type);
        println!("rom banks: {}", num_rom_banks);
        println!("ram banks: {}", num_ram_banks);
        println!("ram size: {}", ram_size);

        let mbc: Box<dyn Mbc> = match cartridge_type {
            0x00 => Box::new(RomOnly::new(&rom[..])),
            0x01..=0x03 => Box::new(Mbc1::new(num_rom_banks, num_ram_banks, &rom[..])),
            0x05..=0x09 => Box::new(Mbc2::new(num_rom_banks, &rom[..])),
            0x0F..=0x13 => Box::new(Mbc3::new(num_rom_banks, num_ram_banks, &rom[..])),
            0x19..=0x1E => Box::new(Mbc5::new(num_rom_banks, num_ram_banks, &rom[..])),
            _ => panic!("Unsupported cartridge: {:02X}", cartridge_type),
        };

        let high_ram = INITIAL_VALUES_FOR_FFXX;

        Memory {
            mbc,
            vram_banks: [0; 0x2000],
            wram_banks: [0; 0x2000],
            oam: [0; 0x100],
            high_ram,
            scan_line: 144,
            joypad_state: 0,
            irq48_signal: 0,
            screen_disabled: false,
            lcd_status_mode: 1,
            gpu_cycles: Default::default(),
            div_cycles: 0,
            tima_cycles: 0,
        }
    }

    pub fn read_byte(&self, index: u16) -> u8 {
        match index {
            0x0000..=0x7FFF => self.mbc.read_byte(index),
            0x8000..=0x9FFF => self.vram_banks[index as usize - 0x8000],
            0xA000..=0xBFFF => self.mbc.read_byte(index),
            0xC000..=0xDFFF => self.wram_banks[index as usize - 0xC000],
            0xE000..=0xFDFF => self.read_byte(index as u16 - 0x2000),
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
        joypad_control & 0b0010_0000 == 0
    }

    pub fn are_direction_keys_enabled(&self) -> bool {
        let joypad_control = self.high_ram[0];
        joypad_control & 0b0001_0000 == 0
    }

    pub fn write_byte(&mut self, index: u16, value: u8) {
        match index {
            0x0000..=0x7FFF => self.mbc.write_byte(index, value),
            0x8000..=0x9FFF => self.vram_banks[index as usize - 0x8000] = value,
            0xA000..=0xBFFF => self.mbc.write_byte(index, value),
            0xC000..=0xDFFF => self.wram_banks[index as usize - 0xC000] = value,
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
                0xFF40 => {
                    let current_lcdc = LcdControlFlag::from_bits_truncate(self.read_byte(0xFF40));
                    let new_lcdc = LcdControlFlag::from_bits_truncate(value);
                    self.high_ram[index as usize - 0xFF00] = value;

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
                0xFF41 => {
                    let current_stat = self.read_byte(0xFF41) & 0x07;
                    let new_stat = (value & 0x78) | (current_stat & 0x07);
                    self.set_lcd_status_from_memory(new_stat);
                    let lcd_control = LcdControlFlag::from_bits_truncate(self.read_byte(0xFF40));
                    let mut signal = self.irq48_signal;
                    let mode = self.lcd_status_mode;
                    signal &= (new_stat >> 3) & 0x0F;
                    self.irq48_signal = signal;

                    if lcd_control.contains(LcdControlFlag::DISPLAY) {
                        if new_stat & 0b0000_1000 == 0b0000_1000 && mode == 0 {
                            if signal == 0 {
                                self.request_interrupt(Interrupt::Lcd);
                            }
                            signal |= 0b01;
                        }

                        if new_stat & 0b0001_0000 == 0b0001_0000 && mode == 1 {
                            if signal == 0 {
                                self.request_interrupt(Interrupt::Lcd);
                            }
                            signal |= 0b10;
                        }

                        if new_stat & 0b0010_0000 == 0b0010_0000 && mode == 2 && signal == 0 {
                            self.request_interrupt(Interrupt::Lcd);
                        }
                        self.compare_ly_to_lyc();
                    }
                }
                0xFF44 => {
                    let current_ly = self.scan_line;
                    if current_ly & 0b1000_0000 == 0b1000_0000 && value & 0b1000_0000 != 0b1000_0000
                    {
                        self.disable_screen();
                    }
                }
                0xFF45 => {
                    let current_lyc = self.read_byte(0xFF45);
                    if current_lyc != value {
                        self.high_ram[index as usize - 0xFF00] = value;
                        let lcd_control =
                            LcdControlFlag::from_bits_truncate(self.read_byte(0xFF40));
                        if lcd_control.contains(LcdControlFlag::DISPLAY) {
                            self.compare_ly_to_lyc();
                        }
                    }
                }
                0xFF46 => {
                    self.high_ram[index as usize - 0xFF00] = value;
                    self.do_dma_transfer(value)
                }
                0xFFFF => self.high_ram[index as usize - 0xFF00] = value & 0x1F,
                _ => self.high_ram[index as usize - 0xFF00] = value,
            },
            _ => {}
        };
    }

    fn do_dma_transfer(&mut self, data: u8) {
        let address = 0x100 * u16::from(data);
        if address >= 0x8000 && address < 0xE000 {
            for i in 0..0xA0 {
                let value = self.read_byte(address + i);
                self.write_byte(0xFE00 + i, value);
            }
        }
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
            if check & (1 << i) != 0 {
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

    pub fn get_lcd_status_from_memory(&self) -> u8 {
        self.high_ram[0xFF41 - 0xFF00]
    }

    pub fn set_lcd_status_from_memory(&mut self, value: u8) {
        self.high_ram[0xFF41 - 0xFF00] = value;
    }

    pub fn compare_ly_to_lyc(&mut self) {
        if !self.screen_disabled {
            let lyc = self.read_byte(0xFF45);
            let mut stat = self.get_lcd_status_from_memory();

            if lyc == self.scan_line {
                stat |= 0b0000_0100;
                if stat & 0b0100_0000 == 0b0100_0000 {
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
        let mut stat = self.read_byte(0xFF41);
        stat &= 0x7C;
        self.set_lcd_status_from_memory(stat);
        self.lcd_status_mode = 0;
        self.gpu_cycles.cycles_counter = 0;
        self.gpu_cycles.aux_cycles_counter = 0;
        self.scan_line = 0;
        self.irq48_signal = 0;
    }

    pub fn reset_window_line(&mut self) {
        let wy = self.read_byte(0xFF4A);

        if (self.gpu_cycles.window_line == 0) && (self.scan_line < 144) && (self.scan_line > wy) {
            self.gpu_cycles.window_line = 144;
        }
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
}
