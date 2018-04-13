use joypad::Joypad;

pub trait Render {
    fn render(&mut self, &[u8]);
}

pub trait Input {
    fn get_input(&mut self) -> &mut Joypad;
}

pub trait Running {
    fn should_run(&self) -> bool;
}
