bitflags! {
    pub struct LcdStatusFlag : u8 {
        const MODE_LOW          = 0b0000_0001;
        const MODE_HIGH         = 0b0000_0010;
        const COINCIDENCE       = 0b0000_0100;
        const MODE_0_INTERRUPT  = 0b0000_1000;
        const MODE_1_INTERRUPT  = 0b0001_0000;
        const MODE_2_INTERRUPT  = 0b0010_0000;
        const LY_COINCIDENCE    = 0b0100_0000;
    }
}
