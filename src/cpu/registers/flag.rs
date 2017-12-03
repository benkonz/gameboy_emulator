bitflags! {
    #[derive(Default)]
    pub struct Flag : u8 {
        const ZERO          = 0b10000000;
        const NEGATIVE      = 0b01000000;
        const HALF_CARRY    = 0b00100000;
        const FULL_CARRY    = 0b00010000;
    }
}