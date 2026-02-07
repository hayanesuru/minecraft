macro_rules! parse_signed {
    ($t:ty) => {
        impl Integer for $t {
            fn parse(mut x: &[u8]) -> ($t, &[u8]) {
                let is_neg;
                let dig = match x[..] {
                    [b'+', dig @ b'0'..=b'9', ref rest @ ..] => {
                        x = rest;
                        is_neg = false;
                        dig
                    }
                    [b'-', dig @ b'0'..=b'9', ref rest @ ..] => {
                        x = rest;
                        is_neg = true;
                        dig
                    }
                    [dig @ b'0'..=b'9', ref rest @ ..] => {
                        x = rest;
                        is_neg = false;
                        dig
                    }
                    _ => return (0, x),
                };
                let out = if !is_neg {
                    let mut out: $t = (dig - b'0') as $t;
                    while let [dig @ b'0'..=b'9', ref y @ ..] = x[..] {
                        x = y;
                        out = out.wrapping_mul(10).wrapping_add((dig - b'0') as $t);
                    }
                    out
                } else {
                    let mut out: $t = 0;
                    out = out.wrapping_sub((dig - b'0') as $t);
                    while let [dig @ b'0'..=b'9', ref y @ ..] = x[..] {
                        x = y;
                        out = out.wrapping_mul(10).wrapping_sub((dig - b'0') as $t);
                    }
                    out
                };
                (out, x)
            }
        }
    };
}

macro_rules! parse_unsigned {
    ($t:ty) => {
        impl Integer for $t {
            fn parse(mut x: &[u8]) -> ($t, &[u8]) {
                let dig = match x[..] {
                    [b'+', dig @ b'0'..=b'9', ref rest @ ..] => {
                        x = rest;
                        dig
                    }
                    [dig @ b'0'..=b'9', ref rest @ ..] => {
                        x = rest;
                        dig
                    }
                    _ => return (0, x),
                };
                let mut out: $t = (dig - b'0') as $t;
                while let [dig @ b'0'..=b'9', ref y @ ..] = x[..] {
                    x = y;
                    out = out.wrapping_mul(10).wrapping_add((dig - b'0') as $t);
                }
                (out, x)
            }
        }
    };
}

pub trait Integer: Copy {
    fn parse(buf: &[u8]) -> (Self, &[u8]);
}

pub fn parse_int<T: Integer>(n: &[u8]) -> (T, usize) {
    let len = n.len();
    let (out, x) = T::parse(n);
    (out, len - x.len())
}

pub fn parse_int_s<T: Integer>(n: &[u8]) -> (T, &[u8]) {
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
