#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Interrupt {
    Vblank = 0,
    Lcd = 1,
    Timer = 2,
    Serial = 3,
    Joypad = 4,
}
