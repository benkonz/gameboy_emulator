use gameboy_core::{CGBColor, Color, PixelMapper};

pub struct Screen {
    pub pixels: Vec<u8>,
}

impl Screen {
    pub fn new() -> Screen {
        Screen {
            pixels: vec![0; 144 * 160 * 4],
        }
    }

    pub fn get_frame_buffer(&self) -> &Vec<u8> {
        &self.pixels
    }
}

impl PixelMapper for Screen {
    fn map_pixel(&mut self, pixel: usize, color: Color) {
        let color_bytes: [u8; 4] = match color {
            Color::White => [255, 255, 255, 255],
            Color::LightGray => [178, 178, 178, 255],
            Color::DarkGray => [102, 102, 102, 255],
            Color::Black => [0, 0, 0, 255],
        };

        for (i, byte) in color_bytes.iter().enumerate() {
            self.pixels[pixel * 4 + i] = *byte;
        }
    }

    fn cgb_map_pixel(&mut self, pixel: usize, color: CGBColor) {
        let color_bytes = [color.red, color.green, color.blue, 255];

        for (i, byte) in color_bytes.iter().enumerate() {
            self.pixels[pixel * 4 + i] = *byte;
        }
    }
}
