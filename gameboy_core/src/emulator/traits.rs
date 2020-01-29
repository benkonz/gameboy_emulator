use crate::gpu::cgb_color::CGBColor;
use crate::gpu::color::Color;

pub trait PixelMapper {
    fn map_pixel(&mut self, pixel: usize, color: Color);
    fn cgb_map_pixel(&mut self, pixel: usize, color: CGBColor);
}

pub trait RTC {
    // get the current unix timestamp in seconds
    fn get_current_time(&self) -> u64;
}
