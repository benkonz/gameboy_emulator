use bitflags::bitflags;

bitflags! {
    pub struct LcdControlFlag : u8 {
        const BACKGROUND            = 0b0000_0001;
        const SPRITES               = 0b0000_0010;
        const SPRITES_SIZE          = 0b0000_0100;
        const BACKGROUND_TILE_MAP   = 0b0000_1000;
        const BACKGROUND_TILE_SET   = 0b0001_0000;
        const WINDOW                = 0b0010_0000;
        const WINDOW_TILE_MAP       = 0b0100_0000;
        const DISPLAY               = 0b1000_0000;
    }
}
