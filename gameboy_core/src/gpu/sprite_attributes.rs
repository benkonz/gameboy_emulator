use bitflags::bitflags;

bitflags! {
    pub struct SpriteAttributes : u8 {
        const PALETTE_NUMBER_BIT_0  = 0b0000_0001;
        const PALETTE_NUMBER_BIT_1  = 0b0000_0010;
        const PALETTE_NUMBER_BIT_2  = 0b0000_0100;
        const VRAM_BANK             = 0b0000_1000;
        const PALETTE               = 0b0001_0000;
        const X_FLIP                = 0b0010_0000;
        const Y_FLIP                = 0b0100_0000;
        const BACKGROUND_PRIORITY   = 0b1000_0000;
    }
}
