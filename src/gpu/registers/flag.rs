bitflags! {
    #[derive(Default)]
    pub struct ControlFlag : u8 {
        const BACKGROUND            = 0b00000001;
        const SPRITES               = 0b00000010;
        const SPRITES_SIZE          = 0b00000100;
        const BACKGROUND_TILE_MAP   = 0b00001000;
        const BACKGROUND_TILE_SET   = 0b00010000;
        const WINDOW                = 0b00100000;
        const WINDOW_TILE_MAP       = 0b01000000;
        const DISPLAY               = 0b10000000;
    }
}