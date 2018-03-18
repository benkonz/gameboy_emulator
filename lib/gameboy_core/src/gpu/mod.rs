mod lcd_control_flag;
mod lcd_status_flag;
mod sprite_attributes;

use self::lcd_control_flag::LcdControlFlag;
use self::lcd_status_flag::LcdStatusFlag;
use self::sprite_attributes::SpriteAttributes;
use mmu::Memory;
use mmu::interrupt::Interrupt;

const SPRITES_START_INDEX: u16 = 0xFE00;
const LCD_CONTROL_INDEX: u16 = 0xFF40;
const LCD_INDEX: u16 = 0xFF41;
const SCROLL_Y_INDEX: u16 = 0xFF42;
const SCROLL_X_INDEX: u16 = 0xFF43;
const LYC_INDEX: u16 = 0xFF45;
const BACKGROUND_PALETTE_INDEX: u16 = 0xFF47;
const OBJECT_PALETTE_0_INDEX: u16 = 0xFF48;
const OBJECT_PALETTE_1_INDEX: u16 = 0xFF49;
const WINDOW_Y_INDEX: u16 = 0xFF4A;
const WINDOW_X_INDEX: u16 = 0xFF4B;

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
            memory.read_byte(LCD_CONTROL_INDEX)).unwrap();

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
            memory.read_byte(LCD_CONTROL_INDEX)).unwrap();

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

    fn get_palette(&self, order: u8) -> [u8; 4] {
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
        let flag = LcdControlFlag::from_bits(
            memory.read_byte(LCD_CONTROL_INDEX)).unwrap();

        if flag.contains(LcdControlFlag::BACKGROUND) {
            self.render_tiles(memory);
        }

        if flag.contains(LcdControlFlag::SPRITES) {
            self.render_sprites(memory);
        }
    }

    fn render_tiles(&mut self, memory: &Memory) {
        let scroll_y = memory.read_byte(SCROLL_Y_INDEX);
        let scroll_x = memory.read_byte(SCROLL_X_INDEX);
        let window_x = memory.read_byte(WINDOW_X_INDEX);
        let window_y = 7 - memory.read_byte(WINDOW_Y_INDEX);
        let order = memory.read_byte(BACKGROUND_PALETTE_INDEX);
        let scan_line = memory.scan_line;

        let lcd_control = LcdControlFlag::from_bits(
            memory.read_byte(LCD_CONTROL_INDEX)).unwrap();
        let palette = self.get_palette(order);

        let using_window = lcd_control.contains(LcdControlFlag::WINDOW) &&
            window_y <= memory.scan_line;

        let tile_offset = if (using_window &&
            lcd_control.contains(LcdControlFlag::WINDOW_TILE_MAP)) ||
            lcd_control.contains(LcdControlFlag::BACKGROUND_TILE_MAP) {
            0x9C00
        } else {
            0x9800
        };

        let line_offset = if using_window {
            (scan_line - window_y) as usize
        } else {
            (scan_line as usize + scroll_y as usize) as usize
        };

        let map_y = line_offset / 8;

        for x in 0..160 {
            let x_offset = if using_window && x >= window_x {
                (x - window_x) as usize
            } else {
                (x as usize + scroll_x as usize) as usize
            };
            let map_x = x_offset / 8;

            let tile_id = memory.read_byte((tile_offset + (32 * map_y + map_x)) as u16);

            let tile = if lcd_control.contains(LcdControlFlag::BACKGROUND_TILE_SET) {
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

            self.pixels[160 * (143 - scan_line as usize) + x as usize] = color;
        }
    }

    fn render_sprites(&mut self, memory: &Memory) {
        let scan_line = memory.scan_line;
        let lcd_control = LcdControlFlag::from_bits(
            memory.read_byte(LCD_CONTROL_INDEX)).unwrap();

        for sprite in 0..40 {
            let index = sprite * 4;
            let y_pos = memory.read_byte(SPRITES_START_INDEX + index) as i16 - 16;
            let x_pos = memory.read_byte(SPRITES_START_INDEX + index + 1) as i16 - 8;
            let tile_location = memory.read_byte(SPRITES_START_INDEX + index + 2);
            let attributes = SpriteAttributes::from_bits_truncate(
                memory.read_byte(SPRITES_START_INDEX + index + 3));


            let y_size = if lcd_control.contains(LcdControlFlag::SPRITES_SIZE) {
                16
            } else {
                8
            };

            if (scan_line as i16) >= y_pos && (scan_line as i16) < y_pos + 8 {
                let palette = if attributes.contains(SpriteAttributes::PALETTE) {
                    let order = memory.read_byte(OBJECT_PALETTE_1_INDEX);
                    self.get_palette(order)
                } else {
                    let order = memory.read_byte(OBJECT_PALETTE_0_INDEX);
                    self.get_palette(order)
                };

                let line = if attributes.contains(SpriteAttributes::Y_FLIP) {
                    (7 - (scan_line as i16 - y_pos)) * 2
                } else {
                    (scan_line as i16 - y_pos) * 2
                };

                let tile = memory.get_tile_from_set1(tile_location);

                let high = tile[line as usize];
                let low = tile[line as usize + 1];

                for tile_pixel in 0..8 {
                    let color_bit = if attributes.contains(SpriteAttributes::X_FLIP) {
                        tile_pixel
                    } else {
                        7 - tile_pixel
                    };

                    let pixel = tile_pixel + x_pos;

                    let high_color = (((high & (1 << color_bit)) != 0) as u8) << 1;
                    let low_color = ((low & (1 << color_bit)) != 0) as u8;
                    let color_index = high_color + low_color;

                    if pixel >= 0 && pixel < 160
                        && color_index != 0
                        && (attributes.contains(SpriteAttributes::BACKGROUND_PRIORITY)
                        || self.pixels[160 * (143 - scan_line as usize) + pixel as usize] != 0) {
                        let color = palette[color_index as usize];
                        self.pixels[160 * (143 - scan_line as usize) + pixel as usize] = color;
                    }
                }
            }
        }
    }
}
