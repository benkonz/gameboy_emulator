use joypad::Joypad;

pub trait Io {
    fn draw(&self, pixels: &[u8; 144 * 160]);
    fn update_joypad(&self, joypad: &mut Joypad);
}