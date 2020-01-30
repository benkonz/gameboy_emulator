use bitflags::bitflags;

bitflags! {
    pub struct Buttons : u8 {
        const RIGHT      = 0b0000_0001;
        const LEFT       = 0b0000_0010;
        const UP         = 0b0000_0100;
        const DOWN       = 0b0000_1000;
        const A          = 0b0001_0000;
        const B          = 0b0010_0000;
        const SELECT     = 0b0100_0000;
        const START      = 0b1000_0000;
    }
}
