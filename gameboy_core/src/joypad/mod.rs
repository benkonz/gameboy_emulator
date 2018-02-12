mod action_keys;
mod direction_keys;
pub mod button;

use self::direction_keys::DirectionKeys;
use self::action_keys::ActionKeys;
use self::button::Button;

use mmu::Memory;

const JOYPAD_INDEX: u16 = 0xFF00;

pub struct Joypad {
    direction_keys: DirectionKeys,
    action_keys: ActionKeys,
}

impl Joypad {
    pub fn new() -> Joypad {
        Joypad {
            direction_keys: DirectionKeys::empty(),
            action_keys: ActionKeys::empty(),
        }
    }

    pub fn press(&mut self, button: Button) {
        match button {
            Button::A => self.action_keys.insert(ActionKeys::A),
            Button::B => self.action_keys.insert(ActionKeys::B),
            Button::Start => self.action_keys.insert(ActionKeys::START),
            Button::Select => self.action_keys.insert(ActionKeys::SELECT),
            Button::Up => self.direction_keys.insert(DirectionKeys::UP),
            Button::Down => self.direction_keys.insert(DirectionKeys::DOWN),
            Button::Left => self.direction_keys.insert(DirectionKeys::LEFT),
            Button::Right => self.direction_keys.insert(DirectionKeys::RIGHT),
        };
    }

    pub fn release(&mut self, button: Button) {
        match button {
            Button::A => self.action_keys.remove(ActionKeys::A),
            Button::B => self.action_keys.remove(ActionKeys::B),
            Button::Start => self.action_keys.remove(ActionKeys::START),
            Button::Select => self.action_keys.remove(ActionKeys::SELECT),
            Button::Up => self.direction_keys.remove(DirectionKeys::UP),
            Button::Down => self.direction_keys.remove(DirectionKeys::DOWN),
            Button::Left => self.direction_keys.remove(DirectionKeys::LEFT),
            Button::Right => self.direction_keys.remove(DirectionKeys::RIGHT),
        };
    }

    pub fn save_to_memory(&self, memory: &mut Memory) {
        let old_keys = memory.read_byte(JOYPAD_INDEX);
        let mut new_keys = match old_keys & 0x30 {
            0x10 => self.action_keys.bits(),
            0x20 => self.direction_keys.bits(),
            _ => 0,
        };
        new_keys = (old_keys & 0xF0) | new_keys;

        memory.write_byte(JOYPAD_INDEX, new_keys);
    }
}
