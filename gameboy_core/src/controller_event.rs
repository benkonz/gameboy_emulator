use crate::button::Button;

#[derive(Copy, Clone)]
pub enum ControllerEvent {
    Pressed(Button),
    Released(Button),
}
