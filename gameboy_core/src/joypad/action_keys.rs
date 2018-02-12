bitflags! {
    #[derive(Default)]
    pub struct ActionKeys : u8 {
        const A =       0b00000001;
        const B =       0b00000010;
        const SELECT =  0b00000100;
        const START =   0b00001000;
    }
}