pub trait Renderable {
    fn render(pixels: &[u8]);
}

pub struct Screen {}

impl Renderable for Screen {
    fn render(pixels: &[u8]) {}
}

impl Default for Screen {
    fn default() -> Screen {
        Screen{}
    }
}