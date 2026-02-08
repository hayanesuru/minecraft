use super::{Compound, List, Tag};
use crate::nbt::TagArray;
use crate::str::BoxStr;
use crate::{Error, Read as _};
use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use mser::{hex_to_u8, parse_float, parse_int_s, u8_to_hex};

const BYTE_ARRAY_PREFIX: &[u8; 3] = b"[B;";
const INT_ARRAY_PREFIX: &[u8; 3] = b"[I;";
const LONG_ARRAY_PREFIX: &[u8; 3] = b"[L;";

#[derive(Clone)]
#[repr(transparent)]
pub struct StringifyCompound(pub Compound);

impl From<Compound> for StringifyCompound {
    #[inline]
    fn from(value: Compound) -> Self {
        Self(value)
    }
}

impl StringifyCompound {
    #[inline]
    pub fn decode(n: &str) -> Result<Self, Error> {
        unsafe { decode(&mut n.as_bytes()).map(Self) }
    }

    #[inline]
    pub fn encode(&self, buf: &mut Vec<u8>) {
        unsafe { encode(buf, &self.0) }
    }
}

#[inline]
fn find_ascii(n: &[u8], p: impl FnMut(u8) -> bool) -> Result<usize, Error> {
    match n.iter().copied().position(p) {
        Some(x) => Ok(x),
        None => Err(Error),
    }
}

fn peek(n: &[u8]) -> Result<(u8, &[u8]), Error> {
    match n {
        [x, rest @ ..] => Ok((*x, rest)),
        _ => Err(Error),
    }
}

fn dec_arr_peek(n: &mut &[u8]) -> Result<TagArray, Error> {
    let (first, second, rest) = match n {
        [_, b, c, rest @ ..] => (*b, *c, rest),
        _ => return Err(Error),
    };
    match [first, second] {
        [b'B', b';'] => {
            *n = rest;
            skip_ws(n);

            let mut vec = Vec::new();
            if let (b']', rest) = peek(n)? {
                *n = rest;
                return Ok(TagArray::ByteArray(vec));
            }
            loop {
                skip_ws(n);
                let x = match peek(n)? {
                    (b't' | b'T', rest) => {
                        *n = dec_true_peek(rest)?;
                        1
                    }
                    (b'f' | b'F', rest) => {
                        *n = dec_false_peek(rest)?;
                        0
                    }
                    _ => {
                        let (a, b) = parse_int_s::<i8>(n);
                        *n = match peek(b)? {
                            (b'B' | b'b', rest) => rest,
                            _ => b,
                        };
                        a
                    }
                };
                skip_ws(n);
                vec.push(x);
                match u8::read(n)? {
                    b']' => break,
                    b',' => continue,
                    _ => return Err(Error),
                }
            }
            vec.shrink_to_fit();
            Ok(TagArray::ByteArray(vec))
        }
        [b'I', b';'] => {
            *n = rest;
            skip_ws(n);

            let mut vec = Vec::new();
            if let (b']', rest) = peek(n)? {
                *n = rest;
                return Ok(TagArray::IntArray(vec));
            }
            loop {
                skip_ws(n);
                let (x, l) = parse_int_s::<i32>(n);
                *n = l;
                skip_ws(n);
                vec.push(x);
                match u8::read(n)? {
                    b']' => break,
                    b',' => continue,
                    _ => return Err(Error),
                }
            }
            vec.shrink_to_fit();
            Ok(TagArray::IntArray(vec))
        }
        [b'L', b';'] => {
            *n = rest;
            skip_ws(n);

            let mut vec = Vec::new();
            if let (b']', rest) = peek(n)? {
                *n = rest;
                return Ok(TagArray::LongArray(vec));
            }
            loop {
                skip_ws(n);
                let (a, b) = parse_int_s::<i64>(n);
                *n = match peek(b)? {
                    (b'L' | b'l', rest) => rest,
                    _ => b,
                };
                skip_ws(n);
                vec.push(a);
                match u8::read(n)? {
                    b']' => break,
                    b',' => continue,
                    _ => return Err(Error),
                }
            }
            vec.shrink_to_fit();
            Ok(TagArray::LongArray(vec))
        }
        _ => Err(Error),
    }
}

unsafe fn decode(n: &mut &[u8]) -> Result<Compound, Error> {
    enum Bl {
        C(Compound),
        L(List),
    }
    let mut tmp = Vec::<u8>::new();
    let mut names = Vec::<u8>::new();
    let mut bls = Vec::<Bl>::new();
    bls.push(Bl::C(Compound::new()));
    let mut on_start = true;
    let mut on_end = false;
    loop {
        let mut bl = match bls.pop() {
            Some(x) => x,
            None => return Err(Error),
        };
        skip_ws(n);

        if on_start {
            on_start = false;
            if matches!(bl, Bl::C(..)) {
                if u8::read(n)? != b'{' {
                    return Err(Error);
                }
                skip_ws(n);
                if let (b'}', rest) = peek(n)? {
                    *n = rest;
                    on_end = true;
                }
            } else {
                if u8::read(n)? != b'[' {
                    return Err(Error);
                }
                skip_ws(n);
                if let (b']', rest) = peek(n)? {
                    *n = rest;
                    on_end = true;
                }
            }
        } else if on_end {
        } else if matches!(&bl, Bl::C(..)) {
            match u8::read(n)? {
                b'}' => on_end = true,
                b',' => (),
                _ => return Err(Error),
            }
            skip_ws(n);
        } else {
            match u8::read(n)? {
                b']' => on_end = true,
                b',' => (),
                _ => return Err(Error),
            }
            skip_ws(n);
        }
        if !on_end {
            if matches!(bl, Bl::C(..)) {
                if let (b'}', rest) = peek(n)? {
                    *n = rest;
                    on_end = true;
                }
            } else if let (b']', rest) = peek(n)? {
                *n = rest;
                on_end = true;
            }
        }
        if on_end {
            on_end = false;
            match &mut bl {
                Bl::C(x) => x.shrink_to_fit(),
                Bl::L(x) => match x {
                    List::List(x) => x.shrink_to_fit(),
                    List::Compound(x) => x.shrink_to_fit(),
                    List::String(x) => x.shrink_to_fit(),
                    List::Int(x) => x.shrink_to_fit(),
                    List::Double(x) => x.shrink_to_fit(),
                    List::Byte(x) => x.shrink_to_fit(),
                    List::Short(x) => x.shrink_to_fit(),
                    List::Long(x) => x.shrink_to_fit(),
                    List::Float(x) => x.shrink_to_fit(),
                    List::ByteArray(x) => x.shrink_to_fit(),
                    List::IntArray(x) => x.shrink_to_fit(),
                    List::LongArray(x) => x.shrink_to_fit(),
                    List::None => (),
                },
            }
            match (bls.last_mut(), bl) {
                (Some(Bl::C(c)), bl2) => match names[..] {
                    [ref rest @ .., l1, l2, l3, l4] => {
                        let len = u32::from_le_bytes([l1, l2, l3, l4]) as usize;
                        let new_len = rest.len() - len;
                        c.push(
                            unsafe {
                                BoxStr::new_unchecked(Box::from(
                                    rest.get_unchecked(rest.len() - len..rest.len()),
                                ))
                            },
                            match bl2 {
                                Bl::C(x) => Tag::Compound(x),
                                Bl::L(x) => Tag::List(x),
                            },
                        );
                        names.truncate(new_len);
                        continue;
                    }
                    _ => return Err(Error),
                },
                (Some(Bl::L(List::Compound(l))), Bl::C(x)) => {
                    l.push(x);
                    continue;
                }
                (Some(Bl::L(List::List(l))), Bl::L(x)) => {
                    l.push(x);
                    continue;
                }
                (None, Bl::C(x)) => {
                    return Ok(x);
                }
                _ => return Err(Error),
            }
        }

        match bl {
            Bl::C(mut c) => unsafe {
                let k = match peek(n)? {
                    (b'\"', rest) => {
                        *n = rest;
                        dec_quoted_str(n, &mut names, b'\"')?
                    }
                    (b'\'', rest) => {
                        *n = rest;
                        dec_quoted_str(n, &mut names, b'\'')?
                    }
                    _ => {
                        let x =
                            find_ascii(n, |x| matches!(x, b':' | b' ' | b'\n' | b'\t' | b'\r'))?;
                        let m = n.get_unchecked(0..x);
                        let a = names.len();
                        names.extend(m);
                        let m = core::str::from_utf8_unchecked(names.get_unchecked(a..));
                        *n = n.get_unchecked(x..);
                        m
                    }
                };
                let kl = (k.len() as u32).to_le_bytes();
                skip_ws(n);
                if let (b':', rest) = peek(n)? {
                    *n = rest;
                } else {
                    return Err(Error);
                }
                skip_ws(n);
                let t = match peek(n)? {
                    (b'{', _) => {
                        names.extend(kl);
                        bls.push(Bl::C(c));
                        bls.push(Bl::C(Compound::new()));
                        on_start = true;
                        continue;
                    }
                    (b'[', _) => match dec_arr_peek(n) {
                        Ok(TagArray::ByteArray(x)) => Tag::ByteArray(x),
                        Ok(TagArray::IntArray(x)) => Tag::IntArray(x),
                        Ok(TagArray::LongArray(x)) => Tag::LongArray(x),
                        Err(_) => {
                            names.extend(kl);
                            bls.push(Bl::C(c));
                            bls.push(Bl::L(List::None));
                            on_start = true;
                            continue;
                        }
                    },
                    (b'"', rest) => {
                        *n = rest;
                        tmp.clear();
                        let s = dec_quoted_str(n, &mut tmp, b'"')?;
                        Tag::String(BoxStr::new_unchecked(Box::from(s.as_bytes())))
                    }
                    (b'\'', rest) => {
                        *n = rest;
                        tmp.clear();
                        let s = dec_quoted_str(n, &mut tmp, b'\'')?;
                        Tag::String(BoxStr::new_unchecked(Box::from(s.as_bytes())))
                    }
                    _ => {
                        let mid = find_ascii(n, |x| {
                            matches!(x, b',' | b'}' | b' ' | b'\n' | b'\t' | b'\r')
                        })?;
                        let b = match n.split_at_checked(mid) {
                            Some((x, y)) => {
                                *n = y;
                                x
                            }
                            None => return Err(Error),
                        };
                        match dec_num(b, &mut tmp) {
                            Ok(x) => Tag::from(x),
                            Err(_) => Tag::String(BoxStr::new_unchecked(Box::from(b))),
                        }
                    }
                };
                c.push(BoxStr::new_unchecked(Box::from(k.as_bytes())), t);
            },
            Bl::L(mut l) => match peek(n)? {
                (b'{', _) => {
                    if let List::None = &l {
                        l = List::Compound(Vec::new());
                    }
                    bls.push(Bl::L(l));
                    bls.push(Bl::C(Compound::new()));
                    on_start = true;
                }
                (b'[', _) => {
                    if let Ok(arr) = dec_arr_peek(n) {
                        match arr {
                            TagArray::ByteArray(b) => {
                                if let List::None = &l {
                                    l = List::ByteArray(Vec::new());
                                }
                                match &mut l {
                                    List::ByteArray(x) => x.push(b),
                                    _ => return Err(Error),
                                }
                            }
                            TagArray::IntArray(b) => {
                                if let List::None = &l {
                                    l = List::IntArray(Vec::new());
                                }
                                match &mut l {
                                    List::IntArray(x) => x.push(b),
                                    _ => return Err(Error),
                                }
                            }
                            TagArray::LongArray(b) => {
                                if let List::None = &l {
                                    l = List::LongArray(Vec::new());
                                }
                                match &mut l {
                                    List::LongArray(x) => x.push(b),
                                    _ => return Err(Error),
                                }
                            }
                        }
                        bls.push(Bl::L(l));
                    } else {
                        match &l {
                            List::None => {
                                l = List::List(Vec::new());
                            }
                            List::List(_) => (),
                            _ => return Err(Error),
                        }
                        bls.push(Bl::L(l));
                        bls.push(Bl::L(List::None));
                        on_start = true;
                    }
                }
                _ => unsafe {
                    let first = peek(n)?;
                    let tag = match first {
                        (b'"', rest) => {
                            *n = rest;
                            tmp.clear();
                            let s = dec_quoted_str(n, &mut tmp, b'"')?;
                            TagNonArray::String(BoxStr::new_unchecked(Box::from(s.as_bytes())))
                        }
                        (b'\'', rest) => {
                            *n = rest;
                            tmp.clear();
                            let s = dec_quoted_str(n, &mut tmp, b'\'')?;
                            TagNonArray::String(BoxStr::new_unchecked(Box::from(s.as_bytes())))
                        }
                        _ => {
                            let i = find_ascii(n, |x| {
                                matches!(x, b',' | b']' | b' ' | b'\n' | b'\t' | b'\r')
                            })?;
                            let b = match n.split_at_checked(i) {
                                Some((x, y)) => {
                                    *n = y;
                                    x
                                }
                                None => return Err(Error),
                            };
                            match dec_num(b, &mut tmp) {
                                Ok(x) => x,
                                Err(_) => TagNonArray::String(BoxStr::new_unchecked(Box::from(b))),
                            }
                        }
                    };
                    let l = dec_list_non_array(n, &mut tmp, tag)?;
                    bls.push(Bl::L(l));
                },
            },
        }
    }
}

unsafe fn dec_list_non_array(
    n: &mut &[u8],
    s: &mut Vec<u8>,
    tag: TagNonArray,
) -> Result<List, Error> {
    Ok(match tag {
        TagNonArray::Byte(x) => {
            let mut list = vec![x];
            loop {
                skip_ws(n);
                match u8::read(n)? {
                    b',' => {
                        skip_ws(n);
                        let num = match peek(n)? {
                            (b'+' | b'-' | b'0'..=b'9', _) => {
                                let (a, b) = parse_int_s::<i8>(n);
                                *n = b;
                                if let (b'b' | b'B', rest) = peek(n)? {
                                    *n = rest;
                                }
                                a
                            }
                            (b't' | b'T', rest) => {
                                *n = dec_true_peek(rest)?;
                                1
                            }
                            (b'f' | b'F', rest) => {
                                *n = dec_false_peek(rest)?;
                                0
                            }
                            (b']', rest) => {
                                *n = rest;
                                break;
                            }
                            _ => return Err(Error),
                        };
                        list.push(num);
                    }
                    b']' => {
                        break;
                    }
                    _ => return Err(Error),
                }
            }
            List::Byte(list)
        }
        TagNonArray::Short(x) => {
            let mut list = vec![x];
            loop {
                skip_ws(n);
                match u8::read(n)? {
                    b',' => {
                        skip_ws(n);
                        match peek(n)? {
                            (b'+' | b'-' | b'0'..=b'9', _) => {}
                            (b']', rest) => {
                                *n = rest;
                                break;
                            }
                            _ => return Err(Error),
                        }
                        let (a, b) = parse_int_s::<i16>(n);
                        *n = b;
                        if let (b's' | b'S', rest) = peek(n)? {
                            *n = rest;
                        }
                        list.push(a);
                    }
                    b']' => {
                        break;
                    }
                    _ => return Err(Error),
                }
            }
            List::Short(list)
        }
        TagNonArray::Int(x) => {
            let mut list = vec![x];
            loop {
                skip_ws(n);
                match u8::read(n)? {
                    b',' => {
                        skip_ws(n);
                        match peek(n)? {
                            (b'+' | b'-' | b'0'..=b'9', _) => {}
                            (b']', rest) => {
                                *n = rest;
                                break;
                            }
                            _ => return Err(Error),
                        }
                        let (a, b) = parse_int_s::<i32>(n);
                        *n = b;
                        list.push(a);
                    }
                    b']' => {
                        break;
                    }
                    _ => return Err(Error),
                }
            }
            List::Int(list)
        }
        TagNonArray::Long(x) => {
            let mut list = vec![x];
            loop {
                skip_ws(n);
                match u8::read(n)? {
                    b',' => {
                        skip_ws(n);
                        match peek(n)? {
                            (b'+' | b'-' | b'0'..=b'9', _) => {}
                            (b']', rest) => {
                                *n = rest;
                                break;
                            }
                            _ => return Err(Error),
                        }
                        let (a, b) = parse_int_s::<i64>(n);
                        *n = b;
                        if let (b'l' | b'L', rest) = peek(n)? {
                            *n = rest;
                        }
                        list.push(a);
                    }
                    b']' => {
                        break;
                    }
                    _ => return Err(Error),
                }
            }
            List::Long(list)
        }
        TagNonArray::Float(x) => unsafe {
            let mut list = vec![x];
            loop {
                skip_ws(n);
                match u8::read(n)? {
                    b',' => {
                        skip_ws(n);
                        let is_positive = match peek(n)? {
                            (b'+', rest) => {
                                *n = rest;
                                true
                            }
                            (b'-', rest) => {
                                *n = rest;
                                false
                            }
                            (b'0'..=b'9' | b'.', _) => true,
                            (b']', rest) => {
                                *n = rest;
                                break;
                            }
                            _ => return Err(Error),
                        };
                        let (a, b) = parse_float(n, Some(is_positive));
                        *n = n.get_unchecked(b..);
                        if let (b'f' | b'F', rest) = peek(n)? {
                            *n = rest;
                        }
                        list.push(a);
                    }
                    b']' => {
                        break;
                    }
                    _ => return Err(Error),
                }
            }
            List::Float(list)
        },
        TagNonArray::Double(x) => unsafe {
            let mut list = vec![x];
            loop {
                skip_ws(n);
                match u8::read(n)? {
                    b',' => {
                        skip_ws(n);
                        let is_positive = match peek(n)? {
                            (b'+', rest) => {
                                *n = rest;
                                true
                            }
                            (b'-', rest) => {
                                *n = rest;
                                false
                            }
                            (b'0'..=b'9' | b'.', _) => true,
                            (b']', rest) => {
                                *n = rest;
                                break;
                            }
                            _ => return Err(Error),
                        };
                        let (a, b) = parse_float(n, Some(is_positive));
                        *n = n.get_unchecked(b..);
                        if let (b'd' | b'D', rest) = peek(n)? {
                            *n = rest;
                        }
                        list.push(a);
                    }
                    b']' => {
                        break;
                    }
                    _ => return Err(Error),
                }
            }
            List::Double(list)
        },
        TagNonArray::String(x) => unsafe {
            let mut list = vec![x];
            loop {
                skip_ws(n);
                match u8::read(n)? {
                    b',' => {
                        skip_ws(n);
                        match peek(n)? {
                            (b']', rest) => {
                                *n = rest;
                                break;
                            }
                            _ => {
                                s.clear();
                                let x = match peek(n)? {
                                    (b'\"', rest) => {
                                        *n = rest;
                                        Box::from(dec_quoted_str(n, s, b'\"')?.as_bytes())
                                    }
                                    (b'\'', rest) => {
                                        *n = rest;
                                        Box::from(dec_quoted_str(n, s, b'\'')?.as_bytes())
                                    }
                                    _ => {
                                        let x = find_ascii(n, |x| {
                                            matches!(x, b',' | b']' | b' ' | b'\n' | b'\t' | b'\r')
                                        })?;
                                        let m = Box::from(n.get_unchecked(0..x));
                                        *n = n.get_unchecked(x..);
                                        m
                                    }
                                };
                                list.push(BoxStr::new_unchecked(x));
                            }
                        };
                    }
                    b']' => {
                        break;
                    }
                    _ => return Err(Error),
                }
            }
            List::String(list)
        },
    })
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

#[derive(Clone)]
enum TagNonArray {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    String(BoxStr),
}

impl From<TagNonArray> for Tag {
    fn from(value: TagNonArray) -> Self {
        match value {
            TagNonArray::Byte(x) => Self::Byte(x),
            TagNonArray::Short(x) => Self::Short(x),
            TagNonArray::Int(x) => Self::Int(x),
            TagNonArray::Long(x) => Self::Long(x),
            TagNonArray::Float(x) => Self::Float(x),
            TagNonArray::Double(x) => Self::Double(x),
            TagNonArray::String(x) => Self::String(x),
        }
    }
}

fn dec_num(mut n: &[u8], tmp: &mut Vec<u8>) -> Result<TagNonArray, Error> {
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
    }

    #[derive(Clone, Copy)]
    enum FloatParser {
        Float,
        Double,
        None,
    }

    let is_positive = match peek(n)? {
        (b'+', rest) => {
            n = rest;
            true
        }
        (b'-', rest) => {
            n = rest;
            false
        }
        _ => true,
    };

    let radix = match peek(n)? {
        (b'0'..=b'9', rest) => match rest {
            [b'x' | b'X', rest @ ..] => {
                n = rest;
                Radix::Hexadecimal
            }
            [b'b' | b'B', rest @ ..] => {
                n = rest;
                Radix::Binary
            }
            _ => Radix::Decimal,
        },
        (b'.', _) => Radix::Decimal,
        (b't' | b'T', rest) => {
            return if dec_true_peek(rest)?.is_empty() {
                Ok(TagNonArray::Byte(1))
            } else {
                Err(Error)
            };
        }
        (b'f' | b'F', rest) => {
            return if dec_false_peek(rest)?.is_empty() {
                Ok(TagNonArray::Byte(0))
            } else {
                Err(Error)
            };
        }
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
                Radix::Hexadecimal => Suffix::SignedInt,
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

    let parser = if let Suffix::Auto = suffix
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
            _ => {
                if n.iter().all(|&x| matches!(x, b'0'..=b'9' | b'_')) {
                    FloatParser::None
                } else {
                    FloatParser::Double
                }
            }
        }
    } else {
        FloatParser::None
    };

    let mut start = 0;
    let mut cur = 0;
    tmp.clear();
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
    while let [first, rest @ ..] = n {
        if *first == b'0' {
            n = rest;
        } else {
            break;
        }
    }
    let ret = match parser {
        FloatParser::Double => unsafe {
            let (f, l) = parse_float(n, Some(is_positive));
            n = n.get_unchecked(l..);
            Ok(TagNonArray::Double(f))
        },
        FloatParser::Float => unsafe {
            let (f, l) = parse_float(n, Some(is_positive));
            n = n.get_unchecked(l..);
            Ok(TagNonArray::Float(f))
        },
        FloatParser::None => match suffix {
            Suffix::UnsignedByte
            | Suffix::UnsignedShort
            | Suffix::UnsignedInt
            | Suffix::UnsignedLong => match radix {
                Radix::Binary => {
                    let mut out: u64 = 0;
                    while let [dig @ b'0'..=b'1', ref y @ ..] = n[..] {
                        n = y;
                        out = out.wrapping_mul(2).wrapping_add((dig - b'0') as u64);
                    }
                    match suffix {
                        Suffix::UnsignedByte => Ok(TagNonArray::Byte(out as u8 as i8)),
                        Suffix::UnsignedShort => Ok(TagNonArray::Short(out as u16 as i16)),
                        Suffix::UnsignedInt => Ok(TagNonArray::Int(out as u32 as i32)),
                        _ => Ok(TagNonArray::Long(out as i64)),
                    }
                }
                Radix::Decimal => {
                    let mut out: u64 = 0;
                    while let [dig @ b'0'..=b'9', ref y @ ..] = n[..] {
                        n = y;
                        out = out.wrapping_mul(10).wrapping_add((dig - b'0') as u64);
                    }
                    match suffix {
                        Suffix::UnsignedByte => Ok(TagNonArray::Byte(out as u8 as i8)),
                        Suffix::UnsignedShort => Ok(TagNonArray::Short(out as u16 as i16)),
                        Suffix::UnsignedInt => Ok(TagNonArray::Int(out as u32 as i32)),
                        _ => Ok(TagNonArray::Long(out as i64)),
                    }
                }
                Radix::Hexadecimal => {
                    let mut out: u64 = 0;
                    while let [dig, ref y @ ..] = n[..] {
                        let dig = match hex_to_u8(dig) {
                            Some(x) => x,
                            None => break,
                        };
                        n = y;
                        out = out.wrapping_mul(16).wrapping_add(dig as u64);
                    }
                    match suffix {
                        Suffix::UnsignedByte => Ok(TagNonArray::Byte(out as u8 as i8)),
                        Suffix::UnsignedShort => Ok(TagNonArray::Short(out as u16 as i16)),
                        Suffix::UnsignedInt => Ok(TagNonArray::Int(out as u32 as i32)),
                        _ => Ok(TagNonArray::Long(out as i64)),
                    }
                }
            },
            _ => match radix {
                Radix::Binary => {
                    let mut out: i64 = 0;
                    if is_positive {
                        while let [dig @ b'0'..=b'1', ref y @ ..] = n[..] {
                            n = y;
                            out = out.wrapping_mul(2).wrapping_add((dig - b'0') as i64);
                        }
                    } else {
                        while let [dig @ b'0'..=b'1', ref y @ ..] = n[..] {
                            n = y;
                            out = out.wrapping_mul(2).wrapping_sub((dig - b'0') as i64);
                        }
                    }
                    match suffix {
                        Suffix::SignedByte => Ok(TagNonArray::Byte(out as i8)),
                        Suffix::SignedShort => Ok(TagNonArray::Short(out as i16)),
                        Suffix::SignedInt | Suffix::Auto => Ok(TagNonArray::Int(out as i32)),
                        _ => Ok(TagNonArray::Long(out)),
                    }
                }
                Radix::Decimal => {
                    let mut out: i64 = 0;
                    if is_positive {
                        while let [dig @ b'0'..=b'9', ref y @ ..] = n[..] {
                            n = y;
                            out = out.wrapping_mul(10).wrapping_add((dig - b'0') as i64);
                        }
                    } else {
                        while let [dig @ b'0'..=b'9', ref y @ ..] = n[..] {
                            n = y;
                            out = out.wrapping_mul(10).wrapping_sub((dig - b'0') as i64);
                        }
                    }
                    match suffix {
                        Suffix::SignedByte => Ok(TagNonArray::Byte(out as i8)),
                        Suffix::SignedShort => Ok(TagNonArray::Short(out as i16)),
                        Suffix::SignedInt | Suffix::Auto => Ok(TagNonArray::Int(out as i32)),
                        _ => Ok(TagNonArray::Long(out)),
                    }
                }
                Radix::Hexadecimal => {
                    let mut out: i64 = 0;
                    if is_positive {
                        while let [dig, ref y @ ..] = n[..] {
                            let dig = match hex_to_u8(dig) {
                                Some(x) => x,
                                None => break,
                            };
                            n = y;
                            out = out.wrapping_mul(16).wrapping_add(dig as i64);
                        }
                    } else {
                        while let [dig, ref y @ ..] = n[..] {
                            let dig = match hex_to_u8(dig) {
                                Some(x) => x,
                                None => break,
                            };
                            n = y;
                            out = out.wrapping_mul(16).wrapping_sub(dig as i64);
                        }
                    }
                    match suffix {
                        Suffix::SignedByte => Ok(TagNonArray::Byte(out as i8)),
                        Suffix::SignedShort => Ok(TagNonArray::Short(out as i16)),
                        Suffix::SignedInt | Suffix::Auto => Ok(TagNonArray::Int(out as i32)),
                        _ => Ok(TagNonArray::Long(out)),
                    }
                }
            },
        },
    };
    if n.is_empty() { ret } else { Err(Error) }
}

const ESCAPE: u8 = b'\\';

unsafe fn dec_quoted_str<'a>(
    n: &mut &[u8],
    buf: &'a mut Vec<u8>,
    quote: u8,
) -> Result<&'a str, Error> {
    let begin = buf.len();
    let mut last = 0;
    let mut cur = find_ascii(n, |p| p == ESCAPE || p == quote)?;

    loop {
        let x = match n.get(cur) {
            Some(x) => *x,
            None => return Err(Error),
        };
        if x == ESCAPE {
            let (peek, y) = match n.get(cur + 1..) {
                Some(x) => peek(x)?,
                None => return Err(Error),
            };
            buf.extend(unsafe { n.get_unchecked(last..cur) });
            cur += match quoted_elsape(peek, y) {
                Some((ch, adv)) => {
                    buf.extend(ch.encode_utf8(&mut [0; 4]).as_bytes());
                    adv + 2
                }
                None => 2,
            };
            last = cur;
            continue;
        } else if x == quote {
            break;
        } else {
            cur += 1;
        }
    }
    unsafe {
        buf.extend(n.get_unchecked(last..cur));
        *n = n.get_unchecked(1 + cur..);
        Ok(core::str::from_utf8_unchecked(buf.get_unchecked(begin..)))
    }
}

fn quoted_elsape(peek: u8, y: &[u8]) -> Option<(char, usize)> {
    match peek {
        ESCAPE => Some(('\\', 0usize)),
        b'\'' => Some(('\'', 0)),
        b'"' => Some(('"', 0)),
        b'b' => Some(('\x08', 0)),
        b't' => Some(('\t', 0)),
        b'r' => Some(('\r', 0)),
        b'f' => Some(('\x0c', 0)),
        b'n' => Some(('\n', 0)),
        b'x' => {
            if let [a, b, ..] = y[..]
                && let Some(a) = hex_to_u8(a)
                && let Some(b) = hex_to_u8(b)
            {
                let ch = (a as u32) << 4 | (b as u32);
                char::from_u32(ch).map(|x| (x, 2))
            } else {
                None
            }
        }
        b'u' => {
            if let [a, b, c, d, ..] = y[..]
                && let Some(a) = hex_to_u8(a)
                && let Some(b) = hex_to_u8(b)
                && let Some(c) = hex_to_u8(c)
                && let Some(d) = hex_to_u8(d)
            {
                let ch = (a as u32) << 12 | (b as u32) << 8 | (c as u32) << 4 | (d as u32);
                char::from_u32(ch).map(|x| (x, 4))
            } else {
                None
            }
        }
        b'U' => {
            if let [a, b, c, d, e, f, g, h, ..] = y[..]
                && let Some(a) = hex_to_u8(a)
                && let Some(b) = hex_to_u8(b)
                && let Some(c) = hex_to_u8(c)
                && let Some(d) = hex_to_u8(d)
                && let Some(e) = hex_to_u8(e)
                && let Some(f) = hex_to_u8(f)
                && let Some(g) = hex_to_u8(g)
                && let Some(h) = hex_to_u8(h)
            {
                let ch = (a as u32) << 28
                    | (b as u32) << 24
                    | (c as u32) << 20
                    | (d as u32) << 16
                    | (e as u32) << 12
                    | (f as u32) << 8
                    | (g as u32) << 4
                    | (h as u32);
                char::from_u32(ch).map(|x| (x, 8))
            } else {
                None
            }
        }
        b'N' => {
            if let [b'{', rest @ ..] = y
                && let Ok(index) = find_ascii(rest, |x| x == b'}')
                && let Some(x) = unicode_names2::character(unsafe {
                    core::str::from_utf8_unchecked(rest.get_unchecked(0..index)).trim_ascii()
                })
            {
                Some((x, index + 2))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn enc_str(buf: &mut Vec<u8>, s: &str) {
    buf.push(b'"');

    let mut last_end = 0;
    for (start, part) in s.match_indices(|x: char| matches!(x, '\\' | '"' | '\0'..='\x1F' | '\x7F'))
    {
        buf.extend(unsafe { s.get_unchecked(last_end..start).as_bytes() });
        buf.push(ESCAPE);
        if let [p] = part.as_bytes() {
            'lb: {
                let e = match *p {
                    b'"' => b'"',
                    b'\x08' => b'b',
                    b'\t' => b't',
                    b'\n' => b'n',
                    b'\r' => b'r',
                    b'\x0c' => b'f',
                    b'\\' => ESCAPE,
                    p => {
                        let (high, low) = u8_to_hex(p);
                        buf.extend([b'x', high, low]);
                        break 'lb;
                    }
                };
                buf.push(e);
            }
        }
        last_end = start + part.len();
    }
    buf.extend(unsafe { s.get_unchecked(last_end..s.len()).as_bytes() });
    buf.push(b'"');
}

/// skip whitespace characters
#[inline]
fn skip_ws(n: &mut &[u8]) {
    while let [b' ' | b'\n' | b'\t' | b'\r', rest @ ..] = n {
        *n = rest;
    }
}

const SPACE: &[u8] = b"    ";
const DELIMITER: &[u8] = b", ";

unsafe fn encode(buf: &mut Vec<u8>, n: &Compound) {
    #[derive(Clone, Copy)]
    enum Bl<'a> {
        C(&'a [(BoxStr, Tag)]),
        None,
        Byte(&'a [i8]),
        Short(&'a [i16]),
        Int(&'a [i32]),
        Long(&'a [i64]),
        Float(&'a [f32]),
        Double(&'a [f64]),
        String(&'a [BoxStr]),
        ByteArray(&'a [Vec<i8>]),
        IntArray(&'a [Vec<i32>]),
        LongArray(&'a [Vec<i64>]),
        List(&'a [List]),
        Compound(&'a [Compound]),
    }
    impl<'a> From<&'a List> for Bl<'a> {
        fn from(value: &'a List) -> Self {
            match value {
                List::None => Self::None,
                List::Byte(items) => Self::Byte(items),
                List::Short(items) => Self::Short(items),
                List::Int(items) => Self::Int(items),
                List::Long(items) => Self::Long(items),
                List::Float(items) => Self::Float(items),
                List::Double(items) => Self::Double(items),
                List::String(box_strs) => Self::String(box_strs),
                List::ByteArray(items) => Self::ByteArray(items),
                List::IntArray(items) => Self::IntArray(items),
                List::LongArray(items) => Self::LongArray(items),
                List::List(lists) => Self::List(lists),
                List::Compound(compounds) => Self::Compound(compounds),
            }
        }
    }

    let mut itoa_buf = itoa::Buffer::new();
    let mut ryu_buf = ryu::Buffer::new();
    let mut bls = vec![(Bl::C(n.as_ref()), 0)];

    buf.push(b'{');

    loop {
        let (bl, index) = match bls.pop() {
            Some(x) => x,
            None => return,
        };
        match bl {
            Bl::C(x) => {
                let (name, tag) = match x.get(index) {
                    Some(t) => t,
                    None => {
                        buf.push(b'\n');
                        for _ in 0..bls.len() {
                            buf.extend(SPACE);
                        }
                        buf.push(b'}');
                        continue;
                    }
                };
                if index != 0 {
                    buf.push(b',');
                }
                buf.push(b'\n');
                bls.push((Bl::C(x), index + 1));
                for _ in 0..bls.len() {
                    buf.extend(SPACE);
                }
                enc_str(buf, name);
                buf.extend(b": ");
                match &tag {
                    Tag::Byte(x) => {
                        let s = itoa_buf.format(*x);
                        buf.extend(s.as_bytes());
                        buf.push(b'B');
                    }
                    Tag::Short(x) => {
                        let s = itoa_buf.format(*x);
                        buf.extend(s.as_bytes());
                        buf.push(b'S');
                    }
                    Tag::Int(x) => {
                        let s = itoa_buf.format(*x);
                        buf.extend(s.as_bytes());
                    }
                    Tag::Long(x) => {
                        let s = itoa_buf.format(*x);
                        buf.extend(s.as_bytes());
                        buf.push(b'L');
                    }
                    Tag::Float(x) => {
                        let s = ryu_buf.format(*x);
                        buf.extend(s.as_bytes());
                        buf.push(b'F');
                    }
                    Tag::Double(x) => {
                        let s = ryu_buf.format(*x);
                        buf.extend(s.as_bytes());
                        buf.push(b'D');
                    }
                    Tag::String(x) => {
                        enc_str(buf, x);
                    }
                    Tag::ByteArray(x) => {
                        buf.extend(BYTE_ARRAY_PREFIX);
                        let mut flag = false;
                        for &y in x {
                            if flag {
                                buf.extend(DELIMITER);
                            }
                            flag = true;
                            let s = itoa_buf.format(y);
                            buf.extend(s.as_bytes());
                            buf.push(b'b');
                        }
                        buf.push(b']');
                    }
                    Tag::IntArray(x) => {
                        buf.extend(INT_ARRAY_PREFIX);
                        let mut flag = false;
                        for &y in x {
                            if flag {
                                buf.extend(DELIMITER);
                            }
                            flag = true;
                            let s = itoa_buf.format(y);
                            buf.extend(s.as_bytes());
                        }
                        buf.push(b']');
                    }
                    Tag::LongArray(x) => {
                        buf.extend(LONG_ARRAY_PREFIX);
                        let mut flag = false;
                        for &y in x {
                            if flag {
                                buf.extend(DELIMITER);
                            }
                            flag = true;
                            let s = itoa_buf.format(y);
                            buf.extend(s.as_bytes());
                            buf.push(b'l');
                        }
                        buf.push(b']');
                    }
                    Tag::List(x) => {
                        buf.push(b'[');
                        bls.push((Bl::from(x), 0));
                    }
                    Tag::Compound(x) => {
                        buf.push(b'{');
                        bls.push((Bl::C(x.as_ref()), 0));
                    }
                }

                continue;
            }
            Bl::None => {}
            Bl::Byte(x) => {
                let mut flag = false;
                for &y in x {
                    if flag {
                        buf.extend(DELIMITER);
                    }
                    flag = true;
                    let s = itoa_buf.format(y);
                    buf.extend(s.as_bytes());
                    buf.push(b'b');
                }
            }
            Bl::Short(x) => {
                let mut flag = false;
                for &y in x {
                    if flag {
                        buf.extend(DELIMITER);
                    }
                    flag = true;
                    let s = itoa_buf.format(y);
                    buf.extend(s.as_bytes());
                    buf.push(b's');
                }
            }
            Bl::Int(x) => {
                let mut flag = false;
                for &y in x {
                    if flag {
                        buf.extend(DELIMITER);
                    }
                    flag = true;
                    let s = itoa_buf.format(y);
                    buf.extend(s.as_bytes());
                }
            }
            Bl::Long(x) => {
                let mut flag = false;
                for &y in x {
                    if flag {
                        buf.extend(DELIMITER);
                    }
                    flag = true;
                    let s = itoa_buf.format(y);
                    buf.extend(s.as_bytes());
                    buf.push(b'l');
                }
            }
            Bl::Float(x) => {
                let mut flag = false;
                for &y in x {
                    if flag {
                        buf.extend(DELIMITER);
                    }
                    flag = true;
                    let s = ryu_buf.format(y);
                    buf.extend(s.as_bytes());
                    buf.push(b'f');
                }
            }
            Bl::Double(x) => {
                let mut flag = false;
                for &y in x {
                    if flag {
                        buf.extend(DELIMITER);
                    }
                    flag = true;
                    let s = ryu_buf.format(y);
                    buf.extend(s.as_bytes());
                    buf.push(b'd');
                }
            }
            Bl::String(x) => {
                let mut flag = false;
                for y in x {
                    if flag {
                        buf.extend(DELIMITER);
                    }
                    flag = true;
                    buf.push(b'"');
                    buf.extend(y.as_bytes());
                    buf.push(b'"');
                }
            }
            Bl::ByteArray(x) => {
                let mut flag = false;
                for y in x {
                    if flag {
                        buf.extend(DELIMITER);
                    }
                    flag = true;
                    buf.extend(BYTE_ARRAY_PREFIX);
                    let mut flag1 = false;
                    for &z in y {
                        if flag1 {
                            buf.extend(DELIMITER);
                        }
                        flag1 = true;
                        let s = itoa_buf.format(z);
                        buf.extend(s.as_bytes());
                        buf.push(b'b');
                    }
                    buf.push(b']');
                }
            }
            Bl::IntArray(x) => {
                let mut flag = false;
                for y in x {
                    if flag {
                        buf.extend(DELIMITER);
                    }
                    flag = true;
                    buf.extend(INT_ARRAY_PREFIX);
                    let mut flag1 = false;
                    for &z in y {
                        if flag1 {
                            buf.extend(DELIMITER);
                        }
                        flag1 = true;
                        let s = itoa_buf.format(z);
                        buf.extend(s.as_bytes());
                    }
                    buf.push(b']');
                }
            }
            Bl::LongArray(x) => {
                let mut flag = false;
                for y in x {
                    if flag {
                        buf.extend(DELIMITER);
                    }
                    flag = true;
                    buf.extend(LONG_ARRAY_PREFIX);
                    let mut flag1 = false;
                    for &z in y {
                        if flag1 {
                            buf.extend(DELIMITER);
                        }
                        flag1 = true;
                        let s = itoa_buf.format(z);
                        buf.extend(s.as_bytes());
                        buf.push(b'l');
                    }
                    buf.push(b']');
                }
            }
            Bl::List(y) => {
                if let Some(l) = y.get(index) {
                    if index != 0 {
                        buf.extend(DELIMITER);
                    }
                    bls.push((Bl::List(y), index + 1));
                    bls.push((Bl::from(l), 0));
                    buf.push(b'[');
                    continue;
                }
            }
            Bl::Compound(y) => {
                if let Some(l) = y.get(index) {
                    bls.push((Bl::Compound(y), index + 1));
                    bls.push((Bl::C(l.as_ref()), 0));
                    if index != 0 {
                        buf.extend(b",\n");
                        for _ in 0..bls.len() {
                            buf.extend(SPACE);
                        }
                    }
                    buf.extend(b"{\n");
                    // next depth
                    for _ in 0..bls.len() + 1 {
                        buf.extend(SPACE);
                    }
                    continue;
                }
            }
        }
        buf.push(b']');
    }
}
