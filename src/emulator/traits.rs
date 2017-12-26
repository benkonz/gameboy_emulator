use joypad::Joypad;

pub trait Io {
    fn draw(&self, pixels: &[u8; 144 * 160]);
    fn update_joypad(&mut self, joypad: &mut Joypad);
}