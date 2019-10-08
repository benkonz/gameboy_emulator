pub mod interrupt;
pub mod gpu_cycles;

use mmu::gpu_cycles::GpuCycles;
use self::interrupt::Interrupt;
use gpu::lcd_control_flag::LcdControlFlag;

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
const _ROM_MBC3_TIMER_BATT: u8 = 0xF;
const _ROM_MBC3_TIMER_RAM_BATT: u8 = 0x10;
const _ROM_MBC3: u8 = 0x11;
const _ROM_MBC3_RAM: u8 = 0x12;
const _ROM_MBC3_RAM_BATT: u8 = 0x13;
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
    disable_bios: u8,
    selected_rom_bank: u8,
    selected_eram_bank: u8,
    selected_wram_bank: u8,
    selected_vram_bank: u8,
    joypad_state: u8,
    cartridge_type: u8,
    in_ram_banking_mode: bool,
    external_ram_enabled: bool,
    pub divider_register: u8,
    pub scan_line: u8,
    pub irq48_signal: u8,
    pub screen_disabled: bool,
    pub lcd_status_mode: u8,
    pub gpu_cycles: GpuCycles
}

impl Memory {
    pub fn new() -> Memory {
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
            divider_register: 0,
            scan_line: 144,
            joypad_state: 0,
            cartridge_type: ROM_ONLY,
            in_ram_banking_mode: false,
            external_ram_enabled: false,
            irq48_signal: 0,
            screen_disabled: false,
            lcd_status_mode: 0,
            gpu_cycles: GpuCycles::new()
        }
    }

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
        println!("{:02X}", cartridge_type);
        println!("{}", num_rom_banks);
        println!("{}", num_ram_banks);
        let mut rom_banks: Vec<[u8; 0x4000]> = vec![[0; 0x4000]; num_rom_banks];
        for (i, bank) in rom_banks.iter_mut().enumerate() {
            let start = i * 0x4000;
            println!("copying rom from {:04X} to {:04X}", start, start + 0x4000);
            bank.copy_from_slice(&rom[start..start + 0x4000]);
        }

        let eram_banks: Vec<[u8; 0x2000]> = vec![[0; 0x2000]; num_ram_banks];
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
            divider_register: 0,
            scan_line: 0,
            joypad_state: 0,
            cartridge_type,
            in_ram_banking_mode: false,
            external_ram_enabled: false,
            irq48_signal: 0,
            screen_disabled: false,
            lcd_status_mode: 0,
            gpu_cycles: GpuCycles::new()
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
            0xA000..=0xBFFF => self.eram_banks[self.selected_eram_bank as usize][index - 0xA000],
            0xC000..=0xCFFF => self.wram_banks[0][index - 0xC000],
            0xD000..=0xDFFF => self.wram_banks[self.selected_wram_bank as usize][index - 0xD000],
            0xE000..=0xFDFF => self.wram_banks[0][index - 0xE000],
            0xFE00..=0xFE9F => self.oam[index - 0xFE00],
            0xFF00..=0xFFFF => match index {
                0xFF00 => self.get_joypad_state(),
                0xFF0F => self.high_ram[index - 0xFF00] | 0xE0,
                0xFF04 => self.divider_register,
                0xFF41 => self.high_ram[index - 0xFF00] | 0b01000_0000,
                0xFF44 => if !self.screen_disabled {
                    self.scan_line
                } else {
                    0x00
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
        let joypad_control = self.high_ram[0x100];
        joypad_control & (1 << 5) == 0
    }

    pub fn are_direction_keys_enabled(&self) -> bool {
        let joypad_control = self.high_ram[0x100];
        joypad_control & (1 << 4) == 0
    }

    fn cartridge_has_mbc1(&self) -> bool {
        self.cartridge_type == ROM_MBC1
            || self.cartridge_type == ROM_MBC1_RAM
            || self.cartridge_type == ROM_MBC1_RAM_BATT
    }

    pub fn write_byte(&mut self, index: u16, value: u8) {
        let index = index as usize;

        match index {
            0x6000..=0x7FFF if self.cartridge_has_mbc1() => {
                self.in_ram_banking_mode = value & 0x01 == 0x01
            }
            0x2000..=0x3FFF if self.cartridge_has_mbc1() => {
                let new_rom_bank = if 0b0001_1111 & value == 0 {
                    1
                } else {
                    0b0001_1111 & value
                };
                self.selected_rom_bank |= new_rom_bank;
            }
            0x4000..=0x5FFF if self.cartridge_has_mbc1() => {
                if self.in_ram_banking_mode {
                    if self.external_ram_enabled {
                        self.selected_eram_bank = 0b0000_0011 & value;
                    }
                } else {
                    self.selected_rom_bank |= (0b0000_0011 & value) << 5;
                }
            }
            0x0000..=0x1FFF if self.cartridge_has_mbc1() => {
                self.external_ram_enabled = value & 0b0000_1010 == 0b0000_1010
            }
            0x8000..=0x9FFF => {
                self.vram_banks[self.selected_vram_bank as usize][index - 0x8000] = value
            }
            0xA000..=0xBFFF => {
                self.eram_banks[self.selected_eram_bank as usize][index - 0xA000] = value
            }
            0xC000..=0xCFFF => self.wram_banks[0][index - 0xC000] = value,
            0xD000..=0xDFFF => {
                self.wram_banks[self.selected_wram_bank as usize][index - 0xD000] = value
            }
            0xFE00..=0xFE9F => self.oam[index - 0xFE00] = value,
            0xFF00..=0xFFFF => match index {
                0xFF04 => self.divider_register = 0,
                0xFF0F => self.high_ram[index - 0xFF00] = value & 0x1F,
                0xFF40 => {
                    let current_lcdc = LcdControlFlag::from_bits(self.read_byte(0xFF40)).unwrap();
                    let new_lcdc = LcdControlFlag::from_bits(value).unwrap();
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
                    self.high_ram[index - 0xFF00] = new_stat;
                    let lcd_control = LcdControlFlag::from_bits(self.read_byte(0xFF40)).unwrap();
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
                    println!("updating LYC");
                    let current_lyc = self.read_byte(0xFF45);
                    if current_lyc != value {
                        self.high_ram[index - 0xFF00] = value;
                        let lcd_control =
                            LcdControlFlag::from_bits(self.read_byte(0xFF40)).unwrap();
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
                _ => {
                    if index == 0xFF02 && value == 0x81 {
                        print!("{}", self.read_byte(0xFF01) as char);
                    }
                    self.high_ram[index - 0xFF00] = value
                }
            },
            _ => {}
        };
    }

    fn do_dma_transfer(&mut self, data: u8) {
        let address = 0x100u16 * data as u16;
        for i in 0..0xA0 {
            let value = self.read_byte(address + i);
            self.write_byte(0xFE00 + i, value);
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

            // println!("comparing LYC: {} to LY: {}", lyc, self.scan_line);
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
        // TODO may need to do some memeory modification here
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
