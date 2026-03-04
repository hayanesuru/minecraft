const HEX_DIG: &[u8; 16] = b"0123456789abcdef";

#[inline]
pub const fn u8_to_hex(b: u8) -> (u8, u8) {
    unsafe {
        (
            *HEX_DIG.as_ptr().add((b >> 4) as usize),
            *HEX_DIG.as_ptr().add((b & 0x0f) as usize),
        )
    }
}

#[inline]
pub const fn hex_to_u8(d: u8) -> Option<u8> {
    match d {
        b'0'..=b'9' => Some(d - b'0'),
        b'a'..=b'f' => Some(d - b'a' + 0xA),
        b'A'..=b'F' => Some(d - b'A' + 0xA),
        _ => None,
    }
}
