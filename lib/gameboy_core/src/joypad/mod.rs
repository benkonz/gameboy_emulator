mod buttons;
pub mod button;

use self::buttons::Buttons;
pub use self::button::Button;

use mmu::interrupt::Interrupt;
use mmu::Memory;
use std::collections::HashMap;

lazy_static! {
    static ref button_map: HashMap<Button, Buttons> = {
        let mut bm = HashMap::new();

        bm.insert(Button::A, Buttons::A);
        bm.insert(Button::B, Buttons::B);
        bm.insert(Button::Start, Buttons::START);
        bm.insert(Button::Select, Buttons::SELECT);
        bm.insert(Button::Down, Buttons::DOWN);
        bm.insert(Button::Up, Buttons::UP);
        bm.insert(Button::Left, Buttons::LEFT);
        bm.insert(Button::Right, Buttons::RIGHT);

        bm
    };
}

#[derive(Copy, Clone)]
pub struct Joypad {
    released_keys: Buttons,
    previously_unset_button_pressed: bool,
    previously_unset_direction_pressed: bool,
}

impl Joypad {
    pub fn new() -> Joypad {
        Joypad {
            released_keys: Buttons::all(),
            previously_unset_button_pressed: false,
            previously_unset_direction_pressed: false,
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
        let pressed = *(button_map.get(&button).unwrap());

        if self.released_keys.contains(pressed) {

            // was an action button just pressed?
            let action_keys = Buttons::A | Buttons::B | Buttons::START | Buttons::SELECT;
            self.previously_unset_button_pressed = action_keys.contains(pressed);

            // was a direction button just pressed?
            let direction_keys = Buttons::UP | Buttons::DOWN | Buttons::LEFT | Buttons::RIGHT;
            self.previously_unset_direction_pressed = direction_keys.contains(pressed);
        }

        self.released_keys.remove(pressed);
    }

    pub fn release(&mut self, button: Button) {
        let released = button_map.get(&button).unwrap();
        self.released_keys.insert(*released);
    }
}
