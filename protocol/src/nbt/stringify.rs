use super::{Compound, List, Tag};
use crate::str::{SmolStr, StringBuilder};
use crate::{Bytes, Error};
use alloc::vec;
use alloc::vec::Vec;
use core::hint::unreachable_unchecked;
use core::ptr::NonNull;
use mser::{parse_float, parse_int};
use smallvec::SmallVec;

const CAP: usize = 24;

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
        unsafe { decode(&mut n.as_bytes()).map(|(x, _)| Self(x)) }
    }

    #[inline]
    pub fn encode(&self, buf: &mut Vec<u8>) {
        unsafe { encode(buf, NonNull::new_unchecked(&self.0 as *const _ as _)) }
    }
}

enum Block {
    C(NonNull<Compound>),
    L(NonNull<List>),
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

fn darr(n: &mut &[u8]) -> Result<Tag, Error> {
    match n.at(1)? {
        b'B' if n.at(2)? == b';' => unsafe {
            let mut vec = Vec::<u8>::new();

            *n = n.get_unchecked(3..);
            dw(n);
            if n.peek1()? == b']' {
                *n = n.get_unchecked(1..);
                return Ok(Tag::from(vec));
            }
            loop {
                dw(n);
                let (x, len) = match parse_int::<i8>(n) {
                    (_, 0) => match n.peek1()? {
                        b't' | b'T' if n.len() >= 4 => (1, 4),
                        b'f' | b'F' if n.len() >= 5 => (0, 5),
                        _ => return Err(Error),
                    },
                    (a, b) => match n.at(b)? {
                        b'B' | b'b' => (a, b + 1),
                        _ => (a, b),
                    },
                };
                vec.push(x as u8);
                *n = n.get_unchecked(len..);
                dw(n);
                match n.u8()? {
                    b']' => break,
                    b',' => continue,
                    _ => return Err(Error),
                }
            }
            vec.shrink_to_fit();
            Ok(Tag::from(vec))
        },
        b'I' if n.at(2)? == b';' => unsafe {
            let mut vec = Vec::<i32>::new();

            *n = n.get_unchecked(3..);
            dw(n);
            if n.peek1()? == b']' {
                *n = n.get_unchecked(1..);
                return Ok(Tag::IntArray(vec));
            }
            loop {
                dw(n);
                let (x, l) = parse_int::<i32>(n);
                vec.push(x);
                *n = n.get_unchecked(l..);
                dw(n);
                match n.u8()? {
                    b']' => break,
                    b',' => continue,
                    _ => return Err(Error),
                }
            }
            vec.shrink_to_fit();
            Ok(Tag::IntArray(vec))
        },
        b'L' if n.at(2)? == b';' => unsafe {
            let mut vec = Vec::<i64>::new();

            *n = n.get_unchecked(2..);
            dw(n);
            if n.peek1()? == b']' {
                *n = n.get_unchecked(1..);
                return Ok(Tag::LongArray(vec));
            }
            loop {
                dw(n);
                let (a, b) = parse_int::<i64>(n);
                let (x, len) = match n.at(b)? {
                    b'L' | b'l' => (a, b + 1),
                    _ => (a, b),
                };
                vec.push(x);
                *n = n.get_unchecked(len..);
                dw(n);
                match n.u8()? {
                    b']' => break,
                    b',' => continue,
                    _ => return Err(Error),
                }
            }
            vec.shrink_to_fit();
            Ok(Tag::LongArray(vec))
        },
        _ => Err(Error),
    }
}

unsafe fn decode(n: &mut &[u8]) -> Result<(Compound, usize), Error> {
    unsafe {
        let len_start = n.len();
        let mut root = Compound::new();
        let mut ptrs = SmallVec::<[Block; CAP]>::new();
        ptrs.push(Block::C(NonNull::new_unchecked(&mut root as *mut _)));
        let mut on_start = true;
        let mut on_end = false;

        loop {
            let ptr = match ptrs.pop() {
                Some(x) => x,
                None => return Err(Error),
            };
            dw(n);

            if on_start {
                on_start = false;
                if matches!(ptr, Block::C(..)) {
                    if n.u8()? != b'{' {
                        return Err(Error);
                    }
                    dw(n);
                    if n.peek1()? == b'}' {
                        *n = n.get_unchecked(1..);
                        on_end = true;
                    }
                } else {
                    if n.u8()? != b'[' {
                        return Err(Error);
                    }
                    dw(n);
                    if n.peek1()? == b']' {
                        *n = n.get_unchecked(1..);
                        on_end = true;
                    }
                }
            } else if on_end {
            } else if matches!(ptr, Block::C(..)) {
                match n.u8()? {
                    b'}' => on_end = true,
                    b',' => (),
                    _ => return Err(Error),
                }
                dw(n);
            } else {
                match n.u8()? {
                    b']' => on_end = true,
                    b',' => (),
                    _ => return Err(Error),
                }
                dw(n);
            }
            if !on_end {
                if matches!(ptr, Block::C(..)) {
                    if n.peek1()? == b'}' {
                        on_end = true;
                        *n = n.get_unchecked(1..);
                    }
                } else if n.peek1()? == b']' {
                    on_end = true;
                    *n = n.get_unchecked(1..);
                }
            }
            if on_end {
                on_end = false;
                match ptr {
                    Block::C(mut x) => x.as_mut().shrink_to_fit(),
                    Block::L(mut x) => match x.as_mut() {
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
                if ptrs.is_empty() {
                    return Ok((root, len_start - n.len()));
                } else {
                    continue;
                }
            }

            match ptr {
                Block::C(mut c) => {
                    let curr = c.as_mut();
                    let k = match n.peek1()? {
                        b'\"' => dqstr2(n)?,
                        b'\'' => dqstr1(n)?,
                        _ => {
                            let x = find_ascii(n, |x| {
                                matches!(x, b':' | b' ' | b'\n' | b'\t' | b'\r')
                            })?;
                            let m =
                                SmolStr::new(core::str::from_utf8_unchecked(n.get_unchecked(0..x)));
                            *n = n.get_unchecked(x..);
                            m
                        }
                    };
                    dw(n);
                    n.slice(1)?;
                    dw(n);
                    match n.peek1()? {
                        b'{' => {
                            let index = curr.len();
                            curr.push(k, Compound::new());
                            let (_, Tag::Compound(last)) = curr.get_unchecked_mut(index) else {
                                unreachable_unchecked()
                            };
                            ptrs.push(Block::C(c));
                            ptrs.push(Block::C(NonNull::new_unchecked(last)));
                            on_start = true;
                        }
                        b'[' => {
                            if let Ok(arr) = darr(n) {
                                curr.push(k, arr);
                            } else {
                                let index = curr.len();
                                curr.push(k, List::None);
                                let (_, Tag::List(last)) = curr.get_unchecked_mut(index) else {
                                    unreachable_unchecked()
                                };
                                ptrs.push(Block::C(c));
                                ptrs.push(Block::L(NonNull::new_unchecked(last)));
                                on_start = true;
                            }
                        }
                        b'"' => {
                            let s = dqstr2(n)?;
                            curr.push(k, s);
                            ptrs.push(Block::C(c));
                        }
                        b'\'' => {
                            let s = dqstr1(n)?;
                            curr.push(k, s);
                            ptrs.push(Block::C(c));
                        }
                        _ => {
                            let s = n.slice(find_ascii(n, |x| {
                                matches!(x, b',' | b'}' | b' ' | b'\n' | b'\t' | b'\r')
                            })?)?;
                            let v = match dnum(s) {
                                Ok(x) => x,
                                Err(_) => {
                                    Tag::String(SmolStr::new(core::str::from_utf8_unchecked(s)))
                                }
                            };
                            curr.push(k, v);
                            ptrs.push(Block::C(c));
                        }
                    }
                }
                Block::L(mut l) => {
                    let ch = n.peek1()?;
                    if ch == b'{' {
                        if let List::None = l.as_ref() {
                            *l.as_mut() = List::Compound(Vec::new());
                        }
                        let comp = match l.as_mut() {
                            List::Compound(x) => x,
                            _ => return Err(Error),
                        };
                        let index = comp.len();
                        comp.push(Compound::new());
                        let last = comp.get_unchecked_mut(index);
                        ptrs.push(Block::L(l));
                        ptrs.push(Block::C(NonNull::new_unchecked(last as *const _ as _)));
                        on_start = true;
                    } else if ch == b'[' {
                        if let Ok(arr) = darr(n) {
                            match arr {
                                Tag::ByteArray(b) => {
                                    if let List::None = l.as_ref() {
                                        *l.as_mut() = List::ByteArray(Vec::new());
                                    }
                                    match l.as_mut() {
                                        List::ByteArray(x) => x.push(b),
                                        _ => return Err(Error),
                                    }
                                }
                                Tag::IntArray(b) => {
                                    if let List::None = l.as_ref() {
                                        *l.as_mut() = List::IntArray(Vec::new());
                                    }
                                    match l.as_mut() {
                                        List::IntArray(x) => x.push(b),
                                        _ => return Err(Error),
                                    }
                                }
                                Tag::LongArray(b) => {
                                    if let List::None = l.as_ref() {
                                        *l.as_mut() = List::LongArray(Vec::new());
                                    }
                                    match l.as_mut() {
                                        List::LongArray(x) => x.push(b),
                                        _ => return Err(Error),
                                    }
                                }
                                _ => unreachable_unchecked(),
                            }
                            ptrs.push(Block::L(l));
                        } else {
                            if let List::None = l.as_ref() {
                                *l.as_mut() = List::List(Vec::new());
                            }
                            let list = match l.as_mut() {
                                List::List(x) => x,
                                _ => return Err(Error),
                            };
                            let index = list.len();
                            list.push(List::None);
                            let last = list.get_unchecked_mut(index);
                            ptrs.push(Block::L(l));
                            ptrs.push(Block::L(NonNull::new_unchecked(last as *const _ as _)));
                            on_start = true;
                        }
                    } else {
                        let first = n.peek1()?;
                        let tag = match first {
                            b'"' => {
                                let s = dqstr2(n)?;
                                Tag::String(s)
                            }
                            b'\'' => {
                                let s = dqstr1(n)?;
                                Tag::String(s)
                            }
                            _ => {
                                let i = find_ascii(n, |x| {
                                    matches!(x, b',' | b']' | b' ' | b'\n' | b'\t' | b'\r')
                                })?;

                                let s = n.slice(i)?;
                                match dnum(s) {
                                    Ok(x) => x,
                                    Err(_) => {
                                        Tag::String(SmolStr::new(core::str::from_utf8_unchecked(s)))
                                    }
                                }
                            }
                        };
                        if let Tag::Byte(x) = tag {
                            let mut list = vec![x];
                            loop {
                                dw(n);
                                match n.u8()? {
                                    b',' => {
                                        dw(n);
                                        match n.peek1()? {
                                            b'+' | b'-' | b'0'..=b'9' => {
                                                let (a, b) = parse_int::<i8>(n);
                                                *n = n.get_unchecked(b..);
                                                if let b'b' | b'B' = n.peek1()? {
                                                    *n = n.get_unchecked(1..);
                                                }
                                                list.push(a);
                                            }
                                            b't' | b'T' => {
                                                n.slice(4)?;
                                                list.push(1);
                                            }
                                            b'f' | b'F' => {
                                                n.slice(5)?;
                                                list.push(0);
                                            }
                                            b']' => {
                                                *n = n.get_unchecked(1..);
                                                on_end = true;
                                                break;
                                            }
                                            _ => return Err(Error),
                                        }
                                    }
                                    b']' => {
                                        on_end = true;
                                        break;
                                    }
                                    _ => return Err(Error),
                                }
                            }
                            l.replace(List::Byte(list));
                        } else if let Tag::Short(x) = tag {
                            let mut list = vec![x];
                            loop {
                                dw(n);
                                match n.u8()? {
                                    b',' => {
                                        dw(n);
                                        match n.peek1()? {
                                            b'+' | b'-' | b'0'..=b'9' => {}
                                            b']' => {
                                                *n = n.get_unchecked(1..);
                                                on_end = true;
                                                break;
                                            }
                                            _ => return Err(Error),
                                        }
                                        let (a, b) = parse_int::<i16>(n);
                                        *n = n.get_unchecked(b..);
                                        if let b's' | b'S' = n.peek1()? {
                                            *n = n.get_unchecked(1..);
                                        }
                                        list.push(a);
                                    }
                                    b']' => {
                                        on_end = true;
                                        break;
                                    }
                                    _ => return Err(Error),
                                }
                            }
                            l.replace(List::Short(list));
                        } else if let Tag::Int(x) = tag {
                            let mut list = vec![x];
                            loop {
                                dw(n);
                                match n.u8()? {
                                    b',' => {
                                        dw(n);
                                        match n.peek1()? {
                                            b'+' | b'-' | b'0'..=b'9' => {}
                                            b']' => {
                                                *n = n.get_unchecked(1..);
                                                on_end = true;
                                                break;
                                            }
                                            _ => return Err(Error),
                                        }
                                        let (a, b) = parse_int::<i32>(n);
                                        *n = n.get_unchecked(b..);
                                        list.push(a);
                                    }
                                    b']' => {
                                        on_end = true;
                                        break;
                                    }
                                    _ => return Err(Error),
                                }
                            }
                            l.replace(List::Int(list));
                        } else if let Tag::Long(x) = tag {
                            let mut list = vec![x];
                            loop {
                                dw(n);
                                match n.u8()? {
                                    b',' => {
                                        dw(n);
                                        match n.peek1()? {
                                            b'+' | b'-' | b'0'..=b'9' => {}
                                            b']' => {
                                                *n = n.get_unchecked(1..);
                                                on_end = true;
                                                break;
                                            }
                                            _ => return Err(Error),
                                        }
                                        let (a, b) = parse_int::<i64>(n);
                                        *n = n.get_unchecked(b..);
                                        if let b'l' | b'L' = n.peek1()? {
                                            *n = n.get_unchecked(1..);
                                        }
                                        list.push(a);
                                    }
                                    b']' => {
                                        on_end = true;
                                        break;
                                    }
                                    _ => return Err(Error),
                                }
                            }
                            l.replace(List::Long(list));
                        } else if let Tag::Float(x) = tag {
                            let mut list = vec![x];
                            loop {
                                dw(n);
                                match n.u8()? {
                                    b',' => {
                                        dw(n);
                                        match n.peek1()? {
                                            b'+' | b'-' | b'0'..=b'9' | b'.' => {}
                                            b']' => {
                                                *n = n.get_unchecked(1..);
                                                on_end = true;
                                                break;
                                            }
                                            _ => return Err(Error),
                                        }
                                        let (a, b) = parse_float(n);
                                        *n = n.get_unchecked(b..);
                                        if let b'f' | b'F' = n.peek1()? {
                                            *n = n.get_unchecked(1..);
                                        }
                                        list.push(a);
                                    }
                                    b']' => {
                                        on_end = true;
                                        break;
                                    }
                                    _ => return Err(Error),
                                }
                            }
                            l.replace(List::Float(list));
                        } else if let Tag::Double(x) = tag {
                            let mut list = vec![x];
                            loop {
                                dw(n);
                                match n.u8()? {
                                    b',' => {
                                        dw(n);
                                        match n.peek1()? {
                                            b'+' | b'-' | b'0'..=b'9' | b'.' => {}
                                            b']' => {
                                                *n = n.get_unchecked(1..);
                                                on_end = true;
                                                break;
                                            }
                                            _ => return Err(Error),
                                        }
                                        let (a, b) = parse_float(n);
                                        *n = n.get_unchecked(b..);
                                        if let b'd' | b'D' = n.peek1()? {
                                            *n = n.get_unchecked(1..);
                                        }
                                        list.push(a);
                                    }
                                    b']' => {
                                        on_end = true;
                                        break;
                                    }
                                    _ => return Err(Error),
                                }
                            }
                            l.replace(List::Double(list));
                        } else if let Tag::String(x) = tag {
                            let mut list = vec![x];
                            loop {
                                dw(n);
                                match n.u8()? {
                                    b',' => {
                                        dw(n);
                                        match n.peek1()? {
                                            b']' => {
                                                *n = n.get_unchecked(1..);
                                                on_end = true;
                                                break;
                                            }
                                            _ => {
                                                let x = match n.peek1()? {
                                                    b'\"' => dqstr2(n)?,
                                                    b'\'' => dqstr1(n)?,
                                                    _ => {
                                                        let x = find_ascii(n, |x| {
                                                            matches!(
                                                                x,
                                                                b',' | b']'
                                                                    | b' '
                                                                    | b'\n'
                                                                    | b'\t'
                                                                    | b'\r'
                                                            )
                                                        })?;
                                                        let m = SmolStr::new(
                                                            core::str::from_utf8_unchecked(
                                                                n.get_unchecked(0..x),
                                                            ),
                                                        );
                                                        *n = n.get_unchecked(x..);
                                                        m
                                                    }
                                                };
                                                list.push(x);
                                            }
                                        };
                                    }
                                    b']' => {
                                        on_end = true;
                                        break;
                                    }
                                    _ => return Err(Error),
                                }
                            }
                            l.replace(List::String(list));
                        } else {
                            unreachable_unchecked()
                        }
                        ptrs.push(Block::L(l));
                    }
                }
            }
        }
    }
}

fn dnum(n: &[u8]) -> Result<Tag, Error> {
    match n.peek1()? {
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
unsafe fn dqstr1(n: &mut &[u8]) -> Result<SmolStr, Error> {
    unsafe {
        *n = n.get_unchecked(1..);
        let mut buf = StringBuilder::new();
        let mut last = 0;
        let mut cur = 0;
        loop {
            let x = n.at(cur)?;
            if x == ESCAPE {
                let y = n.at(cur + 1)?;
                if y == ESCAPE {
                    buf.extend(core::str::from_utf8_unchecked(n.get_unchecked(last..cur)));
                    buf.push2(ESCAPE);
                    cur += 2;
                    last = cur;
                } else if y == b'\'' {
                    buf.extend(core::str::from_utf8_unchecked(n.get_unchecked(last..cur)));
                    buf.push2(b'\'');
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
        buf.extend(core::str::from_utf8_unchecked(n.get_unchecked(last..cur)));
        n.slice(1 + cur)?;
        Ok(buf.finish())
    }
}

/// decode a double quoted string
unsafe fn dqstr2(n: &mut &[u8]) -> Result<SmolStr, Error> {
    unsafe {
        *n = n.get_unchecked(1..);
        let mut buf = StringBuilder::new();
        let mut last = 0;
        let mut cur = 0;
        loop {
            let x = n.at(cur)?;
            if x == ESCAPE {
                let y = n.at(cur + 1)?;
                if y == ESCAPE {
                    buf.extend(core::str::from_utf8_unchecked(n.get_unchecked(last..cur)));
                    buf.push2(ESCAPE);
                    cur += 2;
                    last = cur;
                } else if y == b'\"' {
                    buf.extend(core::str::from_utf8_unchecked(n.get_unchecked(last..cur)));
                    buf.push2(b'\"');
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
        buf.extend(core::str::from_utf8_unchecked(n.get_unchecked(last..cur)));
        n.slice(1 + cur)?;
        Ok(buf.finish())
    }
}

/// skip whitespace characters
#[inline]
fn dw(n: &mut &[u8]) {
    while let [b' ' | b'\n' | b'\t' | b'\r', rest @ ..] = n {
        *n = rest;
    }
}

const SPACE: &[u8] = b"    ";
const DELIMITER: &[u8] = b", ";

unsafe fn encode(buf: &mut Vec<u8>, n: NonNull<Compound>) {
    let mut itoa_buf = itoa::Buffer::new();
    let mut ryu_buf = ryu::Buffer::new();
    let mut ptrs = SmallVec::<[(Block, usize); CAP]>::new();
    ptrs.push((Block::C(n), 0));

    buf.push(b'{');
    unsafe {
        loop {
            let (ptr, x) = ptrs.pop().unwrap_unchecked();
            match ptr {
                Block::C(y) => {
                    let (name, tag) = match y.as_ref().get(x) {
                        Some(t) => t,
                        None => {
                            buf.push(b'\n');
                            for _ in 0..ptrs.len() {
                                buf.extend(SPACE);
                            }
                            buf.push(b'}');
                            if ptrs.is_empty() {
                                return;
                            }
                            continue;
                        }
                    };
                    if x != 0 {
                        buf.push(b',');
                    }
                    buf.push(b'\n');
                    ptrs.push((Block::C(y), x + 1));
                    for _ in 0..ptrs.len() {
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
                            ptrs.push((Block::L(NonNull::new_unchecked(x as *const _ as _)), 0));
                        }
                        Tag::Compound(x) => {
                            buf.push(b'{');
                            ptrs.push((Block::C(NonNull::new_unchecked(x as *const _ as _)), 0));
                        }
                    }
                }
                Block::L(y) => {
                    match y.as_ref() {
                        List::None => {}
                        List::Byte(x) => {
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
                        List::Short(x) => {
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
                        List::Int(x) => {
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
                        List::Long(x) => {
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
                        List::Float(x) => {
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
                        List::Double(x) => {
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
                        List::String(x) => {
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
                        List::ByteArray(x) => {
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
                        List::IntArray(x) => {
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
                        List::LongArray(x) => {
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
                        List::List(l) => {
                            if let Some(l) = l.get(x) {
                                if x != 0 {
                                    buf.extend(DELIMITER);
                                }
                                ptrs.push((Block::L(y), x + 1));
                                ptrs.push((
                                    Block::L(NonNull::new_unchecked(l as *const _ as _)),
                                    0,
                                ));
                                buf.push(b'[');
                                continue;
                            }
                        }
                        List::Compound(l) => {
                            if let Some(l) = l.get(x) {
                                if x != 0 {
                                    buf.extend(b",\n");
                                    for _ in 0..=ptrs.len() {
                                        buf.extend(SPACE);
                                    }
                                }
                                ptrs.push((Block::L(y), x + 1));
                                ptrs.push((
                                    Block::C(NonNull::new_unchecked(l as *const _ as _)),
                                    0,
                                ));
                                buf.extend(b"{\n");
                                for _ in 0..=ptrs.len() {
                                    buf.extend(SPACE);
                                }
                                continue;
                            }
                        }
                    }
                    buf.push(b']');
                    if ptrs.is_empty() {
                        return;
                    }
                }
            }
        }
    }
}
