pub mod interrupt;

use self::interrupt::Interrupt;

const BIOS: &[u8] = include_bytes!("bios.gb");
const INTERRUPT_ENABLE_INDEX: u16 = 0xFFFF;
const INTERRUPT_FLAGS_INDEX: u16 = 0xFF0F;

const ROM_ONLY: u8 = 0x0;
const ROM_MBC1: u8 = 0x1;
const ROM_MBC1_RAM: u8 = 0x2;
const ROM_MBC1_RAM_BATT: u8 = 0x3;
const ROM_MBC2: u8 = 0x5;
const ROM_MBC2_BATT: u8 = 0x6;
const ROM_RAM: u8 = 0x8;
const ROM_RAM_BATT: u8 = 0x9;
const ROM_MMM01: u8 = 0xB;
const ROM_MMM01_SRAM: u8 = 0xC;
const ROM_MMM01_SRAM_BATT: u8 = 0xD;
const ROM_MBC3_TIMER_BATT: u8 = 0xF;
const ROM_MBC3_TIMER_RAM_BATT: u8 = 0x10;
const ROM_MBC3: u8 = 0x11;
const ROM_MBC3_RAM: u8 = 0x12;
const ROM_MBC3_RAM_BATT: u8 = 0x13;
const ROM_MBC5: u8 = 0x19;
const ROM_MBC5_RAM: u8 = 0x1A;
const ROM_MBC5_RAM_BATT: u8 = 0x1B;
const ROM_MBC5_RUMBLE: u8 = 0x1C;
const ROM_MBC5_RUMBLE_SRAM: u8 = 0x1D;
const ROM_MBC5_RUMBLE_SRAM_BATT: u8 = 0x1E;
const MBC6: u8 = 0x20;
const MBC7_SENSOR_RUMBLE_RAM_BATT: u8 = 0x22;
const Pocket_Camera: u8 = 0xFC;
const Bandai_TAMA5: u8 = 0xFD;
const Hudson_HuC3: u8 = 0xFE;
const Hudson_HuC1_RAM_BATTERY: u8 = 0xFF;

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
    pub divider_register: u8,
    pub scan_line: u8,
    joypad_state: u8,
    cartridge_type: u8,
    in_ram_banking_mode: bool,
    external_ram_enabled: bool,
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
            scan_line: 0,
            joypad_state: 0,
            cartridge_type: ROM_ONLY,
            in_ram_banking_mode: false,
            external_ram_enabled: false,
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
                0xFF04 => self.divider_register,
                0xFF44 => self.scan_line,
                0xFF50 => self.disable_bios,
                _ => self.high_ram[index - 0xFE00],
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
                0xFF44 => self.scan_line = 0,
                0xFF46 => self.do_dma_transfer(value),
                0xFF50 => self.disable_bios = value,
                _ => {
                    if index == 0xFF02 && value == 0x81 {
                        print!("{}", self.read_byte(0xFF01) as char);
                    }
                    self.high_ram[index - 0xFE00] = value
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

    pub fn get_tile_from_set1(&self, tile_id: u8) -> &[u8] {
        let index = tile_id as usize * 0x10;
        &self.vram_banks[self.selected_vram_bank as usize][index..index + 0x10]
    }

    pub fn get_tile_from_set0(&self, tile_id: i8) -> &[u8] {
        let index = (0x1000 + tile_id as i32 * 0x10) as usize;
        &self.vram_banks[self.selected_vram_bank as usize][index..index + 0x10]
    }

    pub fn get_sprite_data(&self, sprite_id: u8) -> &[u8] {
        let start_index = (sprite_id * 4) as usize;
        &self.oam[start_index..start_index + 4]
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

    #[test]
    fn test_get_tile_from_map_0() {
        let mut memory = Memory::new();
        let tile = [
            0x7F, 0x00, 0x7F, 0x00, 0x7F, 0x00, 0x7F, 0x00, 0x7F, 0x00, 0x7F, 0x00, 0x7F, 0x00,
            0x7F, 0x00,
        ];

        for (i, row) in tile.iter().enumerate() {
            memory.write_byte((0x9000 + i) as u16, *row);
        }

        let actual = memory.get_tile_from_set0(0);

        for i in 0..0x10 {
            assert_eq!(tile[i], actual[i]);
        }
    }
}
