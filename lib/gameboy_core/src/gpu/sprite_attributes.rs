bitflags! {
    pub struct SpriteAttributes : u8 {
        const PALETTE               = 0b0001_0000;
        const X_FLIP                = 0b0010_0000;
        const Y_FLIP                = 0b0100_0000;
        const BACKGROUND_PRIORITY   = 0b1000_0000;
    }
}
