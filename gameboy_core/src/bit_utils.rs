pub fn is_set(byte: u8, bit: u8) -> bool {
    byte & (1 << bit) == (1 << bit)
}
