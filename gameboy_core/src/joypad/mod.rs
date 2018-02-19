mod buttons;
pub mod button;

use self::buttons::Buttons;
use self::button::Button;

use mmu::interrupt::Interrupt;
use mmu::Memory;
use std::collections::HashMap;

pub struct Joypad {
    released_keys: Buttons,
    previously_unset_button_pressed: bool,
    previously_unset_direction_pressed: bool,
    button_map: HashMap<Button, Buttons>,
}

impl Joypad {
    pub fn new() -> Joypad {
        let mut button_map = HashMap::new();

        button_map.insert(Button::A, Buttons::A);
        button_map.insert(Button::B, Buttons::B);
        button_map.insert(Button::Start, Buttons::START);
        button_map.insert(Button::Select, Buttons::SELECT);
        button_map.insert(Button::Down, Buttons::DOWN);
        button_map.insert(Button::Up, Buttons::UP);
        button_map.insert(Button::Left, Buttons::LEFT);
        button_map.insert(Button::Right, Buttons::RIGHT);

        Joypad {
            released_keys: Buttons::all(),
            previously_unset_button_pressed: false,
            previously_unset_direction_pressed: false,
            button_map,
        }
    }

    pub fn update(&mut self, memory: &mut Memory) {
        if memory.are_action_keys_enabled() && self.previously_unset_button_pressed {
            println!("requesting interrupt for action keys");
            memory.request_interrupt(Interrupt::Joypad);
            self.previously_unset_button_pressed = false;
        } else if memory.are_direction_keys_enabled() && self.previously_unset_direction_pressed {
            println!("requesting interrupt for arrow keys");
            memory.request_interrupt(Interrupt::Joypad);
            self.previously_unset_direction_pressed = false;
        }

        let bits = self.released_keys.bits();
        memory.set_joypad_state(bits);
    }

    pub fn press(&mut self, button: Button) {
        let pressed = *(self.button_map.get(&button).unwrap());

        if self.released_keys.contains(pressed) {
            // was an action button just pressed?
            self.previously_unset_button_pressed = pressed.contains(
                Buttons::A | Buttons::B | Buttons::START | Buttons::SELECT
            );

            // was a direction button just pressed?
            self.previously_unset_direction_pressed = pressed.contains(
                Buttons::UP | Buttons::DOWN | Buttons::LEFT | Buttons::RIGHT
            );
        }

        self.released_keys.remove(pressed);
    }

    pub fn release(&mut self, button: Button) {
        let released = self.button_map.get(&button).unwrap();
        self.released_keys.insert(*released);
    }
}
