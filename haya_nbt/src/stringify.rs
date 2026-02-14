use crate::list::ListPrimitive;
use crate::{Compound, Error, List, Read as _, Tag, TagArray, TagPrimitive};
use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use core::str::from_utf8_unchecked;
use mser::{hex_to_u8, u8_to_hex};

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
        unsafe { decode(&mut n.as_bytes(), 512).map(Self) }
    }

    #[inline]
    pub fn encode(&self, buf: &mut Vec<u8>) {
        encode(buf, &self.0)
    }
}

#[inline]
fn find_ascii(n: &[u8], p: impl FnMut(u8) -> bool) -> Result<usize, Error> {
    match n.iter().copied().position(p) {
        Some(x) => Ok(x),
        None => Err(Error),
    }
}

fn find_next_value(n: &[u8]) -> Result<(&[u8], &[u8]), Error> {
    let i = find_ascii(n, |x| {
        matches!(x, b',' | b']' | b'}' | b' ' | b'\n' | b'\t' | b'\r')
    })?;
    unsafe { Ok(n.split_at_unchecked(i)) }
}

fn peek(n: &[u8]) -> Result<(u8, &[u8]), Error> {
    match n {
        [x, rest @ ..] => Ok((*x, rest)),
        _ => Err(Error),
    }
}

fn dec_arr_peek(n: &mut &[u8], tmp: &mut TBuf) -> Result<TagArray, Error> {
    match n {
        [b'B', b';', rest @ ..] => {
            *n = rest;
            skip_ws(n);

            let mut vec = Vec::new();
            if let (b']', rest) = peek(n)? {
                *n = rest;
                return Ok(TagArray::Byte(vec));
            }
            loop {
                skip_ws(n);
                let (value, rest) = find_next_value(n)?;
                *n = rest;
                let a = match dec_num(value, tmp.next()) {
                    Ok(TagPrimitive::Byte(l)) => l,
                    _ => return Err(Error),
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
            Ok(TagArray::Byte(vec))
        }
        [b'I', b';', rest @ ..] => {
            *n = rest;
            skip_ws(n);

            let mut vec = Vec::new();
            if let (b']', rest) = peek(n)? {
                *n = rest;
                return Ok(TagArray::Int(vec));
            }
            loop {
                skip_ws(n);
                let (value, rest) = find_next_value(n)?;
                *n = rest;
                let a = match dec_num(value, tmp.next()) {
                    Ok(TagPrimitive::Int(l)) => l,
                    _ => return Err(Error),
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
            Ok(TagArray::Int(vec))
        }
        [b'L', b';', rest @ ..] => {
            *n = rest;
            skip_ws(n);

            let mut vec = Vec::new();
            if let (b']', rest) = peek(n)? {
                *n = rest;
                return Ok(TagArray::Long(vec));
            }
            loop {
                skip_ws(n);
                let (value, rest) = find_next_value(n)?;
                *n = rest;
                let a = match dec_num(value, tmp.next()) {
                    Ok(TagPrimitive::Long(l)) => l,
                    _ => return Err(Error),
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
            Ok(TagArray::Long(vec))
        }
        _ => Err(Error),
    }
}

struct TBuf(Vec<u8>);
impl TBuf {
    fn next(&mut self) -> &mut Vec<u8> {
        self.0.clear();
        &mut self.0
    }
}

unsafe fn decode(n: &mut &[u8], max_depth: usize) -> Result<Compound, Error> {
    enum Bl {
        C(Compound),
        L(List),
    }

    let mut tmp = TBuf(Vec::new());
    let mut names = Vec::<u8>::new();
    let mut blocks = vec![Bl::C(Compound::new())];
    let mut on_start = true;
    let mut on_end = false;

    skip_ws(n);
    if u8::read(n)? != b'{' {
        return Err(Error);
    }
    loop {
        // step 1 or none
        if blocks.len() == max_depth {
            return Err(Error);
        }
        let mut bl = match blocks.pop() {
            Some(x) => x,
            None => return Err(Error),
        };
        skip_ws(n);

        if on_start {
            on_start = false;
            if matches!(bl, Bl::C(..)) {
                if let (b'}', rest) = peek(n)? {
                    *n = rest;
                    on_end = true;
                }
            } else {
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
        } else {
            match u8::read(n)? {
                b']' => on_end = true,
                b',' => (),
                _ => return Err(Error),
            }
        }
        if !on_end {
            skip_ws(n);
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
            match (blocks.last_mut(), bl) {
                (Some(Bl::C(c)), bl2) => match names[..] {
                    [ref rest @ .., l1, l2, l3, l4] => {
                        let len = u32::from_le_bytes([l1, l2, l3, l4]) as usize;
                        let new_len = rest.len() - len;
                        c.push(
                            unsafe {
                                Box::from(from_utf8_unchecked(
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
                (Some(Bl::L(l)), Bl::C(x)) => {
                    if let List::None = l {
                        *l = List::Compound(Vec::new())
                    }
                    if let List::Compound(l) = l {
                        l.push(x);
                        continue;
                    } else {
                        return Err(Error);
                    }
                }
                (Some(Bl::L(l)), Bl::L(x)) => {
                    if let List::None = l {
                        *l = List::List(Vec::new())
                    }
                    if let List::List(l) = l {
                        l.push(x);
                        continue;
                    } else {
                        return Err(Error);
                    }
                }
                (None, Bl::C(x)) => return Ok(x),
                _ => return Err(Error),
            }
        }
        match bl {
            Bl::C(mut c) => {
                let k = match peek(n)? {
                    (b'\"', rest) => {
                        *n = rest;
                        dec_quoted_str(n, &mut names, b'\"')?
                    }
                    (b'\'', rest) => {
                        *n = rest;
                        dec_quoted_str(n, &mut names, b'\'')?
                    }
                    _ => unsafe {
                        let x =
                            find_ascii(n, |x| matches!(x, b':' | b' ' | b'\n' | b'\t' | b'\r'))?;
                        let m = n.get_unchecked(0..x);
                        let a = names.len();
                        names.extend(m);
                        let m = names.get_unchecked(a..);
                        *n = n.get_unchecked(x..);
                        m
                    },
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
                    (b'{', rest) => {
                        names.extend(kl);
                        blocks.push(Bl::C(c));
                        blocks.push(Bl::C(Compound::new()));
                        *n = rest;
                        on_start = true;
                        continue;
                    }
                    (b'[', rest) => {
                        *n = rest;
                        match dec_arr_peek(n, &mut tmp) {
                            Ok(TagArray::Byte(x)) => Tag::ByteArray(x),
                            Ok(TagArray::Int(x)) => Tag::IntArray(x),
                            Ok(TagArray::Long(x)) => Tag::LongArray(x),
                            Err(_) => {
                                names.extend(kl);
                                blocks.push(Bl::C(c));
                                blocks.push(Bl::L(List::None));
                                on_start = true;
                                continue;
                            }
                        }
                    }
                    (b'"', rest) => unsafe {
                        *n = rest;
                        let s = dec_quoted_str(n, tmp.next(), b'"')?;
                        Tag::String(Box::from(from_utf8_unchecked(s)))
                    },
                    (b'\'', rest) => unsafe {
                        *n = rest;
                        let s = dec_quoted_str(n, tmp.next(), b'\'')?;
                        Tag::String(Box::from(from_utf8_unchecked(s)))
                    },
                    _ => unsafe {
                        let (value, rest) = find_next_value(n)?;
                        *n = rest;
                        match dec_num(value, tmp.next()) {
                            Ok(x) => Tag::from(x),
                            Err(_) => Tag::String(Box::from(from_utf8_unchecked(value))),
                        }
                    },
                };
                unsafe {
                    c.push(Box::from(from_utf8_unchecked(k)), t);
                }
            }
            Bl::L(mut l) => match peek(n)? {
                (b'{', rest) => {
                    if let List::None = &l {
                        l = List::Compound(Vec::new());
                    }
                    blocks.push(Bl::L(l));
                    blocks.push(Bl::C(Compound::new()));
                    *n = rest;
                    on_start = true;
                }
                (b'[', rest) => {
                    *n = rest;
                    match dec_arr_peek(n, &mut tmp) {
                        Ok(arr) => {
                            match arr {
                                TagArray::Byte(b) => {
                                    if let List::None = &l {
                                        l = List::ByteArray(Vec::new());
                                    }
                                    match &mut l {
                                        List::ByteArray(x) => x.push(b),
                                        _ => return Err(Error),
                                    }
                                }
                                TagArray::Int(b) => {
                                    if let List::None = &l {
                                        l = List::IntArray(Vec::new());
                                    }
                                    match &mut l {
                                        List::IntArray(x) => x.push(b),
                                        _ => return Err(Error),
                                    }
                                }
                                TagArray::Long(b) => {
                                    if let List::None = &l {
                                        l = List::LongArray(Vec::new());
                                    }
                                    match &mut l {
                                        List::LongArray(x) => x.push(b),
                                        _ => return Err(Error),
                                    }
                                }
                            }
                            blocks.push(Bl::L(l));
                        }
                        Err(_) => {
                            match &l {
                                List::None => {
                                    l = List::List(Vec::new());
                                }
                                List::List(_) => (),
                                _ => return Err(Error),
                            }
                            blocks.push(Bl::L(l));
                            blocks.push(Bl::L(List::None));
                            on_start = true;
                        }
                    }
                }
                _ => unsafe {
                    let first = peek(n)?;
                    let tag = match first {
                        (b'"', rest) => {
                            *n = rest;
                            let s = dec_quoted_str(n, tmp.next(), b'"')?;
                            Err(Box::from(from_utf8_unchecked(s)))
                        }
                        (b'\'', rest) => {
                            *n = rest;
                            let s = dec_quoted_str(n, tmp.next(), b'\'')?;
                            Err(Box::from(from_utf8_unchecked(s)))
                        }
                        _ => {
                            let (value, rest) = find_next_value(n)?;
                            *n = rest;
                            match dec_num(value, tmp.next()) {
                                Ok(x) => Ok(x),
                                Err(_) => Err(Box::from(from_utf8_unchecked(value))),
                            }
                        }
                    };
                    let l = match tag {
                        Ok(x) => List::from(dec_list_primitive(n, &mut tmp, x)?),
                        Err(e) => {
                            let mut list = vec![e];
                            dec_list_string(n, &mut tmp, &mut list)?;
                            List::String(list)
                        }
                    };
                    blocks.push(Bl::L(l));
                },
            },
        }
    }
}

unsafe fn dec_list_string(
    n: &mut &[u8],
    tmp: &mut TBuf,
    list: &mut Vec<Box<str>>,
) -> Result<(), Error> {
    loop {
        skip_ws(n);
        match u8::read(n)? {
            b',' => {
                skip_ws(n);
                let buf = tmp.next();
                let x = match peek(n)? {
                    (b'\"', rest) => unsafe {
                        *n = rest;
                        Box::from(from_utf8_unchecked(dec_quoted_str(n, buf, b'\"')?))
                    },
                    (b'\'', rest) => unsafe {
                        *n = rest;
                        Box::from(from_utf8_unchecked(dec_quoted_str(n, buf, b'\'')?))
                    },
                    _ => unsafe {
                        let (value, rest) = find_next_value(n)?;
                        let cloned = Box::from(from_utf8_unchecked(value));
                        *n = rest;
                        cloned
                    },
                };
                list.push(x);
            }
            b']' => return Ok(()),
            _ => return Err(Error),
        }
    }
}

unsafe fn dec_list_primitive(
    n: &mut &[u8],
    tmp: &mut TBuf,
    tag: TagPrimitive,
) -> Result<ListPrimitive, Error> {
    let mut list = match tag {
        TagPrimitive::Byte(x) => ListPrimitive::Byte(vec![x]),
        TagPrimitive::Short(x) => ListPrimitive::Short(vec![x]),
        TagPrimitive::Int(x) => ListPrimitive::Int(vec![x]),
        TagPrimitive::Long(x) => ListPrimitive::Long(vec![x]),
        TagPrimitive::Float(x) => ListPrimitive::Float(vec![x]),
        TagPrimitive::Double(x) => ListPrimitive::Double(vec![x]),
    };
    loop {
        skip_ws(n);
        match u8::read(n)? {
            b',' => 'l: {
                skip_ws(n);
                let (value, rest) = find_next_value(n)?;
                *n = rest;
                let num = match dec_num(value, tmp.next())? {
                    TagPrimitive::Byte(x) => x as i64,
                    TagPrimitive::Short(x) => x as i64,
                    TagPrimitive::Int(x) => x as i64,
                    TagPrimitive::Long(x) => x,
                    TagPrimitive::Float(x) => match &mut list {
                        ListPrimitive::Float(v) => {
                            v.push(x);
                            break 'l;
                        }
                        ListPrimitive::Double(v) => {
                            v.push(x as f64);
                            break 'l;
                        }
                        _ => return Err(Error),
                    },
                    TagPrimitive::Double(x) => match &mut list {
                        ListPrimitive::Float(v) => {
                            v.push(x as f32);
                            break 'l;
                        }
                        ListPrimitive::Double(v) => {
                            v.push(x);
                            break 'l;
                        }
                        _ => return Err(Error),
                    },
                };
                match &mut list {
                    ListPrimitive::Byte(v) => v.push(num as i8),
                    ListPrimitive::Short(v) => v.push(num as i16),
                    ListPrimitive::Int(v) => v.push(num as i32),
                    ListPrimitive::Long(v) => v.push(num),
                    ListPrimitive::Float(v) => v.push(num as f32),
                    ListPrimitive::Double(v) => v.push(num as f64),
                }
            }
            b']' => return Ok(list),
            _ => return Err(Error),
        }
    }
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

fn dec_num(mut n: &[u8], tmp: &mut Vec<u8>) -> Result<TagPrimitive, Error> {
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

    match peek(n)? {
        (b't' | b'T', rest) => {
            return if dec_true_peek(rest)?.is_empty() {
                Ok(TagPrimitive::Byte(1))
            } else {
                Err(Error)
            };
        }
        (b'f' | b'F', rest) => {
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
        let p = peek(n)?;
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

const ESCAPE: u8 = b'\\';

fn dec_quoted_str<'a>(n: &mut &[u8], buf: &'a mut Vec<u8>, quote: u8) -> Result<&'a [u8], Error> {
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
            cur += match escape_quoted(peek, y) {
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
        Ok(buf.get_unchecked(begin..))
    }
}

fn escape_quoted(peek: u8, y: &[u8]) -> Option<(char, usize)> {
    match peek {
        ESCAPE => Some(('\\', 0usize)),
        b'\'' => Some(('\'', 0)),
        b'"' => Some(('"', 0)),
        b'b' => Some(('\x08', 0)),
        b't' => Some(('\t', 0)),
        b'r' => Some(('\r', 0)),
        b'f' => Some(('\x0c', 0)),
        b'n' => Some(('\n', 0)),
        b'x' => dec_char2(y),
        b'u' => dec_char4(y),
        b'U' => dec_char8(y),
        b'N' => {
            if let [b'{', rest @ ..] = y
                && let Ok(index) = find_ascii(rest, |x| x == b'}')
                && let Some(x) = unicode_names2::character(unsafe {
                    from_utf8_unchecked(rest.get_unchecked(0..index)).trim_ascii()
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

fn dec_char2(y: &[u8]) -> Option<(char, usize)> {
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

fn dec_char4(y: &[u8]) -> Option<(char, usize)> {
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

fn dec_char8(y: &[u8]) -> Option<(char, usize)> {
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

fn encode(buf: &mut Vec<u8>, n: &Compound) {
    #[derive(Clone, Copy)]
    enum Bl<'a> {
        C(&'a [(Box<str>, Tag)]),
        None,
        Byte(&'a [i8]),
        Short(&'a [i16]),
        Int(&'a [i32]),
        Long(&'a [i64]),
        Float(&'a [f32]),
        Double(&'a [f64]),
        String(&'a [Box<str>]),
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

    let mut i_buf = itoa::Buffer::new();
    let mut r_buf = ryu::Buffer::new();
    let mut blocks = vec![(Bl::C(n.as_ref()), 0)];

    buf.push(b'{');

    loop {
        let (bl, index) = match blocks.pop() {
            Some(x) => x,
            None => return,
        };
        match bl {
            Bl::C(x) => {
                let (name, tag) = match x.get(index) {
                    Some(t) => t,
                    None => {
                        buf.push(b'\n');
                        for _ in 0..blocks.len() {
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
                blocks.push((Bl::C(x), index + 1));
                for _ in 0..blocks.len() {
                    buf.extend(SPACE);
                }
                enc_str(buf, name);
                buf.extend(b": ");
                match &tag {
                    Tag::Byte(x) => {
                        let s = i_buf.format(*x);
                        buf.extend(s.as_bytes());
                        buf.push(b'B');
                    }
                    Tag::Short(x) => {
                        let s = i_buf.format(*x);
                        buf.extend(s.as_bytes());
                        buf.push(b'S');
                    }
                    Tag::Int(x) => {
                        let s = i_buf.format(*x);
                        buf.extend(s.as_bytes());
                    }
                    Tag::Long(x) => {
                        let s = i_buf.format(*x);
                        buf.extend(s.as_bytes());
                        buf.push(b'L');
                    }
                    Tag::Float(x) => {
                        let s = r_buf.format(*x);
                        buf.extend(s.as_bytes());
                        buf.push(b'F');
                    }
                    Tag::Double(x) => {
                        let s = r_buf.format(*x);
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
                            let s = i_buf.format(y);
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
                            let s = i_buf.format(y);
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
                            let s = i_buf.format(y);
                            buf.extend(s.as_bytes());
                            buf.push(b'l');
                        }
                        buf.push(b']');
                    }
                    Tag::List(x) => {
                        buf.push(b'[');
                        blocks.push((Bl::from(x), 0));
                    }
                    Tag::Compound(x) => {
                        buf.push(b'{');
                        blocks.push((Bl::C(x.as_ref()), 0));
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
                    let s = i_buf.format(y);
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
                    let s = i_buf.format(y);
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
                    let s = i_buf.format(y);
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
                    let s = i_buf.format(y);
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
                    let s = r_buf.format(y);
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
                    let s = r_buf.format(y);
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
                        let s = i_buf.format(z);
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
                        let s = i_buf.format(z);
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
                        let s = i_buf.format(z);
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
                    blocks.push((Bl::List(y), index + 1));
                    blocks.push((Bl::from(l), 0));
                    buf.push(b'[');
                    continue;
                }
            }
            Bl::Compound(y) => {
                if let Some(l) = y.get(index) {
                    blocks.push((Bl::Compound(y), index + 1));
                    blocks.push((Bl::C(l.as_ref()), 0));
                    if index != 0 {
                        buf.extend(b",\n");
                        for _ in 0..blocks.len() {
                            buf.extend(SPACE);
                        }
                    }
                    buf.extend(b"{\n");
                    // next depth
                    for _ in 0..blocks.len() + 1 {
                        buf.extend(SPACE);
                    }
                    continue;
                }
            }
        }
        buf.push(b']');
    }
}
