use joypad::Joypad;

pub trait Io {
    fn update_joypad(&mut self, joypad: &mut Joypad);
}