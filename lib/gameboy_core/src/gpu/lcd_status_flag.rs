bitflags! {
    pub struct LcdStatusFlag : u8 {
        const MODE_LOW          = 0b00000001;
        const MODE_HIGH         = 0b00000010;
        const COINCIDENCE       = 0b00000100;
        const MODE_0_INTERRUPT  = 0b00001000;
        const MODE_1_INTERRUPT  = 0b00010000;
        const MODE_2_INTERRUPT  = 0b00100000;
        const LY_COINCIDENCE    = 0b01000000;
    }
}