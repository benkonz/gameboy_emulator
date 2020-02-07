pub fn is_set(byte: u8, bit: u8) -> bool {
    byte & (1 << bit) != 0
}

pub fn set_bit(byte: u8, bit: u8) -> u8 {
    byte | (1 << bit)
}

pub fn unset_bit(byte: u8, bit: u8) -> u8 {
    byte & !(1 << bit)
}
