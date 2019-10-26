use gameboy_core::{Color, PixelMapper};

pub struct Screen {
    frame_buffer: [u8; 144 * 160 * 3],
}

impl Screen {
    pub fn new() -> Screen {
        Screen {
            frame_buffer: [0; 144 * 160 * 3],
        }
    }

    pub fn get_frame_buffer(&self) -> &[u8; 144 * 160 * 3] {
        &self.frame_buffer
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
            self.frame_buffer[pixel * 3 + i] = *byte;
        }
    }

    fn get_pixel(&self, pixel: usize) -> Color {
        let offset = pixel * 3;
        match self.frame_buffer[offset..offset + 3] {
            [255, 255, 255] => Color::White,
            [178, 178, 178] => Color::LightGray,
            [102, 102, 102] => Color::DarkGray,
            [0, 0, 0] => Color::Black,
            _ => panic!("this should never happen"),
        }
    }
}