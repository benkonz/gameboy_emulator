bitflags! {
    pub struct SpriteAttributes : u8 {
        const PALETTE               = 0b00010000;
        const X_FLIP                = 0b00100000;
        const Y_FLIP                = 0b01000000;
        const BACKGROUND_PRIORITY   = 0b10000000;
    }
}