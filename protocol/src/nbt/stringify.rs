use super::{Compound, List, Tag};
use crate::nbt::TagArray;
use crate::str::BoxStr;
use crate::{Error, Read as _};
use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use core::hint::unreachable_unchecked;
use mser::{parse_float, parse_int};

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
fn find_ascii(n: &[u8], mut p: impl FnMut(u8) -> bool) -> Result<usize, Error> {
    let mut index = 0;
    while let Some(&byte) = n.get(index) {
        match byte {
            x @ 0..=0x7F => {
                if p(x) {
                    return Ok(index);
                }
                index += 1
            }
            0x80..=0xDF => index += 2,
            0xE0..=0xEF => index += 3,
            _ => index += 4,
        }
    }
    Err(Error)
}

fn peek(n: &[u8]) -> Result<u8, Error> {
    match n.first() {
        Some(&byte) => Ok(byte),
        None => Err(Error),
    }
}

fn at(n: &[u8], i: usize) -> Result<u8, Error> {
    match n.get(i) {
        Some(&byte) => Ok(byte),
        None => Err(Error),
    }
}

fn darr(n: &mut &[u8]) -> Result<TagArray, Error> {
    let first = at(n, 1)?;
    let second = at(n, 2)?;
    match [first, second] {
        [b'B', b';'] => unsafe {
            let mut vec = Vec::<i8>::new();

            *n = n.get_unchecked(3..);
            skip_ws(n);
            if peek(n)? == b']' {
                *n = n.get_unchecked(1..);
                return Ok(TagArray::ByteArray(vec));
            }
            loop {
                skip_ws(n);
                let (x, len) = match parse_int::<i8>(n) {
                    (_, 0) => match peek(n)? {
                        b't' | b'T' if n.len() >= 4 => (1, 4),
                        b'f' | b'F' if n.len() >= 5 => (0, 5),
                        _ => return Err(Error),
                    },
                    (a, b) => match at(n, b)? {
                        b'B' | b'b' => (a, b + 1),
                        _ => (a, b),
                    },
                };
                vec.push(x);
                *n = n.get_unchecked(len..);
                skip_ws(n);
                match u8::read(n)? {
                    b']' => break,
                    b',' => continue,
                    _ => return Err(Error),
                }
            }
            vec.shrink_to_fit();
            Ok(TagArray::ByteArray(vec))
        },
        [b'I', b';'] => unsafe {
            let mut vec = Vec::<i32>::new();

            *n = n.get_unchecked(3..);
            skip_ws(n);
            if peek(n)? == b']' {
                *n = n.get_unchecked(1..);
                return Ok(TagArray::IntArray(vec));
            }
            loop {
                skip_ws(n);
                let (x, l) = parse_int::<i32>(n);
                vec.push(x);
                *n = n.get_unchecked(l..);
                skip_ws(n);
                match u8::read(n)? {
                    b']' => break,
                    b',' => continue,
                    _ => return Err(Error),
                }
            }
            vec.shrink_to_fit();
            Ok(TagArray::IntArray(vec))
        },
        [b'L', b';'] => unsafe {
            let mut vec = Vec::<i64>::new();

            *n = n.get_unchecked(2..);
            skip_ws(n);
            if peek(n)? == b']' {
                *n = n.get_unchecked(1..);
                return Ok(TagArray::LongArray(vec));
            }
            loop {
                skip_ws(n);
                let (a, b) = parse_int::<i64>(n);
                let (x, len) = match at(n, b)? {
                    b'L' | b'l' => (a, b + 1),
                    _ => (a, b),
                };
                vec.push(x);
                *n = n.get_unchecked(len..);
                skip_ws(n);
                match u8::read(n)? {
                    b']' => break,
                    b',' => continue,
                    _ => return Err(Error),
                }
            }
            vec.shrink_to_fit();
            Ok(TagArray::LongArray(vec))
        },
        _ => Err(Error),
    }
}

unsafe fn decode(n: &mut &[u8]) -> Result<Compound, Error> {
    enum Bl {
        C(Compound),
        L(List),
    }
    let mut s = Vec::<u8>::new();
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
                if peek(n)? == b'}' {
                    *n = unsafe { n.get_unchecked(1..) };
                    on_end = true;
                }
            } else {
                if u8::read(n)? != b'[' {
                    return Err(Error);
                }
                skip_ws(n);
                if peek(n)? == b']' {
                    *n = unsafe { n.get_unchecked(1..) };
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
                if peek(n)? == b'}' {
                    on_end = true;
                    *n = unsafe { n.get_unchecked(1..) };
                }
            } else if peek(n)? == b']' {
                on_end = true;
                *n = unsafe { n.get_unchecked(1..) };
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
                        let new_len = rest.len() - len - 4;
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
                    b'\"' => dqstr2(n, &mut names)?,
                    b'\'' => dqstr1(n, &mut names)?,
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
                *n = match n.get(1..) {
                    Some(x) => x,
                    None => return Err(Error),
                };
                skip_ws(n);
                let t = match peek(n)? {
                    b'{' => {
                        names.extend(kl);
                        bls.push(Bl::C(c));
                        bls.push(Bl::C(Compound::new()));
                        on_start = true;
                        continue;
                    }
                    b'[' => match darr(n) {
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
                    b'"' => {
                        s.clear();
                        let s = dqstr2(n, &mut s)?;
                        Tag::String(BoxStr::new_unchecked(Box::from(s.as_bytes())))
                    }
                    b'\'' => {
                        s.clear();
                        let s = dqstr1(n, &mut s)?;
                        Tag::String(BoxStr::new_unchecked(Box::from(s.as_bytes())))
                    }
                    _ => {
                        let mid = find_ascii(n, |x| {
                            matches!(x, b',' | b'}' | b' ' | b'\n' | b'\t' | b'\r')
                        })?;
                        let s = match n.split_at_checked(mid) {
                            Some((x, y)) => {
                                *n = y;
                                x
                            }
                            None => return Err(Error),
                        };
                        match dnum(s) {
                            Ok(x) => x,
                            Err(_) => Tag::String(BoxStr::new_unchecked(Box::from(s))),
                        }
                    }
                };
                c.push(BoxStr::new_unchecked(Box::from(k.as_bytes())), t);
            },
            Bl::L(mut l) => match peek(n)? {
                b'{' => {
                    if let List::None = &l {
                        l = List::Compound(Vec::new());
                    }
                    bls.push(Bl::L(l));
                    bls.push(Bl::C(Compound::new()));
                    on_start = true;
                }
                b'[' => {
                    if let Ok(arr) = darr(n) {
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
                        b'"' => {
                            s.clear();
                            let s = dqstr2(n, &mut s)?;
                            Tag::String(BoxStr::new_unchecked(Box::from(s.as_bytes())))
                        }
                        b'\'' => {
                            s.clear();
                            let s = dqstr1(n, &mut s)?;
                            Tag::String(BoxStr::new_unchecked(Box::from(s.as_bytes())))
                        }
                        _ => {
                            let i = find_ascii(n, |x| {
                                matches!(x, b',' | b']' | b' ' | b'\n' | b'\t' | b'\r')
                            })?;
                            let s = match n.split_at_checked(i) {
                                Some((x, y)) => {
                                    *n = y;
                                    x
                                }
                                None => return Err(Error),
                            };
                            match dnum(s) {
                                Ok(x) => x,
                                Err(_) => Tag::String(BoxStr::new_unchecked(Box::from(s))),
                            }
                        }
                    };
                    let l = dec_list_non_array(n, &mut s, tag)?;
                    bls.push(Bl::L(l));
                },
            },
        }
    }
}

unsafe fn dec_list_non_array(n: &mut &[u8], s: &mut Vec<u8>, tag: Tag) -> Result<List, Error> {
    Ok(match tag {
        Tag::Byte(x) => unsafe {
            let mut list = vec![x];
            loop {
                skip_ws(n);
                match u8::read(n)? {
                    b',' => {
                        skip_ws(n);
                        match peek(n)? {
                            b'+' | b'-' | b'0'..=b'9' => {
                                let (a, b) = parse_int::<i8>(n);
                                *n = n.get_unchecked(b..);
                                if let b'b' | b'B' = peek(n)? {
                                    *n = n.get_unchecked(1..);
                                }
                                list.push(a);
                            }
                            b't' | b'T' => {
                                *n = match n.get(4..) {
                                    Some(x) => x,
                                    None => return Err(Error),
                                };
                                list.push(1);
                            }
                            b'f' | b'F' => {
                                *n = match n.get(5..) {
                                    Some(x) => x,
                                    None => return Err(Error),
                                };
                                list.push(0);
                            }
                            b']' => {
                                *n = n.get_unchecked(1..);
                                break;
                            }
                            _ => return Err(Error),
                        }
                    }
                    b']' => {
                        break;
                    }
                    _ => return Err(Error),
                }
            }
            List::Byte(list)
        },
        Tag::Short(x) => unsafe {
            let mut list = vec![x];
            loop {
                skip_ws(n);
                match u8::read(n)? {
                    b',' => {
                        skip_ws(n);
                        match peek(n)? {
                            b'+' | b'-' | b'0'..=b'9' => {}
                            b']' => {
                                *n = n.get_unchecked(1..);
                                break;
                            }
                            _ => return Err(Error),
                        }
                        let (a, b) = parse_int::<i16>(n);
                        *n = n.get_unchecked(b..);
                        if let b's' | b'S' = peek(n)? {
                            *n = n.get_unchecked(1..);
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
        },
        Tag::Int(x) => unsafe {
            let mut list = vec![x];
            loop {
                skip_ws(n);
                match u8::read(n)? {
                    b',' => {
                        skip_ws(n);
                        match peek(n)? {
                            b'+' | b'-' | b'0'..=b'9' => {}
                            b']' => {
                                *n = n.get_unchecked(1..);
                                break;
                            }
                            _ => return Err(Error),
                        }
                        let (a, b) = parse_int::<i32>(n);
                        *n = n.get_unchecked(b..);
                        list.push(a);
                    }
                    b']' => {
                        break;
                    }
                    _ => return Err(Error),
                }
            }
            List::Int(list)
        },
        Tag::Long(x) => unsafe {
            let mut list = vec![x];
            loop {
                skip_ws(n);
                match u8::read(n)? {
                    b',' => {
                        skip_ws(n);
                        match peek(n)? {
                            b'+' | b'-' | b'0'..=b'9' => {}
                            b']' => {
                                *n = n.get_unchecked(1..);
                                break;
                            }
                            _ => return Err(Error),
                        }
                        let (a, b) = parse_int::<i64>(n);
                        *n = n.get_unchecked(b..);
                        if let b'l' | b'L' = peek(n)? {
                            *n = n.get_unchecked(1..);
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
        },
        Tag::Float(x) => unsafe {
            let mut list = vec![x];
            loop {
                skip_ws(n);
                match u8::read(n)? {
                    b',' => {
                        skip_ws(n);
                        match peek(n)? {
                            b'+' | b'-' | b'0'..=b'9' | b'.' => {}
                            b']' => {
                                *n = n.get_unchecked(1..);
                                break;
                            }
                            _ => return Err(Error),
                        }
                        let (a, b) = parse_float(n);
                        *n = n.get_unchecked(b..);
                        if let b'f' | b'F' = peek(n)? {
                            *n = n.get_unchecked(1..);
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
        Tag::Double(x) => unsafe {
            let mut list = vec![x];
            loop {
                skip_ws(n);
                match u8::read(n)? {
                    b',' => {
                        skip_ws(n);
                        match peek(n)? {
                            b'+' | b'-' | b'0'..=b'9' | b'.' => {}
                            b']' => {
                                *n = n.get_unchecked(1..);
                                break;
                            }
                            _ => return Err(Error),
                        }
                        let (a, b) = parse_float(n);
                        *n = n.get_unchecked(b..);
                        if let b'd' | b'D' = peek(n)? {
                            *n = n.get_unchecked(1..);
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
        Tag::String(x) => unsafe {
            let mut list = vec![x];
            loop {
                skip_ws(n);
                match u8::read(n)? {
                    b',' => {
                        skip_ws(n);
                        match peek(n)? {
                            b']' => {
                                *n = n.get_unchecked(1..);
                                break;
                            }
                            _ => {
                                s.clear();
                                let x = match peek(n)? {
                                    b'\"' => Box::from(dqstr2(n, s)?.as_bytes()),
                                    b'\'' => Box::from(dqstr1(n, s)?.as_bytes()),
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
        _ => unsafe { unreachable_unchecked() },
    })
}

fn dnum(n: &[u8]) -> Result<Tag, Error> {
    match peek(n)? {
        b'+' | b'-' | b'0'..=b'9' | b'.' => (),
        b't' | b'T' => unsafe {
            return match n.get_unchecked(1..) {
                [b'r' | b'R', b'u' | b'U', b'e' | b'E'] => Ok(Tag::Byte(1)),
                _ => Err(Error),
            };
        },
        b'f' | b'F' => unsafe {
            return match n.get_unchecked(1..) {
                [b'a' | b'A', b'l' | b'L', b's' | b'S', b'e' | b'E'] => Ok(Tag::Byte(0)),
                _ => Err(Error),
            };
        },
        _ => return Err(Error),
    }

    if let [rest @ .., a] = n {
        match *a {
            b'B' | b'b' => {
                let (a, b) = parse_int::<i8>(rest);
                if b != rest.len() {
                    Err(Error)
                } else {
                    Ok(Tag::Byte(a))
                }
            }
            b'S' | b's' => {
                let (a, b) = parse_int::<i16>(rest);
                if b != rest.len() {
                    Err(Error)
                } else {
                    Ok(Tag::Short(a))
                }
            }
            b'L' | b'l' => {
                let (a, b) = parse_int::<i64>(rest);
                if b != rest.len() {
                    Err(Error)
                } else {
                    Ok(Tag::Long(a))
                }
            }
            b'F' | b'f' => {
                let (a, b) = parse_float(rest);
                if b != rest.len() {
                    Err(Error)
                } else {
                    Ok(Tag::Float(a))
                }
            }
            b'D' | b'd' => {
                let (a, b) = parse_float(rest);
                if b != rest.len() {
                    Err(Error)
                } else {
                    Ok(Tag::Double(a))
                }
            }
            _ => unsafe {
                if n.get_unchecked(1..).iter().all(|x| x.is_ascii_digit()) {
                    let (a, b) = parse_int::<i32>(n);
                    if b != n.len() {
                        Err(Error)
                    } else {
                        Ok(Tag::Int(a))
                    }
                } else {
                    let (a, b) = parse_float(n);
                    if b != n.len() {
                        Err(Error)
                    } else {
                        Ok(Tag::Double(a))
                    }
                }
            },
        }
    } else {
        Err(Error)
    }
}

const ESCAPE: u8 = b'\\';

/// decode single quoted string
unsafe fn dqstr1<'a>(n: &mut &[u8], buf: &'a mut Vec<u8>) -> Result<&'a str, Error> {
    let begin = buf.len();
    unsafe {
        *n = n.get_unchecked(1..);
        let mut last = 0;
        let mut cur = 0;
        loop {
            let x = at(n, cur)?;
            if x == ESCAPE {
                let y = at(n, cur + 1)?;
                if y == ESCAPE {
                    buf.extend(n.get_unchecked(last..cur));
                    buf.push(ESCAPE);
                    cur += 2;
                    last = cur;
                } else if y == b'\'' {
                    buf.extend(n.get_unchecked(last..cur));
                    buf.push(b'\'');
                    cur += 2;
                    last = cur;
                } else {
                    cur += 1;
                }
            } else if x == b'\'' {
                break;
            } else if x < 0x80 {
                cur += 1;
            } else if x < 0xE0 {
                cur += 2;
            } else if x < 0xF0 {
                cur += 3;
            } else {
                cur += 4;
            }
        }
        buf.extend(n.get_unchecked(last..cur));
        *n = match n.get(1 + cur..) {
            Some(x) => x,
            None => return Err(Error),
        };
        Ok(core::str::from_utf8_unchecked(buf.get_unchecked(begin..)))
    }
}

/// decode a double quoted string
unsafe fn dqstr2<'a>(n: &mut &[u8], buf: &'a mut Vec<u8>) -> Result<&'a str, Error> {
    let begin = buf.len();
    unsafe {
        *n = n.get_unchecked(1..);
        let mut last = 0;
        let mut cur = 0;
        loop {
            let x = at(n, cur)?;
            if x == ESCAPE {
                let y = at(n, cur + 1)?;
                if y == ESCAPE {
                    buf.extend(n.get_unchecked(last..cur));
                    buf.push(ESCAPE);
                    cur += 2;
                    last = cur;
                } else if y == b'\"' {
                    buf.extend(n.get_unchecked(last..cur));
                    buf.push(b'\"');
                    cur += 2;
                    last = cur;
                } else {
                    cur += 1;
                }
            } else if x == b'\"' {
                break;
            } else if x < 0x80 {
                cur += 1;
            } else if x < 0xE0 {
                cur += 2;
            } else if x < 0xF0 {
                cur += 3;
            } else {
                cur += 4;
            }
        }
        buf.extend(n.get_unchecked(last..cur));
        *n = match n.get(1 + cur..) {
            Some(x) => x,
            None => return Err(Error),
        };
        Ok(core::str::from_utf8_unchecked(buf.get_unchecked(begin..)))
    }
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
        let (bl, x) = unsafe { bls.pop().unwrap_unchecked() };
        match bl {
            Bl::C(y) => {
                let (name, tag) = match y.get(x) {
                    Some(t) => t,
                    None => {
                        buf.push(b'\n');
                        for _ in 0..bls.len() {
                            buf.extend(SPACE);
                        }
                        buf.push(b'}');
                        if bls.is_empty() {
                            return;
                        }
                        continue;
                    }
                };
                if x != 0 {
                    buf.push(b',');
                }
                buf.push(b'\n');
                bls.push((Bl::C(y), x + 1));
                for _ in 0..bls.len() {
                    buf.extend(SPACE);
                }
                buf.push(b'"');
                buf.extend(name.as_bytes());
                buf.extend(b"\": ");
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
                        buf.push(b'"');
                        buf.extend(x.as_str().as_bytes());
                        buf.push(b'"');
                    }
                    Tag::ByteArray(x) => {
                        buf.extend(b"[B;");
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
                        buf.extend(b"[I;");
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
                        buf.extend(b"[L;");
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
                    buf.extend(b"[B;");
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
                    buf.extend(b"[I;");
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
                    buf.extend(b"[B;");
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
                if let Some(l) = y.get(x) {
                    if x != 0 {
                        buf.extend(DELIMITER);
                    }
                    bls.push((Bl::List(y), x + 1));
                    bls.push((Bl::from(l), 0));
                    buf.push(b'[');
                    continue;
                }
            }
            Bl::Compound(y) => {
                if let Some(l) = y.get(x) {
                    if x != 0 {
                        buf.extend(b",\n");
                        for _ in 0..=bls.len() {
                            buf.extend(SPACE);
                        }
                    }
                    bls.push((Bl::Compound(y), x + 1));
                    bls.push((Bl::C(l.as_ref()), 0));
                    buf.extend(b"{\n");
                    for _ in 0..=bls.len() {
                        buf.extend(SPACE);
                    }
                    continue;
                }
            }
        }
        buf.push(b']');
        if bls.is_empty() {
            return;
        }
    }
}
