use gpu::color::Color;

pub trait PixelMapper {
    fn map_pixel(&mut self, pixel: usize, color: Color);
    fn get_pixel(&self, pixel: usize) -> Color;
}
