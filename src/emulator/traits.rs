use joypad::direction_keys::DirectionKeys;
use joypad::action_keys::ActionKeys;

pub trait Drawer {
    fn draw(&self, pixels: &[u8; 144 * 160]);
}

pub trait Controller {
    fn get_keys(&self) -> (ActionKeys, DirectionKeys);
}