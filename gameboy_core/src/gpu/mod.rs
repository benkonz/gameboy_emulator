mod bg_attributes;
pub mod cgb_color;
pub mod color;
pub mod lcd_control_flag;
mod sprite_attributes;

use self::bg_attributes::BgAttributes;
use self::cgb_color::CGBColor;
use self::color::Color;
use self::lcd_control_flag::LcdControlFlag;
use self::sprite_attributes::SpriteAttributes;
use bit_utils;
use emulator::traits::PixelMapper;
use mmu::interrupt::Interrupt;
use mmu::Memory;

const SPRITES_START_INDEX: u16 = 0xFE00;
const LCD_CONTROL_INDEX: u16 = 0xFF40;
const LCD_INDEX: u16 = 0xFF41;
const SCROLL_Y_INDEX: u16 = 0xFF42;
const SCROLL_X_INDEX: u16 = 0xFF43;
const BACKGROUND_PALETTE_INDEX: u16 = 0xFF47;
const OBJECT_PALETTE_0_INDEX: u16 = 0xFF48;
const OBJECT_PALETTE_1_INDEX: u16 = 0xFF49;
const WINDOW_Y_INDEX: u16 = 0xFF4A;
const WINDOW_X_INDEX: u16 = 0xFF4B;

const HBLANK: u8 = 0b00;
const VBLANK: u8 = 0b01;
const OAM_SCAN: u8 = 0b10;
const LCD_TRANSFER: u8 = 0b11;

const GAMEBOY_WIDTH: i32 = 160;
const GAMEBOY_HEIGHT: i32 = 144;

pub struct GPU {
    is_cgb: bool,
    background: [u8; (GAMEBOY_HEIGHT * GAMEBOY_WIDTH) as usize],
    hide_frames: i32,
    scan_line_transferred: bool,
    vblank_line: i32,
    tile_cycles_counter: i32,
}

fn gb_color_from_palette(palette: u8, pixel: u8) -> Color {
    let color_bits = (palette >> (pixel * 2)) & 0x03;
    match color_bits {
        0b00 => Color::White,
        0b01 => Color::LightGray,
        0b10 => Color::DarkGray,
        0b11 => Color::Black,
        _ => unreachable!(),
    }
}

fn cgb_color_to_byte(color: u8) -> u8 {
    ((color as u16) * 0xFF / 0x1F) as u8
}

fn cgb_color_to_rgb_color(color: CGBColor) -> CGBColor {
    CGBColor {
        red: cgb_color_to_byte(color.red),
        green: cgb_color_to_byte(color.green),
        blue: cgb_color_to_byte(color.blue),
    }
}

impl GPU {
    pub fn new(is_cgb: bool) -> GPU {
        GPU {
            is_cgb,
            background: [0; (GAMEBOY_WIDTH * GAMEBOY_HEIGHT) as usize],
            hide_frames: 0,
            scan_line_transferred: false,
            vblank_line: 0,
            tile_cycles_counter: 0,
        }
    }

    // return value indicated whether a vblank has happened
    // true -> vblank has happened, render the frame buffer
    // false -> no vblank, continue stepping
    pub fn step<T: PixelMapper>(
        &mut self,
        cycles: i32,
        memory: &mut Memory,
        pixel_mapper: &mut T,
    ) -> bool {
        let mut vblank = false;
        memory.gpu_cycles.cycles_counter += cycles;

        if !memory.screen_disabled {
            match memory.lcd_status_mode {
                HBLANK => vblank = self.step_hblank(memory),
                VBLANK => self.step_vblank(memory, cycles),
                OAM_SCAN => self.step_oam_scan(memory),
                LCD_TRANSFER => self.step_lcd_transfer(memory, cycles, pixel_mapper),
                _ => unreachable!(),
            }
        } else if memory.gpu_cycles.screen_enable_delay_cycles > 0 {
            memory.gpu_cycles.screen_enable_delay_cycles -= cycles;

            if memory.gpu_cycles.screen_enable_delay_cycles <= 0 {
                self.hide_frames = 3;
                self.vblank_line = 0;
                self.tile_cycles_counter = 0;
                memory.screen_disabled = false;
                memory.lcd_status_mode = 0;
                memory.scan_line = 0;
                memory.irq48_signal = 0;
                memory.gpu_cycles.screen_enable_delay_cycles = 0;
                memory.gpu_cycles.cycles_counter = 0;
                memory.gpu_cycles.aux_cycles_counter = 0;
                memory.gpu_cycles.window_line = 0;
                memory.gpu_cycles.pixel_counter = 0;

                let stat = memory.get_lcd_status_from_memory();
                if bit_utils::is_set(stat, 5) {
                    memory.request_interrupt(Interrupt::Lcd);
                    memory.irq48_signal |= 0b0000_0100;
                }

                memory.compare_ly_to_lyc();
            }
        } else if memory.gpu_cycles.cycles_counter >= 70224 {
            memory.gpu_cycles.cycles_counter -= 70224;
            vblank = true;
        }
        vblank
    }

    fn step_hblank(&mut self, memory: &mut Memory) -> bool {
        let mut vblank = false;
        if memory.gpu_cycles.cycles_counter >= 204 {
            memory.gpu_cycles.cycles_counter -= 204;
            memory.lcd_status_mode = (memory.lcd_status_mode & 0b1111_1100) | OAM_SCAN;

            memory.scan_line += 1;
            memory.compare_ly_to_lyc();

            if self.is_cgb && memory.is_hdma_enabled() {
                let _cycles = memory.do_hdma();
                // memory.gpu_cycles.cycles_counter += cycles;
            }

            if memory.scan_line == 144 {
                memory.lcd_status_mode = (memory.lcd_status_mode & 0b1111_1100) | VBLANK;
                self.vblank_line = 0;
                memory.gpu_cycles.aux_cycles_counter = memory.gpu_cycles.cycles_counter;

                memory.request_interrupt(Interrupt::Vblank);

                memory.irq48_signal &= 0x09;
                let stat = memory.get_lcd_status_from_memory();
                if bit_utils::is_set(stat, 4) {
                    if !bit_utils::is_set(memory.irq48_signal, 0)
                        && !bit_utils::is_set(memory.irq48_signal, 3)
                    {
                        memory.request_interrupt(Interrupt::Lcd);
                    }
                    memory.irq48_signal |= 0b0000_0010;
                }
                memory.irq48_signal &= 0x0E;

                if self.hide_frames > 0 {
                    self.hide_frames -= 1;
                } else {
                    vblank = true;
                }

                memory.gpu_cycles.window_line = 0;
            } else {
                memory.irq48_signal &= 0x09;
                let stat = memory.get_lcd_status_from_memory();

                if bit_utils::is_set(stat, 5) {
                    if memory.irq48_signal == 0 {
                        memory.request_interrupt(Interrupt::Lcd);
                    }
                    memory.irq48_signal |= 0b0000_0100;
                }
                memory.irq48_signal &= 0x0E;
            }
            self.update_stat_register(memory);
        }
        vblank
    }

    fn step_vblank(&mut self, memory: &mut Memory, cycles: i32) {
        memory.gpu_cycles.aux_cycles_counter += cycles;

        if memory.gpu_cycles.aux_cycles_counter >= 456 {
            memory.gpu_cycles.aux_cycles_counter -= 456;
            self.vblank_line += 1;

            if self.vblank_line <= 9 {
                memory.scan_line += 1;
                memory.compare_ly_to_lyc();
            }
        }

        if memory.gpu_cycles.cycles_counter >= 4104
            && memory.gpu_cycles.aux_cycles_counter >= 4
            && memory.scan_line == 153
        {
            memory.scan_line = 0;
            memory.compare_ly_to_lyc();
        }

        if memory.gpu_cycles.cycles_counter >= 4560 {
            memory.gpu_cycles.cycles_counter -= 4560;
            memory.lcd_status_mode = (memory.lcd_status_mode & 0b1111_1100) | OAM_SCAN;
            self.update_stat_register(memory);

            memory.irq48_signal &= 0x0A;
            let stat = memory.get_lcd_status_from_memory();
            if bit_utils::is_set(stat, 5) {
                if memory.irq48_signal == 0 {
                    memory.request_interrupt(Interrupt::Lcd);
                }
                memory.irq48_signal |= 0b0000_0100;
            }
            memory.irq48_signal &= 0x0D;
        }
    }

    fn step_oam_scan(&mut self, memory: &mut Memory) {
        if memory.gpu_cycles.cycles_counter >= 80 {
            memory.gpu_cycles.cycles_counter -= 80;
            memory.lcd_status_mode = (memory.lcd_status_mode & 0b1111_1100) | 0b11;
            memory.irq48_signal &= 0x08;
            self.scan_line_transferred = false;
            self.update_stat_register(memory);
        }
    }

    fn step_lcd_transfer<T: PixelMapper>(
        &mut self,
        memory: &mut Memory,
        cycles: i32,
        pixel_mapper: &mut T,
    ) {
        if memory.gpu_cycles.pixel_counter < 160 {
            self.tile_cycles_counter += cycles;

            let lcdc = LcdControlFlag::from_bits_truncate(memory.read_byte(LCD_CONTROL_INDEX));
            if !memory.screen_disabled && lcdc.contains(LcdControlFlag::DISPLAY) {
                while self.tile_cycles_counter >= 3 {
                    self.render_background(
                        memory,
                        i32::from(memory.scan_line),
                        memory.gpu_cycles.pixel_counter,
                        4,
                        pixel_mapper,
                    );
                    memory.gpu_cycles.pixel_counter += 4;
                    self.tile_cycles_counter -= 3;

                    if memory.gpu_cycles.pixel_counter >= 160 {
                        break;
                    }
                }
            }
        }

        if memory.gpu_cycles.cycles_counter >= 160 && !self.scan_line_transferred {
            self.scan_line(memory, i32::from(memory.scan_line), pixel_mapper);
            self.scan_line_transferred = true;
        }

        if memory.gpu_cycles.cycles_counter >= 172 {
            memory.gpu_cycles.pixel_counter = 0;
            memory.gpu_cycles.cycles_counter -= 172;
            memory.lcd_status_mode = 0;
            self.tile_cycles_counter = 0;
            self.update_stat_register(memory);

            memory.irq48_signal &= 0x08;
            let stat = memory.get_lcd_status_from_memory();
            if bit_utils::is_set(stat, 3) {
                if !bit_utils::is_set(memory.irq48_signal, 3) {
                    memory.request_interrupt(Interrupt::Lcd);
                }
                memory.irq48_signal |= 0b0000_0001;
            }
        }
    }

    fn update_stat_register(&self, memory: &mut Memory) {
        let stat = memory.read_byte(LCD_INDEX);
        memory.set_lcd_status_from_memory((stat & 0xFC) | (memory.lcd_status_mode & 0x3));
    }

    fn scan_line<T: PixelMapper>(&mut self, memory: &mut Memory, line: i32, pixel_mapper: &mut T) {
        let lcd_control = LcdControlFlag::from_bits_truncate(memory.read_byte(LCD_CONTROL_INDEX));
        if !memory.screen_disabled && lcd_control.contains(LcdControlFlag::DISPLAY) {
            self.render_window(memory, line, pixel_mapper);
            self.render_sprites(memory, line, pixel_mapper);
        } else {
            let line_width = (GAMEBOY_HEIGHT - 1 - line) * GAMEBOY_WIDTH;
            for x in 0..GAMEBOY_WIDTH {
                let index = (line_width + x) as usize;
                if self.is_cgb {
                    let white = CGBColor {
                        red: 0,
                        green: 0,
                        blue: 0,
                    };
                    pixel_mapper.cgb_map_pixel(index, white);
                } else {
                    pixel_mapper.map_pixel(index, Color::White);
                }
            }
        }
    }

    fn render_background<T: PixelMapper>(
        &mut self,
        memory: &Memory,
        line: i32,
        pixel: i32,
        count: i32,
        pixel_mapper: &mut T,
    ) {
        let offset_x_start = pixel % 8;
        let offset_x_end = offset_x_start + count;
        let screen_tile = pixel / 8;
        let lcd_control = LcdControlFlag::from_bits_truncate(memory.read_byte(LCD_CONTROL_INDEX));
        let line_width = (GAMEBOY_HEIGHT - 1 - line) * GAMEBOY_WIDTH;

        if self.is_cgb || lcd_control.contains(LcdControlFlag::DISPLAY) {
            let tile_start_addr = if lcd_control.contains(LcdControlFlag::BACKGROUND_TILE_SET) {
                0x8000
            } else {
                0x8800
            };

            let map_start_addr = if lcd_control.contains(LcdControlFlag::BACKGROUND_TILE_MAP) {
                0x9C00
            } else {
                0x9800
            };

            let scroll_x = memory.read_byte(SCROLL_X_INDEX);
            let scroll_y = memory.read_byte(SCROLL_Y_INDEX);
            let line_scrolled = scroll_y.wrapping_add(line as u8);
            let line_scrolled_32 = (i32::from(line_scrolled) / 8) * 32;
            let tile_pixel_y = i32::from(line_scrolled % 8);
            let tile_pixel_y_2 = tile_pixel_y * 2;
            let tile_pixel_y_flip_2 = (7 - tile_pixel_y) * 2;

            for offset_x in offset_x_start..offset_x_end {
                let screen_pixel_x = (screen_tile * 8) + offset_x;
                let map_pixel_x = scroll_x.wrapping_add(screen_pixel_x as u8);
                let map_tile_x = i32::from(map_pixel_x / 8);
                let map_tile_offset_x = map_pixel_x % 8;
                let map_tile_addr = (map_start_addr + line_scrolled_32 + map_tile_x) as u16;

                let map_tile = if lcd_control.contains(LcdControlFlag::BACKGROUND_TILE_SET) {
                    i32::from(memory.read_cgb_lcd_ram(map_tile_addr, 0))
                } else {
                    (i32::from(memory.read_cgb_lcd_ram(map_tile_addr, 0) as i8) + 128)
                };

                let cgb_tile_attrs = if self.is_cgb {
                    BgAttributes::from_bits_truncate(memory.read_cgb_lcd_ram(map_tile_addr, 1))
                } else {
                    BgAttributes::empty()
                };
                let cgb_tile_pal = if self.is_cgb {
                    cgb_tile_attrs.bits() & 0b111
                } else {
                    0
                };
                let cgb_tile_bank = if self.is_cgb {
                    cgb_tile_attrs.contains(BgAttributes::VRAM_BANK)
                } else {
                    false
                };
                let cgb_tile_xflip = if self.is_cgb {
                    cgb_tile_attrs.contains(BgAttributes::XFLIP)
                } else {
                    false
                };
                let cgb_tile_yflip = if self.is_cgb {
                    cgb_tile_attrs.contains(BgAttributes::YFLIP)
                } else {
                    false
                };
                let cgb_tile_priority = if self.is_cgb {
                    cgb_tile_attrs.contains(BgAttributes::BG_PRIORITY)
                } else {
                    false
                };
                let map_tile_16 = map_tile * 16;
                let final_pixely_2 = if self.is_cgb && cgb_tile_yflip {
                    tile_pixel_y_flip_2
                } else {
                    tile_pixel_y_2
                };
                let tile_address = (tile_start_addr + map_tile_16 + final_pixely_2) as u16;

                let (byte1, byte2) = if self.is_cgb && cgb_tile_bank {
                    (
                        memory.read_cgb_lcd_ram(tile_address, 1),
                        memory.read_cgb_lcd_ram(tile_address + 1, 1),
                    )
                } else {
                    (
                        memory.read_cgb_lcd_ram(tile_address, 0),
                        memory.read_cgb_lcd_ram(tile_address + 1, 0),
                    )
                };
                let mut pixel_x_in_tile = i32::from(map_tile_offset_x);
                if self.is_cgb && cgb_tile_xflip {
                    pixel_x_in_tile = 7 - pixel_x_in_tile;
                }
                let pixel_x_in_tile_bit = 1 << (7 - pixel_x_in_tile) as u8;

                let mut pixel = 0;
                if byte1 & pixel_x_in_tile_bit != 0 {
                    pixel |= 1;
                }
                if byte2 & pixel_x_in_tile_bit != 0 {
                    pixel |= 2;
                }

                let index = (line_width + screen_pixel_x) as usize;
                self.background[index] = pixel & 0x03;

                if self.is_cgb {
                    if cgb_tile_priority && (pixel != 0) {
                        self.background[index] |= 0b0100;
                    }
                    let color =
                        memory.cgb_background_palettes[cgb_tile_pal as usize][pixel as usize];
                    pixel_mapper.cgb_map_pixel(index, cgb_color_to_rgb_color(color));
                } else {
                    let palette = memory.read_byte(BACKGROUND_PALETTE_INDEX);
                    let color = gb_color_from_palette(palette, pixel);
                    pixel_mapper.map_pixel(index, color);
                }
            }
        } else {
            for x in 0..GAMEBOY_WIDTH {
                let index = (line_width + x) as usize;
                self.background[index] = 0;

                if self.is_cgb {
                    let white = CGBColor {
                        red: 0,
                        green: 0,
                        blue: 0,
                    };
                    pixel_mapper.cgb_map_pixel(index, white);
                } else {
                    pixel_mapper.map_pixel(index, Color::White);
                }
            }
        }
    }

    fn render_window<T: PixelMapper>(
        &mut self,
        memory: &mut Memory,
        line: i32,
        pixel_mapper: &mut T,
    ) {
        if memory.gpu_cycles.window_line > 143 {
            return;
        }

        let lcd_control = LcdControlFlag::from_bits_truncate(memory.read_byte(LCD_CONTROL_INDEX));
        if !lcd_control.contains(LcdControlFlag::WINDOW) {
            return;
        }

        let wx = i32::from(memory.read_byte(WINDOW_X_INDEX)) - 7;
        if wx > 159 {
            return;
        }

        let wy = i32::from(memory.read_byte(WINDOW_Y_INDEX));
        if (wy > 143) || (wy > line) {
            return;
        }

        let tiles = if lcd_control.contains(LcdControlFlag::BACKGROUND_TILE_SET) {
            0x8000
        } else {
            0x8800
        };

        let map = if lcd_control.contains(LcdControlFlag::WINDOW_TILE_MAP) {
            0x9C00
        } else {
            0x9800
        };

        let line_adjusted = memory.gpu_cycles.window_line as i32;
        let y_32 = (line_adjusted / 8) * 32;
        let pixely = line_adjusted % 8;
        let pixely_2 = pixely * 2;
        let pixely_2_flip = (7 - pixely) * 2;
        let line_width = (GAMEBOY_HEIGHT - 1 - line) * GAMEBOY_WIDTH;

        for x in 0..32 {
            let tile = if lcd_control.contains(LcdControlFlag::BACKGROUND_TILE_SET) {
                i32::from(memory.read_cgb_lcd_ram((map + y_32 + x) as u16, 0))
            } else {
                (i32::from(memory.read_cgb_lcd_ram((map + y_32 + x) as u16, 0) as i8) + 128)
            };

            let cgb_tile_attrs = if self.is_cgb {
                BgAttributes::from_bits_truncate(
                    memory.read_cgb_lcd_ram((map + y_32 + x) as u16, 1),
                )
            } else {
                BgAttributes::empty()
            };
            let cgb_tile_pal = if self.is_cgb {
                cgb_tile_attrs.bits() & 0b111
            } else {
                0
            };
            let cgb_tile_bank = if self.is_cgb {
                cgb_tile_attrs.contains(BgAttributes::VRAM_BANK)
            } else {
                false
            };
            let cgb_tile_xflip = if self.is_cgb {
                cgb_tile_attrs.contains(BgAttributes::XFLIP)
            } else {
                false
            };
            let cgb_tile_yflip = if self.is_cgb {
                cgb_tile_attrs.contains(BgAttributes::YFLIP)
            } else {
                false
            };
            let cgb_tile_priority = if self.is_cgb {
                cgb_tile_attrs.contains(BgAttributes::BG_PRIORITY)
            } else {
                false
            };
            let map_offset_x = x * 8;
            let tile_16 = tile * 16;
            let final_pixely_2 = if self.is_cgb && cgb_tile_yflip {
                pixely_2_flip
            } else {
                pixely_2
            };
            let tile_address = (tiles + tile_16 + final_pixely_2) as u16;
            let (byte1, byte2) = if self.is_cgb && cgb_tile_bank {
                (
                    memory.read_cgb_lcd_ram(tile_address, 1),
                    memory.read_cgb_lcd_ram(tile_address + 1, 1),
                )
            } else {
                (
                    memory.read_cgb_lcd_ram(tile_address, 0),
                    memory.read_cgb_lcd_ram(tile_address + 1, 0),
                )
            };

            for pixelx in 0..8 {
                let buffer_x = map_offset_x + pixelx + wx;
                if buffer_x < 0 || buffer_x >= GAMEBOY_WIDTH {
                    continue;
                }

                let pixelx_pos = if self.is_cgb && cgb_tile_xflip {
                    7 - pixelx as u8
                } else {
                    pixelx as u8
                };

                let mut pixel = 0;
                if (byte1 & (0x1 << (7 - pixelx_pos))) != 0 {
                    pixel |= 1;
                }
                if (byte2 & (0x1 << (7 - pixelx_pos))) != 0 {
                    pixel |= 2;
                };

                let position = (line_width + buffer_x) as usize;
                self.background[position] = pixel & 0x03;

                if self.is_cgb {
                    if cgb_tile_priority && pixel != 0 {
                        self.background[position] |= 0b0100;
                    }
                    let color =
                        memory.cgb_background_palettes[cgb_tile_pal as usize][pixel as usize];
                    pixel_mapper.cgb_map_pixel(position, cgb_color_to_rgb_color(color));
                } else {
                    let palette = memory.read_byte(BACKGROUND_PALETTE_INDEX);
                    let color = gb_color_from_palette(palette, pixel);
                    pixel_mapper.map_pixel(position, color);
                }
            }
        }
        memory.gpu_cycles.window_line += 1;
    }

    fn render_sprites<T: PixelMapper>(&mut self, memory: &Memory, line: i32, pixel_mapper: &mut T) {
        let lcd_control = LcdControlFlag::from_bits_truncate(memory.read_byte(LCD_CONTROL_INDEX));

        if !lcd_control.contains(LcdControlFlag::SPRITES) {
            return;
        }

        let sprite_height = if lcd_control.contains(LcdControlFlag::SPRITES_SIZE) {
            16
        } else {
            8
        };

        let line_width = (GAMEBOY_HEIGHT - 1 - line) * GAMEBOY_WIDTH;

        for sprite in (0..40).rev() {
            let sprite_4 = sprite * 4;

            let sprite_y = i32::from(memory.read_byte(SPRITES_START_INDEX + sprite_4)) - 16;
            if (sprite_y > line) || (sprite_y + sprite_height) <= line {
                continue;
            }

            let sprite_x = i32::from(memory.read_byte(SPRITES_START_INDEX + sprite_4 + 1)) - 8;
            if (sprite_x < -7) || (sprite_x >= GAMEBOY_WIDTH) {
                continue;
            }

            let sprite_tile_16 = if lcd_control.contains(LcdControlFlag::SPRITES_SIZE) {
                i32::from(memory.read_byte((SPRITES_START_INDEX + sprite_4 + 2) as u16) & 0xFE) * 16
            } else {
                i32::from(memory.read_byte((SPRITES_START_INDEX + sprite_4 + 2) as u16)) * 16
            };

            let sprite_flags = SpriteAttributes::from_bits_truncate(
                memory.read_byte((SPRITES_START_INDEX + sprite_4 + 3) as u16),
            );

            let sprite_pallette = sprite_flags.contains(SpriteAttributes::PALETTE);
            let xflip = sprite_flags.contains(SpriteAttributes::X_FLIP);
            let yflip = sprite_flags.contains(SpriteAttributes::Y_FLIP);
            let behind_bg = sprite_flags.contains(SpriteAttributes::BACKGROUND_PRIORITY);
            let cgb_tile_bank = sprite_flags.contains(SpriteAttributes::VRAM_BANK);
            let cgb_tile_pal = sprite_flags.bits() & 0x07;
            let tiles = 0x8000;

            let pixel_y = if yflip {
                let height = if lcd_control.contains(LcdControlFlag::SPRITES_SIZE) {
                    15
                } else {
                    7
                };
                height - (line - sprite_y)
            } else {
                line - sprite_y
            };

            let (pixel_y_2, offset) =
                if lcd_control.contains(LcdControlFlag::SPRITES_SIZE) && pixel_y >= 8 {
                    ((pixel_y - 8) * 2, 16)
                } else {
                    (pixel_y * 2, 0)
                };

            let tile_address = (tiles + sprite_tile_16 + pixel_y_2 + offset) as u16;

            let (byte1, byte2) = if self.is_cgb && cgb_tile_bank {
                (
                    memory.read_cgb_lcd_ram(tile_address, 1),
                    memory.read_cgb_lcd_ram(tile_address + 1, 1),
                )
            } else {
                (
                    memory.read_cgb_lcd_ram(tile_address, 0),
                    memory.read_cgb_lcd_ram(tile_address + 1, 0),
                )
            };

            for pixelx in 0..8 {
                let mut pixel = 0;

                if xflip {
                    if byte1 & (0x01 << pixelx) != 0 {
                        pixel |= 1;
                    }
                } else if byte1 & (0x01 << (7 - pixelx)) != 0 {
                    pixel |= 1;
                }

                if xflip {
                    if byte2 & (0x01 << pixelx) != 0 {
                        pixel |= 2;
                    }
                } else if byte2 & (0x01 << (7 - pixelx)) != 0 {
                    pixel |= 2;
                }
                if pixel == 0 {
                    continue;
                }

                let buffer_x = sprite_x + pixelx as i32;
                if buffer_x < 0 || buffer_x >= GAMEBOY_WIDTH {
                    continue;
                }

                let position = (line_width + buffer_x) as usize;
                let background_color = self.background[position];

                if self.is_cgb && bit_utils::is_set(background_color as u8, 2) {
                    continue;
                }

                if behind_bg && (background_color & 0x03) != 0 {
                    continue;
                }

                if self.is_cgb {
                    let color = memory.cgb_sprite_palettes[cgb_tile_pal as usize][pixel as usize];
                    pixel_mapper.cgb_map_pixel(position, cgb_color_to_rgb_color(color));
                } else {
                    let palette = if sprite_pallette {
                        memory.read_byte(OBJECT_PALETTE_1_INDEX)
                    } else {
                        memory.read_byte(OBJECT_PALETTE_0_INDEX)
                    };
                    let color = gb_color_from_palette(palette, pixel);
                    pixel_mapper.map_pixel(position, color);
                }
            }
        }
    }
}
