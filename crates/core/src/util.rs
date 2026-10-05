pub fn u16_low_byte(value: u16) -> u8 {
    value as u8
}

pub fn u16_high_byte(value: u16) -> u8 {
    (value >> 8) as u8
}
