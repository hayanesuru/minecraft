macro_rules! parse_impl {
    ($($t:ident => $to:ident),* $(,)?) => {
        $(
        impl Integer for $t {
            fn parse(buf: &[u8]) -> (Self, usize) {
                let mut x = buf;

                let first = match x.first() {
                    Some(x) => *x,
                    None => return ($t::default(), 0),
                };
                if first == b'+' {
                    unsafe {
                        x = x.get_unchecked(1..);
                    }
                }
                let mut out: $to = 0;
                match x.split_first() {
                    Some((&dig, y)) => {
                        if let Some(dig) = hex_to_u8(dig) {
                            x = y;
                            out = out.wrapping_mul(16).wrapping_add(dig as _);
                        } else {
                            return ($t::default(), 0);
                        }
                    }
                    _ => return ($t::default(), 0),
                }
                while let Some((&dig, y)) = x.split_first() {
                    if let Some(dig) = hex_to_u8(dig) {
                        x = y;
                        out = out.wrapping_mul(16).wrapping_add(dig as _);
                    } else {
                        break;
                    }
                }
                ($t::from_ne_bytes(out.to_ne_bytes()), buf.len() - x.len())
            }
        }
        )*
    };
}

pub trait Integer: Copy {
    fn parse(buf: &[u8]) -> (Self, usize);
}

pub fn parse_hex<T: Integer>(n: &[u8]) -> (T, usize) {
    T::parse(n)
}

parse_impl! {
    u8 => u8,
    u16 => u16,
    u32 => u32,
    u64 => u64,
    i8 => u8,
    i16 => u16,
    i32 => u32,
    f32 => u32,
    f64 => u64,
}
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
