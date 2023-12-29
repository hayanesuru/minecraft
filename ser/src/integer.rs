macro_rules! parse_signed {
    ($t:ty) => {
        impl Integer for $t {
            fn parse(mut x: &[u8]) -> (Self, usize) {
                let first = match x.first() {
                    Some(x) => *x,
                    None => return (0, 0),
                };
                let len = x.len();
                let mut pos = true;
                if first == b'-' {
                    unsafe {
                        x = x.get_unchecked(1..);
                    }
                    pos = false;
                } else if first == b'+' {
                    unsafe {
                        x = x.get_unchecked(1..);
                    }
                }
                let mut out: $t = 0;
                if pos {
                    match x.split_first() {
                        Some((&dig @ b'0'..=b'9', y)) => {
                            x = y;
                            out = out.wrapping_mul(10).wrapping_add((dig - b'0') as _);
                        }
                        _ => return (0, 0),
                    }
                    while let Some((&dig @ b'0'..=b'9', y)) = x.split_first() {
                        x = y;
                        out = out.wrapping_mul(10).wrapping_add((dig - b'0') as _);
                    }
                } else {
                    match x.split_first() {
                        Some((&dig @ b'0'..=b'9', y)) => {
                            x = y;
                            out = out.wrapping_mul(10).wrapping_sub((dig - b'0') as _);
                        }
                        _ => return (0, 0),
                    }
                    while let Some((&dig @ b'0'..=b'9', y)) = x.split_first() {
                        x = y;
                        out = out.wrapping_mul(10).wrapping_sub((dig - b'0') as _);
                    }
                }
                (out, len - x.len())
            }
        }
    };
}

macro_rules! parse_unsigned {
    ($t:ty) => {
        impl Integer for $t {
            fn parse(mut x: &[u8]) -> (Self, usize) {
                let first = match x.first() {
                    Some(x) => *x,
                    None => return (0, 0),
                };
                let len = x.len();
                if first == b'+' {
                    unsafe {
                        x = x.get_unchecked(1..);
                    }
                }
                let mut out: $t = 0;
                match x.split_first() {
                    Some((&dig @ b'0'..=b'9', y)) => {
                        x = y;
                        out = out.wrapping_mul(10).wrapping_add((dig - b'0') as _);
                    }
                    _ => return (0, 0),
                }
                while let Some((&dig @ b'0'..=b'9', y)) = x.split_first() {
                    x = y;
                    out = out.wrapping_mul(10).wrapping_add((dig - b'0') as _);
                }
                (out, len - x.len())
            }
        }
    };
}

pub trait Integer: Copy {
    fn parse(buf: &[u8]) -> (Self, usize);
}

pub fn parse_int<T: Integer>(n: &[u8]) -> (T, usize) {
    T::parse(n)
}

parse_signed!(i8);
parse_signed!(i16);
parse_signed!(i32);
parse_signed!(i64);
parse_unsigned!(u8);
parse_unsigned!(u16);
parse_unsigned!(u32);
parse_unsigned!(u64);

#[test]
fn test() {
    assert_eq!(parse_int(b"1004"), (1004, 4));
    assert_eq!(parse_int(b"-44a"), (-44, 3));
    assert_eq!(parse_int(b"+142"), (142, 4));
    assert_eq!(parse_int(b"+4544["), (4544, 5));
    assert_eq!(parse_int(b"++4544["), (0, 0));
}
