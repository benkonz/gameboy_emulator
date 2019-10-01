use gpu::color::Color;

pub trait PixelMapper {
    fn map_pixel(&mut self, x: u8, y: u8, color: Color);
    fn get_pixel(&self, x: u8, y: u8) -> Color;
}
