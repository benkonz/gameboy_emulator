bitflags! {
    #[derive(Default)]
    pub struct Flag : u8 {
        const ZERO          = 0b1000_0000;
        const NEGATIVE      = 0b0100_0000;
        const HALF_CARRY    = 0b0010_0000;
        const FULL_CARRY    = 0b0001_0000;
    }
}
