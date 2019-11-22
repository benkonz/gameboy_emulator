use gpu::cgb_color::CGBColor;
use gpu::color::Color;

pub trait PixelMapper {
    fn map_pixel(&mut self, pixel: usize, color: Color);
    fn cgb_map_pixel(&mut self, pixel: usize, color: CGBColor);
}
