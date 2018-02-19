bitflags! {
    pub struct Buttons : u8 {
        const RIGHT      = 0b00000001;
        const LEFT       = 0b00000010;
        const UP         = 0b00000100;
        const DOWN       = 0b00001000;
        const A          = 0b00010000;
        const B          = 0b00100000;
        const SELECT     = 0b01000000;
        const START      = 0b10000000;
    }
}