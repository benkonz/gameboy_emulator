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
use std::collections::VecDeque;

const SPRITES_START_INDEX: u16 = 0xFE00;
const LCD_CONTROL_INDEX: u16 = 0xFF40;
const LCD_STAT_INDEX: u16 = 0xFF41;
const SCROLL_Y_INDEX: u16 = 0xFF42;
const SCROLL_X_INDEX: u16 = 0xFF43;
const LYC_INDEX: u16 = 0xFF45;
const BACKGROUND_PALETTE_INDEX: u16 = 0xFF47;
const OBJECT_PALETTE_0_INDEX: u16 = 0xFF48;
const OBJECT_PALETTE_1_INDEX: u16 = 0xFF49;
const WINDOW_Y_INDEX: u16 = 0xFF4A;
const WINDOW_X_INDEX: u16 = 0xFF4B;
const CGB_SPRITE_PALETTE_INDEX: u16 = 0xFF6A;
const CGB_SPRITE_PALETTE_INDEX_INDEX: u16 = 0xFF6B;
const CGB_BACKGROUND_PALETTE_INDEX: u16 = 0xFF68;
const CGB_BACKGROUND_PALETTE_INDEX_INDEX: u16 = 0xFF69;

const HBLANK: u8 = 0b00;
const VBLANK: u8 = 0b01;
const OAM_SCAN: u8 = 0b10;
const LCD_TRANSFER: u8 = 0b11;

const GAMEBOY_WIDTH: i32 = 160;
const GAMEBOY_HEIGHT: i32 = 144;

pub struct GPU {
    is_cgb: bool,
    hide_frames: i32,
    scan_line_transferred: bool,
    vblank_line: i32,
    tile_cycles_counter: i32,
    scan_line: u8,
    irq48_signal: u8,
    screen_disabled: bool,
    lcd_status_mode: u8,
    cycles_counter: i32,
    aux_cycles_counter: i32,
    pixel_counter: i32,
    screen_enable_delay_cycles: i32,
    window_line: i32,
    vram_bank: i32,
    vram: Vec<u8>,
    oam: Vec<u8>,
    high_ram: Vec<u8>,
    cgb_background_palettes: [[CGBColor; 4]; 8],
    cgb_sprite_palettes: [[CGBColor; 4]; 8],
    background: Vec<u8>,
    interrupt_queue: VecDeque<Interrupt>,
}

impl GPU {
    pub fn new(is_cgb: bool) -> GPU {
        let vram = if is_cgb {
            vec![0x00; 0x2000 * 2]
        } else {
            vec![0x00; 0x2000]
        };

        let white = CGBColor {
            red: 0,
            green: 0,
            blue: 0,
        };

        GPU {
            is_cgb,
            hide_frames: 0,
            scan_line_transferred: false,
            vblank_line: 0,
            tile_cycles_counter: 0,
            scan_line: 144,
            irq48_signal: 0,
            lcd_status_mode: VBLANK,
            screen_disabled: false,
            cycles_counter: 0,
            aux_cycles_counter: 0,
            pixel_counter: 0,
            screen_enable_delay_cycles: 0,
            window_line: 0,
            vram_bank: 0,
            vram,
            oam: vec![0; 0x100],
            high_ram: vec![0; 0x100],
            cgb_background_palettes: [[white; 4]; 8],
            cgb_sprite_palettes: [[white; 4]; 8],
            background: vec![0; (GAMEBOY_WIDTH * GAMEBOY_HEIGHT) as usize],
            interrupt_queue: VecDeque::new(),
        }
    }

    // return value indicated whether a vblank has happened
    // true -> vblank has happened, render the frame buffer
    // false -> no vblank, continue stepping
    pub fn step<T: PixelMapper>(&mut self, cycles: i32, pixel_mapper: &mut T) -> bool {
        let mut vblank = false;
        self.cycles_counter += cycles;

        if !self.screen_disabled {
            match self.lcd_status_mode {
                HBLANK => vblank = self.step_hblank(),
                VBLANK => self.step_vblank(cycles),
                OAM_SCAN => self.step_oam_scan(),
                LCD_TRANSFER => self.step_lcd_transfer(cycles, pixel_mapper),
                _ => unreachable!(),
            }
        } else if self.screen_enable_delay_cycles > 0 {
            self.screen_enable_delay_cycles -= cycles;

            if self.screen_enable_delay_cycles <= 0 {
                self.hide_frames = 3;
                self.vblank_line = 0;
                self.tile_cycles_counter = 0;
                self.screen_disabled = false;
                self.lcd_status_mode = 0;
                self.scan_line = 0;
                self.irq48_signal = 0;
                self.screen_enable_delay_cycles = 0;
                self.cycles_counter = 0;
                self.aux_cycles_counter = 0;
                self.window_line = 0;
                self.pixel_counter = 0;

                let stat = self.load(LCD_STAT_INDEX);
                if bit_utils::is_set(stat, 5) {
                    self.request_interrupt(Interrupt::Lcd);
                    self.irq48_signal |= 0b0000_0100;
                }

                self.compare_ly_to_lyc();
            }
        } else if self.cycles_counter >= 70224 {
            self.cycles_counter -= 70224;
            vblank = true;
        }
        vblank
    }

    fn step_hblank(&mut self) -> bool {
        let mut vblank = false;
        if self.cycles_counter >= 204 {
            self.cycles_counter -= 204;
            self.lcd_status_mode = OAM_SCAN;

            self.scan_line += 1;
            self.compare_ly_to_lyc();

            // if self.is_cgb && memory.is_hdma_enabled() {
            //     let _cycles = memory.do_hdma();
            // self.cycles_counter += cycles;
            // }

            if self.scan_line == 144 {
                self.lcd_status_mode = VBLANK;
                self.vblank_line = 0;
                self.aux_cycles_counter = self.cycles_counter;

                self.request_interrupt(Interrupt::Vblank);

                self.irq48_signal &= 0x09;
                let stat = self.load(LCD_STAT_INDEX);
                if bit_utils::is_set(stat, 4) {
                    if !bit_utils::is_set(self.irq48_signal, 0)
                        && !bit_utils::is_set(self.irq48_signal, 3)
                    {
                        self.request_interrupt(Interrupt::Lcd);
                    }
                    self.irq48_signal |= 0b0000_0010;
                }
                self.irq48_signal &= 0x0E;

                if self.hide_frames > 0 {
                    self.hide_frames -= 1;
                } else {
                    vblank = true;
                }

                self.window_line = 0;
            } else {
                self.irq48_signal &= 0x09;
                let stat = self.load(LCD_STAT_INDEX);

                if bit_utils::is_set(stat, 5) {
                    if self.irq48_signal == 0 {
                        self.request_interrupt(Interrupt::Lcd);
                    }
                    self.irq48_signal |= 0b0000_0100;
                }
                self.irq48_signal &= 0x0E;
            }
            self.update_stat_register();
        }
        vblank
    }

    fn step_vblank(&mut self, cycles: i32) {
        self.aux_cycles_counter += cycles;

        if self.aux_cycles_counter >= 456 {
            self.aux_cycles_counter -= 456;
            self.vblank_line += 1;

            if self.vblank_line <= 9 {
                self.scan_line += 1;
                self.compare_ly_to_lyc();
            }
        }

        if self.cycles_counter >= 4104 && self.aux_cycles_counter >= 4 && self.scan_line == 153 {
            self.scan_line = 0;
            self.compare_ly_to_lyc();
        }

        if self.cycles_counter >= 4560 {
            self.cycles_counter -= 4560;
            self.lcd_status_mode = OAM_SCAN;
            self.update_stat_register();

            self.irq48_signal &= 0x0A;
            let stat = self.load(LCD_STAT_INDEX);
            if bit_utils::is_set(stat, 5) {
                if self.irq48_signal == 0 {
                    self.request_interrupt(Interrupt::Lcd);
                }
                self.irq48_signal |= 0b0000_0100;
            }
            self.irq48_signal &= 0x0D;
        }
    }

    fn step_oam_scan(&mut self) {
        if self.cycles_counter >= 80 {
            self.cycles_counter -= 80;
            self.lcd_status_mode = (self.lcd_status_mode & 0b1111_1100) | 0b11;
            self.irq48_signal &= 0x08;
            self.scan_line_transferred = false;
            self.update_stat_register();
        }
    }

    fn step_lcd_transfer<T: PixelMapper>(&mut self, cycles: i32, pixel_mapper: &mut T) {
        if self.pixel_counter < 160 {
            self.tile_cycles_counter += cycles;

            let lcdc = LcdControlFlag::from_bits_truncate(self.load(LCD_CONTROL_INDEX));
            if !self.screen_disabled && lcdc.contains(LcdControlFlag::DISPLAY) {
                while self.tile_cycles_counter >= 3 {
                    self.render_background(
                        i32::from(self.scan_line),
                        self.pixel_counter,
                        4,
                        pixel_mapper,
                    );
                    self.pixel_counter += 4;
                    self.tile_cycles_counter -= 3;

                    if self.pixel_counter >= 160 {
                        break;
                    }
                }
            }
        }

        if self.cycles_counter >= 160 && !self.scan_line_transferred {
            self.scan_line(i32::from(self.scan_line), pixel_mapper);
            self.scan_line_transferred = true;
        }

        if self.cycles_counter >= 172 {
            self.pixel_counter = 0;
            self.cycles_counter -= 172;
            self.lcd_status_mode = 0;
            self.tile_cycles_counter = 0;
            self.update_stat_register();

            self.irq48_signal &= 0x08;
            let stat = self.load(LCD_STAT_INDEX);
            if bit_utils::is_set(stat, 3) {
                if !bit_utils::is_set(self.irq48_signal, 3) {
                    self.request_interrupt(Interrupt::Lcd);
                }
                self.irq48_signal |= 0b0000_0001;
            }
        }
    }

    fn request_interrupt(&mut self, interrupt: Interrupt) {
        self.interrupt_queue.push_back(interrupt)
    }

    pub fn dequeue_interrupt(&mut self) -> Option<Interrupt> {
        self.interrupt_queue.pop_front()
    }

    fn update_stat_register(&mut self) {
        let stat = self.load(LCD_STAT_INDEX);
        self.store(LCD_STAT_INDEX, (stat & 0xFC) | (self.lcd_status_mode & 0x3));
    }

    fn scan_line<T: PixelMapper>(&mut self, line: i32, pixel_mapper: &mut T) {
        let lcd_control = LcdControlFlag::from_bits_truncate(self.load(LCD_CONTROL_INDEX));
        if !self.screen_disabled && lcd_control.contains(LcdControlFlag::DISPLAY) {
            self.render_window(line, pixel_mapper);
            self.render_sprites(line, pixel_mapper);
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
        line: i32,
        pixel: i32,
        count: i32,
        pixel_mapper: &mut T,
    ) {
        let offset_x_start = pixel % 8;
        let offset_x_end = offset_x_start + count;
        let screen_tile = pixel / 8;
        let lcd_control = LcdControlFlag::from_bits_truncate(self.load(LCD_CONTROL_INDEX));
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

            let scroll_x = self.load(SCROLL_X_INDEX);
            let scroll_y = self.load(SCROLL_Y_INDEX);
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
                    i32::from(self.read_cgb_lcd_ram(map_tile_addr, 0))
                } else {
                    (i32::from(self.read_cgb_lcd_ram(map_tile_addr, 0) as i8) + 128)
                };

                let cgb_tile_attrs = if self.is_cgb {
                    BgAttributes::from_bits_truncate(self.read_cgb_lcd_ram(map_tile_addr, 1))
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
                        self.read_cgb_lcd_ram(tile_address, 1),
                        self.read_cgb_lcd_ram(tile_address + 1, 1),
                    )
                } else {
                    (
                        self.read_cgb_lcd_ram(tile_address, 0),
                        self.read_cgb_lcd_ram(tile_address + 1, 0),
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
                    let color = self.cgb_background_palettes[cgb_tile_pal as usize][pixel as usize];
                    pixel_mapper.cgb_map_pixel(index, GPU::cgb_color_to_rgb_color(color));
                } else {
                    let palette = self.load(BACKGROUND_PALETTE_INDEX);
                    let color = GPU::gb_color_from_palette(palette, pixel);
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

    fn render_window<T: PixelMapper>(&mut self, line: i32, pixel_mapper: &mut T) {
        if self.window_line > 143 {
            return;
        }

        let lcd_control = LcdControlFlag::from_bits_truncate(self.load(LCD_CONTROL_INDEX));
        if !lcd_control.contains(LcdControlFlag::WINDOW) {
            return;
        }

        let wx = i32::from(self.load(WINDOW_X_INDEX)) - 7;
        if wx > 159 {
            return;
        }

        let wy = i32::from(self.load(WINDOW_Y_INDEX));
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

        let line_adjusted = self.window_line as i32;
        let y_32 = (line_adjusted / 8) * 32;
        let pixely = line_adjusted % 8;
        let pixely_2 = pixely * 2;
        let pixely_2_flip = (7 - pixely) * 2;
        let line_width = (GAMEBOY_HEIGHT - 1 - line) * GAMEBOY_WIDTH;

        for x in 0..32 {
            let tile = if lcd_control.contains(LcdControlFlag::BACKGROUND_TILE_SET) {
                i32::from(self.read_cgb_lcd_ram((map + y_32 + x) as u16, 0))
            } else {
                (i32::from(self.read_cgb_lcd_ram((map + y_32 + x) as u16, 0) as i8) + 128)
            };

            let cgb_tile_attrs = if self.is_cgb {
                BgAttributes::from_bits_truncate(self.read_cgb_lcd_ram((map + y_32 + x) as u16, 1))
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
                    self.read_cgb_lcd_ram(tile_address, 1),
                    self.read_cgb_lcd_ram(tile_address + 1, 1),
                )
            } else {
                (
                    self.read_cgb_lcd_ram(tile_address, 0),
                    self.read_cgb_lcd_ram(tile_address + 1, 0),
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
                    let color = self.cgb_background_palettes[cgb_tile_pal as usize][pixel as usize];
                    pixel_mapper.cgb_map_pixel(position, GPU::cgb_color_to_rgb_color(color));
                } else {
                    let palette = self.load(BACKGROUND_PALETTE_INDEX);
                    let color = GPU::gb_color_from_palette(palette, pixel);
                    pixel_mapper.map_pixel(position, color);
                }
            }
        }
        self.window_line += 1;
    }

    fn render_sprites<T: PixelMapper>(&mut self, line: i32, pixel_mapper: &mut T) {
        let lcd_control = LcdControlFlag::from_bits_truncate(self.load(LCD_CONTROL_INDEX));

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

            let sprite_y = i32::from(self.read_byte(SPRITES_START_INDEX + sprite_4)) - 16;
            if (sprite_y > line) || (sprite_y + sprite_height) <= line {
                continue;
            }

            let sprite_x = i32::from(self.read_byte(SPRITES_START_INDEX + sprite_4 + 1)) - 8;
            if (sprite_x < -7) || (sprite_x >= GAMEBOY_WIDTH) {
                continue;
            }

            let sprite_tile_16 = if lcd_control.contains(LcdControlFlag::SPRITES_SIZE) {
                i32::from(self.read_byte((SPRITES_START_INDEX + sprite_4 + 2) as u16) & 0xFE) * 16
            } else {
                i32::from(self.read_byte((SPRITES_START_INDEX + sprite_4 + 2) as u16)) * 16
            };

            let sprite_flags = SpriteAttributes::from_bits_truncate(
                self.read_byte((SPRITES_START_INDEX + sprite_4 + 3) as u16),
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
                    self.read_cgb_lcd_ram(tile_address, 1),
                    self.read_cgb_lcd_ram(tile_address + 1, 1),
                )
            } else {
                (
                    self.read_cgb_lcd_ram(tile_address, 0),
                    self.read_cgb_lcd_ram(tile_address + 1, 0),
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
                    let color = self.cgb_sprite_palettes[cgb_tile_pal as usize][pixel as usize];
                    pixel_mapper.cgb_map_pixel(position, GPU::cgb_color_to_rgb_color(color));
                } else {
                    let palette = if sprite_pallette {
                        self.load(OBJECT_PALETTE_1_INDEX)
                    } else {
                        self.load(OBJECT_PALETTE_0_INDEX)
                    };
                    let color = GPU::gb_color_from_palette(palette, pixel);
                    pixel_mapper.map_pixel(position, color);
                }
            }
        }
    }

    fn read_cgb_lcd_ram(&self, index: u16, bank: i32) -> u8 {
        let offset = 0x2000 * bank as usize;
        let address = index as usize - 0x8000 + offset;
        self.vram[address]
    }

    fn write_cgb_lcd_ram(&mut self, index: u16, value: u8, bank: i32) {
        let offset = 0x2000 * bank as usize;
        let address = index as usize - 0x8000 + offset;
        self.vram[address] = value;
    }

    pub fn read_byte(&self, index: u16) -> u8 {
        match index {
            0x8000..=0x9FFF => self.read_cgb_lcd_ram(index, self.vram_bank),
            0xFE00..=0xFEFF => self.oam[index as usize - 0xFE00],
            0xFF41 => self.load(index) | 0x80,
            0xFF44 => {
                if self.screen_disabled {
                    0x00
                } else {
                    self.scan_line
                }
            }
            0xFF40 | 0xFF42..=0xFF43 | 0xFF45..=0xFF4B => self.load(index),
            0xFF68 | 0xFF6A => {
                if self.is_cgb {
                    self.load(index) | 0x40
                } else {
                    0xC0
                }
            }
            0xFF69 | 0xFF6B => {
                if self.is_cgb {
                    self.load(index) | 0xF8
                } else {
                    0xFF
                }
            }
            0xFF4F => self.load(index) | 0xFE,
            _ => panic!("index out of GPU address range: {:04X}", index),
        }
    }

    pub fn write_byte(&mut self, index: u16, value: u8) {
        match index {
            0x8000..=0x9FFF => self.write_cgb_lcd_ram(index, value, self.vram_bank),
            0xFE00..=0xFEFF => self.oam[index as usize - 0xFE00] = value,
            0xFF40 => self.do_lcd_control_write(value),
            0xFF41 => self.do_lcd_status_write(value),
            0xFF44 => self.do_scanline_write(value),
            0xFF45 => self.do_lyc_write(value),
            0xFF42..=0xFF43 | 0xFF46..=0xFF4B => self.store(index, value),
            0xFF4F => {
                self.store(index, value);
                if self.is_cgb {
                    self.switch_cgb_vram_bank(value)
                }
            }
            0xFF68 => {
                self.store(index, value);
                if self.is_cgb {
                    self.update_color_palette(true, value)
                }
            }
            0xFF69 => {
                self.store(index, value);
                if self.is_cgb {
                    self.set_color_palette(true, value)
                }
            }
            0xFF6A => {
                self.store(index, value);
                if self.is_cgb {
                    self.update_color_palette(false, value)
                }
            }
            0xFF6B => {
                self.store(index, value);
                if self.is_cgb {
                    self.set_color_palette(false, value)
                }
            }
            _ => panic!("index out of GPU address range: {:04X}", index),
        }
    }

    fn do_lcd_control_write(&mut self, value: u8) {
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

    fn reset_window_line(&mut self) {
        let wy = self.load(WINDOW_Y_INDEX);

        if (self.window_line == 0) && (self.scan_line < 144) && (self.scan_line > wy) {
            self.window_line = 144;
        }
    }

    fn enable_screen(&mut self) {
        if self.screen_disabled {
            self.screen_enable_delay_cycles = 244;
        }
    }

    fn disable_screen(&mut self) {
        self.screen_disabled = true;
        let mut stat = self.load(LCD_STAT_INDEX);
        stat &= 0x7C;
        self.store(LCD_STAT_INDEX, stat);
        self.lcd_status_mode = 0;
        self.cycles_counter = 0;
        self.aux_cycles_counter = 0;
        self.scan_line = 0;
        self.irq48_signal = 0;
    }

    fn do_lcd_status_write(&mut self, value: u8) {
        let current_stat = self.load(LCD_STAT_INDEX) & 0x07;
        let new_stat = (value & 0x78) | (current_stat & 0x07);
        self.store(LCD_STAT_INDEX, new_stat);
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

    fn do_scanline_write(&mut self, value: u8) {
        let current_ly = self.scan_line;
        if bit_utils::is_set(current_ly, 7) && !bit_utils::is_set(value, 7) {
            self.disable_screen();
        }
    }

    fn do_lyc_write(&mut self, value: u8) {
        let current_lyc = self.load(LYC_INDEX);
        if current_lyc != value {
            self.store(LYC_INDEX, value);
            let lcd_control = LcdControlFlag::from_bits_truncate(self.load(LCD_CONTROL_INDEX));
            if lcd_control.contains(LcdControlFlag::DISPLAY) {
                self.compare_ly_to_lyc();
            }
        }
    }

    fn compare_ly_to_lyc(&mut self) {
        if !self.screen_disabled {
            let lyc = self.load(LYC_INDEX);
            let mut stat = self.load(LCD_STAT_INDEX);

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
            self.store(LCD_STAT_INDEX, stat);
        }
    }

    fn switch_cgb_vram_bank(&mut self, value: u8) {
        let value = value & 1;
        self.vram_bank = value as i32;
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
            self.store(CGB_BACKGROUND_PALETTE_INDEX_INDEX, final_value);
        } else {
            self.store(CGB_SPRITE_PALETTE_INDEX_INDEX, final_value);
        }
    }

    fn set_color_palette(&mut self, background: bool, value: u8) {
        let mut ps = if background {
            self.load(CGB_BACKGROUND_PALETTE_INDEX)
        } else {
            self.load(CGB_SPRITE_PALETTE_INDEX)
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
                self.store(CGB_BACKGROUND_PALETTE_INDEX, ps);
            } else {
                self.store(CGB_SPRITE_PALETTE_INDEX, ps);
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
            red: GPU::cgb_color_to_byte(color.red),
            green: GPU::cgb_color_to_byte(color.green),
            blue: GPU::cgb_color_to_byte(color.blue),
        }
    }

    // directly load a value from high ram
    // addresses are relative to the gameboy's physical memory, e.g. 0xFF00 = high_ram[0], 0xFFFF = high_ram[0xFF]
    fn load(&self, address: u16) -> u8 {
        self.high_ram[address as usize - 0xFF00]
    }

    // directly store a value to high ram
    // addresses are relative to the gameboy's physical memory, e.g. 0xFF00 = high_ram[0], 0xFFFF = high_ram[0xFF]
    fn store(&mut self, address: u16, value: u8) {
        self.high_ram[address as usize - 0xFF00] = value;
    }

    pub fn get_lcd_status_mode(&self) -> u8 {
        self.lcd_status_mode
    }
}
