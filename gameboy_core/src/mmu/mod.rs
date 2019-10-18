pub mod gpu_cycles;
pub mod interrupt;

use self::interrupt::Interrupt;
use gpu::lcd_control_flag::LcdControlFlag;
use mmu::gpu_cycles::GpuCycles;
use std::cmp;

const BIOS: &[u8] = include_bytes!("bios.gb");
const INTERRUPT_ENABLE_INDEX: u16 = 0xFFFF;
const INTERRUPT_FLAGS_INDEX: u16 = 0xFF0F;

const ROM_ONLY: u8 = 0x0;
const ROM_MBC1: u8 = 0x1;
const ROM_MBC1_RAM: u8 = 0x2;
const ROM_MBC1_RAM_BATT: u8 = 0x3;
const _ROM_MBC2: u8 = 0x5;
const _ROM_MBC2_BATT: u8 = 0x6;
const _ROM_RAM: u8 = 0x8;
const _ROM_RAM_BATT: u8 = 0x9;
const _ROM_MMM01: u8 = 0xB;
const _ROM_MMM01_SRAM: u8 = 0xC;
const _ROM_MMM01_SRAM_BATT: u8 = 0xD;
const ROM_MBC3_TIMER_BATT: u8 = 0xF;
const ROM_MBC3_TIMER_RAM_BATT: u8 = 0x10;
const ROM_MBC3: u8 = 0x11;
const ROM_MBC3_RAM: u8 = 0x12;
const ROM_MBC3_RAM_BATT: u8 = 0x13; // TODO: needed for pokemon
const _ROM_MBC5: u8 = 0x19;
const _ROM_MBC5_RAM: u8 = 0x1A;
const _ROM_MBC5_RAM_BATT: u8 = 0x1B;
const _ROM_MBC5_RUMBLE: u8 = 0x1C;
const _ROM_MBC5_RUMBLE_SRAM: u8 = 0x1D;
const _ROM_MBC5_RUMBLE_SRAM_BATT: u8 = 0x1E;
const _MBC6: u8 = 0x20;
const _MBC7_SENSOR_RUMBLE_RAM_BATT: u8 = 0x22;
const _POCKET_CAMERA: u8 = 0xFC;
const _BANDAI_TAMA5: u8 = 0xFD;
const _HUDSON_HU_C3: u8 = 0xFE;
const _HUDSON_HU_C1_RAM_BATTERY: u8 = 0xFF;

pub struct Memory {
    rom_banks: Vec<[u8; 0x4000]>,
    eram_banks: Vec<[u8; 0x2000]>,
    wram_banks: Vec<[u8; 0x1000]>,
    vram_banks: Vec<[u8; 0x2000]>,
    oam: [u8; 0x100],
    high_ram: [u8; 0x200],
    pub disable_bios: u8,
    selected_rom_bank: u8,
    selected_eram_bank: u8,
    selected_wram_bank: u8,
    selected_vram_bank: u8,
    joypad_state: u8,
    cartridge_type: u8,
    in_ram_banking_mode: bool,
    external_ram_enabled: bool,
    higher_rom_bank_bits: u8,
    pub scan_line: u8,
    pub irq48_signal: u8,
    pub screen_disabled: bool,
    pub lcd_status_mode: u8,
    // TODO make this private with a getter
    pub gpu_cycles: GpuCycles,
    // TODO make this into a private struct
    pub div_cycles: i32,
    pub tima_cycles: i32

}

impl Memory {
    pub fn new() -> Memory {
        // TODO: delete this at some point in time, only used in CPU tests
        Memory {
            rom_banks: vec![[0; 0x4000]; 2],
            vram_banks: vec![[0; 0x2000]],
            eram_banks: vec![[0; 0x2000]],
            wram_banks: vec![[0; 0x1000]; 2],
            oam: [0; 0x100],
            high_ram: [0; 0x200],
            disable_bios: 0,
            selected_rom_bank: 1,
            selected_vram_bank: 0,
            selected_eram_bank: 0,
            selected_wram_bank: 1,
            scan_line: 144,
            joypad_state: 0,
            cartridge_type: ROM_ONLY,
            in_ram_banking_mode: false,
            external_ram_enabled: false,
            irq48_signal: 0,
            screen_disabled: false,
            lcd_status_mode: 0,
            gpu_cycles: GpuCycles::new(),
            higher_rom_bank_bits: 0,
            div_cycles: 0,
            tima_cycles: 0
        }
    }

    pub fn from_rom(rom: Vec<u8>) -> Memory {
        let cartridge_type = rom[0x0147];
        // if cartridge_type > 0x3 {
        //     panic!("unsupported cartridge type: {:02X} (for now...)", cartridge_type);
        // }
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
        println!("{:02X}", cartridge_type);
        println!("{}", num_rom_banks);
        println!("{}", num_ram_banks);
        let mut rom_banks: Vec<[u8; 0x4000]> = vec![[0; 0x4000]; num_rom_banks];
        for (i, bank) in rom_banks.iter_mut().enumerate() {
            let start = i * 0x4000;
            let end = cmp::min(start + 0x4000, rom.len());
            bank.copy_from_slice(&rom[start..end]);
            if end == rom.len() {
                break;
            }
        }

        let eram_banks: Vec<[u8; 0x2000]> = vec![[0; 0x2000]; 8];
        Memory {
            rom_banks,
            vram_banks: vec![[0; 0x2000]],
            eram_banks,
            wram_banks: vec![[0; 0x1000]; 2],
            oam: [0; 0x100],
            high_ram: [0; 0x200],
            disable_bios: 0,
            selected_rom_bank: 1,
            selected_vram_bank: 0,
            selected_eram_bank: 0,
            selected_wram_bank: 1,
            scan_line: 0,
            joypad_state: 0,
            cartridge_type,
            in_ram_banking_mode: false,
            external_ram_enabled: false,
            irq48_signal: 0,
            screen_disabled: false,
            lcd_status_mode: 0,
            gpu_cycles: GpuCycles::new(),
            higher_rom_bank_bits: 0,
            div_cycles: 0,
            tima_cycles: 0
        }
    }

    pub fn read_byte(&self, index: u16) -> u8 {
        let index = index as usize;

        match index {
            0x0000..=0x00FF => {
                if self.disable_bios == 0 {
                    BIOS[index]
                } else {
                    self.rom_banks[0][index]
                }
            }
            0x0100..=0x3FFF => self.rom_banks[0][index],
            0x4000..=0x7FFF => self.rom_banks[self.selected_rom_bank as usize][index - 0x4000],
            0x8000..=0x9FFF => self.vram_banks[self.selected_vram_bank as usize][index - 0x8000],
            0xA000..=0xBFFF => {
                let selected_bank = if self.in_ram_banking_mode {
                    self.selected_eram_bank as usize
                } else {
                    0
                };
                self.eram_banks[selected_bank][index - 0xA000]
            }
            0xC000..=0xCFFF => self.wram_banks[0][index - 0xC000],
            0xD000..=0xDFFF => self.wram_banks[self.selected_wram_bank as usize][index - 0xD000],
            0xE000..=0xFDFF => self.read_byte(index as u16 - 0x2000),
            0xFE00..=0xFE9F => self.oam[index - 0xFE00],
            0xFF00..=0xFFFF => match index {
                0xFF00 => self.get_joypad_state(),
                0xFF07 => self.high_ram[index - 0xFF00] | 0xF8,
                0xFF0F => self.high_ram[index - 0xFF00] | 0xE0,
                0xFF41 => self.high_ram[index - 0xFF00] | 0b01000_0000,
                0xFF44 => {
                    if !self.screen_disabled {
                        self.scan_line
                    } else {
                        0x00
                    }
                }
                0xFF50 => self.disable_bios,
                _ => self.high_ram[index - 0xFF00],
            },
            _ => 0,
        }
    }

    fn get_joypad_state(&self) -> u8 {
        let joypad_control = self.high_ram[0x100];

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

    fn cartridge_has_mbc1(&self) -> bool {
        self.cartridge_type == ROM_MBC1
            || self.cartridge_type == ROM_MBC1_RAM
            || self.cartridge_type == ROM_MBC1_RAM_BATT
    }

    fn cartridge_has_mbc3(&self) -> bool {
        self.cartridge_type == ROM_MBC3
            || self.cartridge_type == ROM_MBC3_RAM
            || self.cartridge_type == ROM_MBC3_RAM_BATT
            || self.cartridge_type == ROM_MBC3_TIMER_BATT
            || self.cartridge_type == ROM_MBC3_TIMER_RAM_BATT
    }

    pub fn write_byte(&mut self, index: u16, value: u8) {
        let index = index as usize;

        match index {
            0x0000..=0x1FFF if self.cartridge_has_mbc1() => {
                self.external_ram_enabled = value & 0b0000_1010 == 0b0000_1010
            }
            0x0000..=0x1FFF if self.cartridge_has_mbc3() => {
                self.external_ram_enabled =  value & 0b0000_1010 == 0b0000_1010
            }
            0x2000..=0x3FFF if self.cartridge_has_mbc1() => {
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

                self.selected_rom_bank &= (self.rom_banks.len() - 1) as u8;
            }
            0x2000..=0x3FFF if self.cartridge_has_mbc3() => {
                let mut value = value & 0b0111_1111;
                if value == 0 {
                    value = 1;
                }

                self.selected_rom_bank = value;
            }
            0x4000..=0x5FFF if self.cartridge_has_mbc1() => {
                if self.in_ram_banking_mode {
                    self.selected_eram_bank = value & 0x03;
                    self.selected_eram_bank &= (self.eram_banks.len() - 1) as u8;
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
                    self.selected_rom_bank &= (self.rom_banks.len() - 1) as u8;
                }
            }
            0x4000..=0x5FFF if self.cartridge_has_mbc3() => {
                match value {
                    0x00..=0x07 => {
                        self.selected_eram_bank = value;
                        self.selected_eram_bank &= (self.eram_banks.len() - 1) as u8;
                    },
                    0x08..=0x0C => panic!("RTC not implemented!"),
                    _ => panic!("selecting unknown register: {:02X}", value)
                };
            }
            0x6000..=0x7FFF if self.cartridge_has_mbc1() => {
                self.in_ram_banking_mode = value & 0x01 == 0x01
            }
            0x8000..=0x9FFF => {
                self.vram_banks[self.selected_vram_bank as usize][index - 0x8000] = value
            }
            0xA000..=0xBFFF => {
                if self.in_ram_banking_mode {
                    self.eram_banks[self.selected_eram_bank as usize][index - 0xA000] = value;
                } else {
                    self.eram_banks[0][index - 0xA000] = value;
                }
            }
            0xC000..=0xCFFF => self.wram_banks[0][index - 0xC000] = value,
            0xD000..=0xDFFF => {
                self.wram_banks[self.selected_wram_bank as usize][index - 0xD000] = value
            }
            0xFE00..=0xFE9F => self.oam[index - 0xFE00] = value,
            0xFF00..=0xFFFF => match index {
                0xFF04 => self.reset_div_cycles(),
                0xFF07 => {
                    let value = value & 0x07;
                    let current_tac = self.read_byte(0xFF07);
                    if (current_tac & 0x03) != (value & 0x03) {
                        self.reset_tima_cycles();
                    }
                    self.high_ram[index - 0xFF00] = value;
                }
                0xFF0F => self.high_ram[index - 0xFF00] = value & 0x1F,
                0xFF40 => {
                    let current_lcdc = LcdControlFlag::from_bits_truncate(self.read_byte(0xFF40));
                    let new_lcdc = LcdControlFlag::from_bits_truncate(value);
                    self.high_ram[index - 0xFF00] = value;

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

                        if new_stat & 0b0010_0000 == 0b0010_0000 && mode == 2 {
                            if signal == 0 {
                                self.request_interrupt(Interrupt::Lcd);
                            }
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
                        self.high_ram[index - 0xFF00] = value;
                        let lcd_control =
                            LcdControlFlag::from_bits_truncate(self.read_byte(0xFF40));
                        if lcd_control.contains(LcdControlFlag::DISPLAY) {
                            self.compare_ly_to_lyc();
                        }
                    }
                }
                0xFF46 => {
                    self.high_ram[index - 0xFF00] = value;
                    self.do_dma_transfer(value)
                }
                0xFF50 => self.disable_bios = value,
                0xFFFF => self.high_ram[index - 0xFF00] = value & 0x1F,
                _ => self.high_ram[index - 0xFF00] = value
            },
            _ => {}
        };
    }

    fn do_dma_transfer(&mut self, data: u8) {
        let address = 0x100u16 * data as u16;
        if address >= 0x8000 && address < 0xE000 {
            for i in 0..0xA0 {
                let value = self.read_byte(address + i);
                self.write_byte(0xFE00 + i, value);
            }
        }
    }

    pub fn read_word(&self, index: u16) -> u16 {
        let low = self.read_byte(index) as u16;
        let high = self.read_byte(index + 1) as u16;
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

impl Default for Memory {
    fn default() -> Self {
        Memory::new()
    }
}

#[cfg(test)]
mod tests {
    use mmu::Memory;

    #[test]
    fn test_write_byte() {
        let mut memory = Memory::new();
        memory.write_byte(0xFF80, 1);

        assert_eq!(memory.read_byte(0xFF80), 1);
    }

    #[test]
    fn test_write_word() {
        let mut memory = Memory::new();
        memory.write_word(0xFF80, 0x1122);
        assert_eq!(memory.read_word(0xFF80), 0x1122);
    }
}
