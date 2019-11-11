use gameboy_core::emulator::traits::PixelMapper;
use gameboy_core::Color;

pub struct Screen {
    pub pixels: Vec<u8>,
}

impl Screen {
    pub fn new() -> Screen {
        Screen {
            pixels: vec![0; 144 * 160 * 3],
        }
    }

    pub fn get_frame_buffer(&self) -> &Vec<u8> {
        &self.pixels
    }
}

impl PixelMapper for Screen {
    fn map_pixel(&mut self, pixel: usize, color: Color) {
        let color_bytes: [u8; 3] = match color {
            Color::White => [255, 255, 255],
            Color::LightGray => [178, 178, 178],
            Color::DarkGray => [102, 102, 102],
            Color::Black => [0, 0, 0],
        };

        for (i, byte) in color_bytes.iter().enumerate() {
            self.pixels[pixel * 3 + i] = *byte;
        }
    }

    fn get_pixel(&self, pixel: usize) -> Color {
        let offset = pixel * 3;
        match self.pixels[offset..offset + 3] {
            [255, 255, 255] => Color::White,
            [178, 178, 178] => Color::LightGray,
            [102, 102, 102] => Color::DarkGray,
            [0, 0, 0] => Color::Black,
            _ => unreachable!(),
        }
    }
}
