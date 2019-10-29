mod buttons;

use self::buttons::Buttons;
use button::Button;
use mmu::Memory;
use mmu::interrupt::Interrupt;

pub struct Controller {
    released: Buttons,
    previously_unset_button_pressed: bool,
    previously_unset_direction_pressed: bool,
}

fn button_to_buttons(button: Button) -> Buttons {
    match button {
        Button::Up => Buttons::UP,
        Button::Down => Buttons::DOWN,
        Button::Left => Buttons::LEFT,
        Button::Right => Buttons::RIGHT,
        Button::A => Buttons::A,
        Button::B => Buttons::B,
        Button::Start => Buttons::START,
        Button::Select => Buttons::SELECT,
    }
}

impl Controller {
    pub const fn new() -> Controller {
        Controller {
            released: Buttons::all(),
            previously_unset_button_pressed: false,
            previously_unset_direction_pressed: false,
        }
    }

    pub fn update(&mut self, memory: &mut Memory) {
        // the interrupts here don't fire correctly
        if memory.are_action_keys_enabled() && self.previously_unset_button_pressed {
            memory.request_interrupt(Interrupt::Joypad);
            self.previously_unset_button_pressed = false;
        } else if memory.are_direction_keys_enabled() && self.previously_unset_direction_pressed {
            memory.request_interrupt(Interrupt::Joypad);
            self.previously_unset_direction_pressed = false;
        }

        let bits = self.released.bits();
        memory.set_joypad_state(bits);
    }

    pub fn press(&mut self, button: Button) {
        let button = button_to_buttons(button);
        if self.released.contains(button) {
            let action_keys = Buttons::A | Buttons::B | Buttons::START | Buttons::SELECT;
            self.previously_unset_button_pressed = action_keys.contains(button);

            let direction_keys = Buttons::UP | Buttons::DOWN | Buttons::LEFT | Buttons::RIGHT;
            self.previously_unset_direction_pressed = direction_keys.contains(button);
        }

        self.released.remove(button);
    }

    pub fn release(&mut self, button: Button) {
        let button = button_to_buttons(button);
        self.released.insert(button);
    }
}
