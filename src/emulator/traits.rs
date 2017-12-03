use input::keys::Joypad;

pub trait Drawer {
    fn draw(&self, pixels: &[u8; 144 * 160]);
}

pub trait Controller {
    fn update_controller(&self, controller: &mut Joypad);
}