mod lcd_control_flag;
mod lcd_status_flag;

use self::lcd_control_flag::LcdControlFlag;
use self::lcd_status_flag::LcdStatusFlag;
use mmu::Memory;
use mmu::interrupt::Interrupt;

const CONTROL_REGISTER_INDEX: u16 = 0xFF40;
const LCD_INDEX: u16 = 0xFF41;
const SCROLL_Y_INDEX: u16 = 0xFF42;
const SCROLL_X_INDEX: u16 = 0xFF43;
const BACKGROUND_PALETTE_INDEX: u16 = 0xFF47;
const LYC_INDEX: u16 = 0xFF45;

const WHITE: u8 = 0b11111111;
const LIGHT_GRAY: u8 = 0b01001010;
const DARK_GRAY: u8 = 0b00100101;
const BLACK: u8 = 0b00000000;

pub struct GPU {
    pub pixels: [u8; 144 * 160],
    cycles: i32,
}

impl GPU {
    pub fn new() -> GPU {
        GPU {
            pixels: [0; 144 * 160],
            cycles: 456,
        }
    }

    pub fn step(&mut self, steps: i32, memory: &mut Memory) {
        let control_register = LcdControlFlag::from_bits(
            memory.read_byte(CONTROL_REGISTER_INDEX)).unwrap();

        self.set_lcd_status(memory);

        if control_register.contains(LcdControlFlag::DISPLAY) {
            self.cycles -= steps;

            if self.cycles <= 0 {
                memory.scan_line += 1;

                self.cycles = 456;

                if memory.scan_line == 144 {
                    memory.request_interrupt(Interrupt::Vblank);
                } else if memory.scan_line == 153 {
                    memory.scan_line = 0;
                } else if memory.scan_line < 144 {
                    self.render_scan(memory);
                }
            }
        }
    }

    fn set_lcd_status(&mut self, memory: &mut Memory) {
        let mut lcd_status = LcdStatusFlag::from_bits(
            memory.read_byte(LCD_INDEX)).unwrap();
        let control_register = LcdControlFlag::from_bits(
            memory.read_byte(CONTROL_REGISTER_INDEX)).unwrap();

        if control_register.contains(LcdControlFlag::DISPLAY) {
            let current_mode = lcd_status.bits() & 3;

            let request_interrupt = if memory.scan_line >= 144 {
                lcd_status.insert(LcdStatusFlag::MODE_LOW);
                lcd_status.remove(LcdStatusFlag::MODE_HIGH);
                lcd_status.contains(LcdStatusFlag::MODE_0_INTERRUPT)
            } else {
                let mode_2_bounds = 456 - 80;
                let mode_3_bounds = mode_2_bounds - 172;

                if self.cycles >= mode_2_bounds {
                    lcd_status.insert(LcdStatusFlag::MODE_HIGH);
                    lcd_status.remove(LcdStatusFlag::MODE_LOW);
                    lcd_status.contains(LcdStatusFlag::MODE_1_INTERRUPT)
                } else if self.cycles >= mode_3_bounds {
                    lcd_status.insert(LcdStatusFlag::MODE_HIGH | LcdStatusFlag::MODE_LOW);
                    false
                } else {
                    lcd_status.remove(LcdStatusFlag::MODE_HIGH | LcdStatusFlag::MODE_LOW);
                    lcd_status.contains(LcdStatusFlag::MODE_0_INTERRUPT)
                }
            };
            let mode = lcd_status.bits() & 3;
            if request_interrupt && (current_mode != mode) {
                memory.request_interrupt(Interrupt::Lcd);
            }
            let lyc = memory.read_byte(LYC_INDEX);
            if memory.scan_line == lyc {
                lcd_status.insert(LcdStatusFlag::COINCIDENCE);
                if lcd_status.contains(LcdStatusFlag::LY_COINCIDENCE) {
                    memory.request_interrupt(Interrupt::Lcd);
                }
            } else {
                lcd_status.remove(LcdStatusFlag::COINCIDENCE);
            }
        } else {
            self.cycles = 456;
            memory.scan_line = 0;
            lcd_status.remove(
                LcdStatusFlag::MODE_0_INTERRUPT |
                    LcdStatusFlag::MODE_1_INTERRUPT |
                    LcdStatusFlag::MODE_2_INTERRUPT |
                    LcdStatusFlag::LY_COINCIDENCE
            );
            lcd_status.insert(LcdStatusFlag::MODE_LOW);
        }

        memory.write_byte(LCD_INDEX, lcd_status.bits());
    }

    fn get_palette(&self, memory: &Memory) -> [u8; 4] {
        let order = memory.read_byte(BACKGROUND_PALETTE_INDEX);
        let mut palette: [u8; 4] = [0; 4];

        // iterate through each pair of two bits in the byte
        for i in 0..4 {
            match (order >> (i * 2)) & 0b11 {
                0b00 => palette[i] = WHITE,
                0b01 => palette[i] = LIGHT_GRAY,
                0b10 => palette[i] = DARK_GRAY,
                0b11 => palette[i] = BLACK,
                _ => {}
            }
        }

        palette
    }

    fn render_scan(&mut self, memory: &Memory) {
        let flag = memory.read_byte(CONTROL_REGISTER_INDEX);
        let scroll_y = memory.read_byte(SCROLL_Y_INDEX);
        let scroll_x = memory.read_byte(SCROLL_X_INDEX);
        let palette = self.get_palette(memory);

        let flag = LcdControlFlag::from_bits(flag).unwrap();

        if flag.contains(LcdControlFlag::BACKGROUND) {
            let line_offset = (memory.scan_line + scroll_y) as usize;

            let map_y = line_offset / 8;

            for x in 0..160 {
                let x_offset = (x + scroll_x) as usize;
                let map_x = x_offset / 8;

                let tile_id = if flag.contains(LcdControlFlag::BACKGROUND_TILE_MAP) {
                    memory.read_byte((0x9C00 + (32 * map_y + map_x)) as u16)
                } else {
                    memory.read_byte((0x9800 + (32 * map_y + map_x)) as u16)
                };

                let tile = if flag.contains(LcdControlFlag::BACKGROUND_TILE_SET) {
                    memory.get_tile_from_set1(tile_id)
                } else {
                    memory.get_tile_from_set0(tile_id as i8)
                };

                let row_num = line_offset % 8 * 2;
                let column_num = 7 - x_offset % 8;

                let high = tile[row_num];
                let low = tile[(row_num + 1)];

                let high_color = ((high & (1 << column_num) != 0) as u8) << 1;
                let low_color = (low & (1 << column_num) != 0) as u8;
                let color = palette[(high_color + low_color) as usize];

                self.pixels[(160 * (143 - memory.scan_line as usize) + x as usize)] = color;
            }
        }

        if flag.contains(LcdControlFlag::SPRITES) {}
    }

//    fn render_tiles(&mut self) {
//
//    }
//
//    fn render_spries(&mut self) {
//
//    }
}
