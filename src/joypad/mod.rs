pub mod direction_keys;
pub mod action_keys;

use input::keys::direction_keys::DirectionKeys;
use input::keys::action_keys::ActionKeys;

pub struct Joypad {
    pub action_keys: ActionKeys,
    pub direction_keys: DirectionKeys,
    pub use_direction_keys: bool
}

impl  Joypad {
    pub fn new() -> Joypad {
        Joypad {
            action_keys: Default::default(),
            direction_keys: Default::default(),
            use_direction_keys: false
        }
    }
}