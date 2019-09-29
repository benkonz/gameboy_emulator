bitflags! {
    pub struct LcdControlFlag : u8 {
        const BACKGROUND            = 0b000_00001;
        const SPRITES               = 0b000_00010;
        const SPRITES_SIZE          = 0b000_00100;
        const BACKGROUND_TILE_MAP   = 0b000_01000;
        const BACKGROUND_TILE_SET   = 0b000_10000;
        const WINDOW                = 0b001_00000;
        const WINDOW_TILE_MAP       = 0b010_00000;
        const DISPLAY               = 0b100_00000;
    }
}
