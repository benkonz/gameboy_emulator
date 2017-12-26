pub mod action_keys;
pub mod direction_keys;

use self::direction_keys::DirectionKeys;
use self::action_keys::ActionKeys;

use mmu::Memory;

pub struct Joypad {
    use_direction_keys: bool,
    direction_keys: DirectionKeys,
    action_keys: ActionKeys,
}

impl Joypad {
    pub fn new() -> Joypad {
        Joypad {
            use_direction_keys: false,
            direction_keys: DirectionKeys::empty(),
            action_keys: ActionKeys::empty()
        }
    }

//    pub fn from_memory(memory: &Memory) -> Joypad {}

    pub fn save_to_memory(&self, memory: &mut Memory) {}
}