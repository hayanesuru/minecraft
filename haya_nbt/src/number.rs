use crate::TagPrimitive;
use alloc::vec::Vec;
use core::str::from_utf8_unchecked;
use haya_str::hex_to_u8;
use mser::Error;

#[derive(Clone, Copy)]
enum Suffix {
    SignedByte,
    UnsignedByte,
    SignedShort,
    UnsignedShort,
    SignedInt,
    UnsignedInt,
    SignedLong,
    UnsignedLong,
    Auto,
}

#[derive(Clone, Copy)]
enum Radix {
    Hexadecimal,
    Binary,
    Decimal,
    NegativeHexadecimal,
    NegativeBinary,
    NegativeDecimal,
}

#[derive(Clone, Copy)]
enum FloatParser {
    Float,
    Double,
    None,
}

pub(crate) unsafe fn dec_num(mut n: &[u8], tmp: &mut Vec<u8>) -> Result<TagPrimitive, Error> {
    match n.split_first() {
        Some((b't' | b'T', rest)) => {
            return if dec_true_peek(rest)?.is_empty() {
                Ok(TagPrimitive::Byte(1))
            } else {
                Err(Error)
            };
        }
        Some((b'f' | b'F', rest)) => {
            return if dec_false_peek(rest)?.is_empty() {
                Ok(TagPrimitive::Byte(0))
            } else {
                Err(Error)
            };
        }
        _ => (),
    }
    let radix = match n {
        [b'0', b'x' | b'X', rest @ ..] => {
            n = rest;
            Radix::Hexadecimal
        }
        [b'0', b'b' | b'B', rest @ ..] => {
            n = rest;
            Radix::Binary
        }
        [b'-', b'0', b'x' | b'X', rest @ ..] => {
            n = rest;
            Radix::NegativeHexadecimal
        }
        [b'-', b'0', b'b' | b'B', rest @ ..] => {
            n = rest;
            Radix::NegativeBinary
        }
        [b'+', b'0', b'x' | b'X', rest @ ..] => {
            n = rest;
            Radix::Hexadecimal
        }
        [b'+', b'0', b'b' | b'B', rest @ ..] => {
            n = rest;
            Radix::Binary
        }
        [b'.' | b'0'..=b'9' | b'+' | b'-', ..] => Radix::Decimal,
        _ => return Err(Error),
    };
    let (last, rest) = match n {
        [rest @ .., a] => (*a, rest),
        _ => return Err(Error),
    };
    let suffix = match last {
        b'B' | b'b' => match rest[..] {
            [ref rest @ .., b'U' | b'u'] => {
                n = rest;
                Suffix::UnsignedByte
            }
            [ref rest @ .., b'S' | b's'] => {
                n = rest;
                Suffix::SignedByte
            }
            _ => match radix {
                Radix::Hexadecimal | Radix::NegativeHexadecimal => Suffix::SignedInt,
                _ => {
                    n = rest;
                    Suffix::SignedByte
                }
            },
        },
        b'S' | b's' => match rest[..] {
            [ref rest @ .., b'U' | b'u'] => {
                n = rest;
                Suffix::UnsignedShort
            }
            [ref rest @ .., b'S' | b's'] => {
                n = rest;
                Suffix::SignedShort
            }
            _ => {
                n = rest;
                Suffix::SignedShort
            }
        },
        b'I' | b'i' => match rest[..] {
            [ref rest @ .., b'U' | b'u'] => {
                n = rest;
                Suffix::UnsignedInt
            }
            [ref rest @ .., b'S' | b's'] => {
                n = rest;
                Suffix::SignedInt
            }
            _ => {
                n = rest;
                Suffix::SignedInt
            }
        },
        b'L' | b'l' => match rest[..] {
            [ref rest @ .., b'U' | b'u'] => {
                n = rest;
                Suffix::UnsignedLong
            }
            [ref rest @ .., b'S' | b's'] => {
                n = rest;
                Suffix::SignedLong
            }
            _ => {
                n = rest;
                Suffix::SignedLong
            }
        },
        _ => Suffix::Auto,
    };
    let mut parser = if let Suffix::Auto = suffix
        && let Radix::Decimal = radix
    {
        match last {
            b'F' | b'f' => {
                n = rest;
                FloatParser::Float
            }
            b'D' | b'd' => {
                n = rest;
                FloatParser::Double
            }
            _ => FloatParser::None,
        }
    } else {
        FloatParser::None
    };
    let radix = if let FloatParser::None = parser
        && let Radix::Decimal = radix
    {
        let p = match n.split_first() {
            Some((x, y)) => (*x, y),
            None => return Err(Error),
        };
        let only_dig = if let Suffix::Auto = suffix {
            match p {
                (b'+' | b'-', rest) => rest,
                _ => n,
            }
            .iter()
            .all(|&x| matches!(x, b'0'..=b'9' | b'_'))
        } else {
            true
        };
        match p {
            (b'+', rest) => {
                if !only_dig {
                    parser = FloatParser::Double;
                } else {
                    n = rest;
                }
                Radix::Decimal
            }
            (b'-', rest) => {
                if !only_dig {
                    parser = FloatParser::Double;
                } else {
                    n = rest;
                }
                Radix::NegativeDecimal
            }
            _ => {
                if !only_dig {
                    parser = FloatParser::Double;
                }
                Radix::Decimal
            }
        }
    } else {
        radix
    };

    let mut start = 0;
    let mut cur = 0;
    while let Some(b'_') = n.get(cur) {
        tmp.extend(unsafe { n.get_unchecked(start..cur) });
        cur += 1;
        start = cur;
    }
    let mut n = if start != 0 {
        tmp.extend(unsafe { n.get_unchecked(start..) });
        &tmp[..]
    } else {
        n
    };

    match parser {
        FloatParser::Double => unsafe {
            return match from_utf8_unchecked(n).parse() {
                Ok(x) => Ok(TagPrimitive::Double(x)),
                Err(_) => Err(Error),
            };
        },
        FloatParser::Float => unsafe {
            return match from_utf8_unchecked(n).parse() {
                Ok(x) => Ok(TagPrimitive::Float(x)),
                Err(_) => Err(Error),
            };
        },
        FloatParser::None => {}
    }

    while let [first, rest @ ..] = n {
        if *first == b'0' {
            n = rest;
        } else {
            break;
        }
    }
    let ret = match suffix {
        Suffix::UnsignedByte
        | Suffix::UnsignedShort
        | Suffix::UnsignedInt
        | Suffix::UnsignedLong => {
            let mut out: u64 = 0;
            match radix {
                Radix::Binary => {
                    while let [dig @ b'0'..=b'1', ref y @ ..] = n[..] {
                        n = y;
                        out = out.wrapping_mul(2).wrapping_add((dig - b'0') as u64);
                    }
                }
                Radix::Decimal => {
                    while let [dig @ b'0'..=b'9', ref y @ ..] = n[..] {
                        n = y;
                        out = out.wrapping_mul(10).wrapping_add((dig - b'0') as u64);
                    }
                }
                Radix::Hexadecimal => {
                    while let [dig, ref y @ ..] = n[..] {
                        let dig = match hex_to_u8(dig) {
                            Some(x) => x,
                            None => break,
                        };
                        n = y;
                        out = out.wrapping_mul(16).wrapping_add(dig as u64);
                    }
                }
                _ => return Err(Error),
            }
            match suffix {
                Suffix::UnsignedByte => Ok(TagPrimitive::Byte(out as u8 as i8)),
                Suffix::UnsignedShort => Ok(TagPrimitive::Short(out as u16 as i16)),
                Suffix::UnsignedInt => Ok(TagPrimitive::Int(out as u32 as i32)),
                _ => Ok(TagPrimitive::Long(out as i64)),
            }
        }
        _ => {
            let mut out: i64 = 0;
            match radix {
                Radix::Binary => {
                    while let [dig @ b'0'..=b'1', ref y @ ..] = n[..] {
                        n = y;
                        out = out.wrapping_mul(2).wrapping_add((dig - b'0') as i64);
                    }
                }
                Radix::NegativeBinary => {
                    while let [dig @ b'0'..=b'1', ref y @ ..] = n[..] {
                        n = y;
                        out = out.wrapping_mul(2).wrapping_sub((dig - b'0') as i64);
                    }
                }
                Radix::Decimal => {
                    while let [dig @ b'0'..=b'9', ref y @ ..] = n[..] {
                        n = y;
                        out = out.wrapping_mul(10).wrapping_add((dig - b'0') as i64);
                    }
                }
                Radix::NegativeDecimal => {
                    while let [dig @ b'0'..=b'9', ref y @ ..] = n[..] {
                        n = y;
                        out = out.wrapping_mul(10).wrapping_sub((dig - b'0') as i64);
                    }
                }
                Radix::Hexadecimal => {
                    while let [dig, ref y @ ..] = n[..] {
                        let dig = match hex_to_u8(dig) {
                            Some(x) => x,
                            None => break,
                        };
                        n = y;
                        out = out.wrapping_mul(16).wrapping_add(dig as i64);
                    }
                }
                Radix::NegativeHexadecimal => {
                    while let [dig, ref y @ ..] = n[..] {
                        let dig = match hex_to_u8(dig) {
                            Some(x) => x,
                            None => break,
                        };
                        n = y;
                        out = out.wrapping_mul(16).wrapping_sub(dig as i64);
                    }
                }
            }
            match suffix {
                Suffix::SignedByte => Ok(TagPrimitive::Byte(out as i8)),
                Suffix::SignedShort => Ok(TagPrimitive::Short(out as i16)),
                Suffix::SignedInt | Suffix::Auto => Ok(TagPrimitive::Int(out as i32)),
                _ => Ok(TagPrimitive::Long(out)),
            }
        }
    };
    if n.is_empty() { ret } else { Err(Error) }
}

fn dec_true_peek(n: &[u8]) -> Result<&[u8], Error> {
    match n {
        [b'r' | b'R', b'u' | b'U', b'e' | b'E', rest @ ..] => Ok(rest),
        _ => Err(Error),
    }
}

fn dec_false_peek(n: &[u8]) -> Result<&[u8], Error> {
    match n {
        [
            b'a' | b'A',
            b'l' | b'L',
            b's' | b'S',
            b'e' | b'E',
            rest @ ..,
        ] => Ok(rest),
        _ => Err(Error),
    }
}
