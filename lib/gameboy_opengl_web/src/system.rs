use std::rc::Rc;
use std::cell::RefCell;
use screen::Screen;
use gameboy_core::traits::*;
use gameboy_core::joypad::Joypad;
use gameboy_core::Color;

pub struct System {
    rc: Rc<RefCell<Screen>>,
    joypad: Joypad
}

impl System {
    pub fn new() -> System {
        let screen = Screen::new();

        System {
            rc: Rc::new(RefCell::new(screen)),
            joypad: Joypad::new()
        }
    }

    pub fn should_run(&self) -> bool {
        self.rc.borrow_mut().should_run()
    }
}

impl Render for System {
    fn render(&mut self) {
        let rc = self.rc.clone();
        self.rc.borrow_mut().animate(rc);
    }
}

impl Input for System {
    fn get_input(&mut self) -> &mut Joypad {
        &mut self.joypad
    }
}

impl PixelMapper for System {
    fn map_pixel(&mut self, x: u8, y: u8, color: Color) {
        self.rc.borrow_mut().map_pixel(x, y, color);
    }

    fn get_pixel(&self, x: u8, y: u8) -> Color {
        self.rc.borrow_mut().get_pixel(x, y)
    }
}