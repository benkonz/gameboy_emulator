bitflags! {
    #[derive(Default)]
    pub struct DirectionKeys : u8 {
        const RIGHT = 0b00000001;
        const LEFT =  0b00000010;
        const UP =    0b00000100;
        const DOWN =  0b00001000;
    }
}