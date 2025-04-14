use super::{Compound, List, Tag};
use crate::{parse_float, parse_int, Bytes};
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::hint::unreachable_unchecked;
use core::ptr::NonNull;
use smallvec::SmallVec;
use smol_str::{SmolStr, SmolStrBuilder};

const CAP: usize = 32;

#[derive(Clone, Default)]
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
    pub fn decode(n: &str) -> Option<Self> {
        unsafe { decode(&mut n.as_bytes()).map(|(x, _)| Self(x)) }
    }

    #[inline]
    pub fn encode(&self, buf: &mut String) {
        unsafe { encode(buf, NonNull::new_unchecked(&self.0 as *const _ as _)) }
    }
}

type E = ptr_union::Union2<NonNull<Compound>, NonNull<List>>;

#[inline]
fn find_ascii(n: &[u8], mut p: impl FnMut(u8) -> bool) -> Option<usize> {
    let mut index = 0;
    while let Some(&byte) = n.get(index) {
        match byte {
            x @ 0..=0x7F => {
                if p(x) {
                    return Some(index);
                }
                index += 1
            }
            0x80..=0xDF => index += 2,
            0xE0..=0xEF => index += 3,
            _ => index += 4,
        }
    }
    None
}

fn darr(n: &mut &[u8]) -> Option<Tag> {
    match n.get(1)? {
        b'B' if *n.get(2)? == b';' => unsafe {
            let mut vec = Vec::<u8>::new();

            *n = n.get_unchecked(3..);
            dw(n);
            if *n.first()? == b']' {
                *n = n.get_unchecked(1..);
                return Some(Tag::ByteArray(vec));
            }
            loop {
                dw(n);
                let (x, len) = match parse_int::<i8>(n) {
                    (_, 0) => match *n.first()? {
                        b't' | b'T' if n.len() >= 4 => (1, 4),
                        b'f' | b'F' if n.len() >= 5 => (0, 5),
                        _ => return None,
                    },
                    (a, b) => match *n.get(b)? {
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
                    _ => return None,
                }
            }
            vec.shrink_to_fit();
            Some(Tag::ByteArray(vec))
        },
        b'I' if *n.get(2)? == b';' => unsafe {
            let mut vec = Vec::<i32>::new();

            *n = n.get_unchecked(3..);
            dw(n);
            if *n.first()? == b']' {
                *n = n.get_unchecked(1..);
                return Some(Tag::IntArray(vec));
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
                    _ => return None,
                }
            }
            vec.shrink_to_fit();
            Some(Tag::IntArray(vec))
        },
        b'L' if *n.get(2)? == b';' => unsafe {
            let mut vec = Vec::<i64>::new();

            *n = n.get_unchecked(2..);
            dw(n);
            if *n.first()? == b']' {
                *n = n.get_unchecked(1..);
                return Some(Tag::LongArray(vec));
            }
            loop {
                dw(n);
                let (a, b) = parse_int::<i64>(n);
                let (x, len) = match *n.get(b)? {
                    b'L' | b'l' => (a, b + 1),
                    _ => (a, b),
                };
                vec.push(x);
                *n = n.get_unchecked(len..);
                dw(n);
                match n.u8()? {
                    b']' => break,
                    b',' => continue,
                    _ => return None,
                }
            }
            vec.shrink_to_fit();
            Some(Tag::LongArray(vec))
        },
        _ => None,
    }
}

unsafe fn decode(n: &mut &[u8]) -> Option<(Compound, usize)> {
    let len_start = n.len();
    let mut root = Compound::default();
    let mut ptrs = SmallVec::<[E; CAP]>::new();
    ptrs.push(E::new_a(NonNull::new_unchecked(&mut root as *mut _)).unwrap_unchecked());
    let mut on_start = true;
    let mut on_end = false;

    loop {
        let ptr = ptrs.pop()?;
        dw(n);

        if on_start {
            on_start = false;
            if ptr.is_a() {
                if n.u8()? != b'{' {
                    return None;
                }
                dw(n);
                if *n.first()? == b'}' {
                    *n = n.get_unchecked(1..);
                    on_end = true;
                }
            } else {
                if n.u8()? != b'[' {
                    return None;
                }
                dw(n);
                if *n.first()? == b']' {
                    *n = n.get_unchecked(1..);
                    on_end = true;
                }
            }
        } else if on_end {
        } else if ptr.is_a() {
            match n.u8()? {
                b'}' => on_end = true,
                b',' => (),
                _ => return None,
            }
            dw(n);
        } else {
            match n.u8()? {
                b']' => on_end = true,
                b',' => (),
                _ => return None,
            }
            dw(n);
        }
        if !on_end {
            if ptr.is_a() {
                if *n.first()? == b'}' {
                    on_end = true;
                    *n = n.get_unchecked(1..);
                }
            } else if *n.first()? == b']' {
                on_end = true;
                *n = n.get_unchecked(1..);
            }
        }
        if on_end {
            on_end = false;
            match ptr.into_a() {
                Ok(mut x) => x.as_mut().shrink_to_fit(),
                Err(e) => match e.into_b().unwrap_unchecked().as_mut() {
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
                return Some((root, len_start - n.len()));
            } else {
                continue;
            }
        }

        match ptr.into_a() {
            Ok(mut c) => {
                let curr = c.as_mut();
                let k = match *n.first()? {
                    b'\"' => dqstr2(n)?,
                    b'\'' => dqstr1(n)?,
                    _ => {
                        let x =
                            find_ascii(n, |x| matches!(x, b':' | b' ' | b'\n' | b'\t' | b'\r'))?;
                        let m = unsafe {
                            SmolStr::new(core::str::from_utf8_unchecked(n.get_unchecked(0..x)))
                        };
                        *n = n.get_unchecked(x..);
                        m
                    }
                };
                dw(n);
                *n = n.get(1..)?;
                dw(n);
                match *n.first()? {
                    b'{' => {
                        let index = curr.len();
                        curr.push(k, Compound::default());
                        let (_, Tag::Compound(last)) = curr.get_unchecked_mut(index) else {
                            unreachable_unchecked()
                        };
                        ptrs.push(E::new_a(c).unwrap_unchecked());
                        ptrs.push(E::new_a(NonNull::new_unchecked(last)).unwrap_unchecked());
                        on_start = true;
                    }
                    b'[' => {
                        if let Some(arr) = darr(n) {
                            curr.push(k, arr);
                        } else {
                            let index = curr.len();
                            curr.push(k, List::None);
                            let (_, Tag::List(last)) = curr.get_unchecked_mut(index) else {
                                unreachable_unchecked()
                            };
                            ptrs.push(E::new_a(c).unwrap_unchecked());
                            ptrs.push(E::new_b(NonNull::new_unchecked(last)).unwrap_unchecked());
                            on_start = true;
                        }
                    }
                    b'"' => {
                        let s = dqstr2(n)?;
                        curr.push(k, s);
                        ptrs.push(E::new_a(c).unwrap_unchecked());
                    }
                    b'\'' => {
                        let s = dqstr1(n)?;
                        curr.push(k, s);
                        ptrs.push(E::new_a(c).unwrap_unchecked());
                    }
                    _ => {
                        let s = n
                            .slice(find_ascii(n, |x| {
                                matches!(x, b',' | b'}' | b' ' | b'\n' | b'\t' | b'\r')
                            })?)
                            .unwrap_unchecked();
                        let v = match dnum(s) {
                            Some(x) => x,
                            None => Tag::String(SmolStr::new(core::str::from_utf8_unchecked(s))),
                        };
                        curr.push(k, v);
                        ptrs.push(E::new_a(c).unwrap_unchecked());
                    }
                }
            }
            Err(l) => {
                let mut l = l.into_b().unwrap_unchecked();
                let ch = *n.first()?;
                if ch == b'{' {
                    if let List::None = l.as_ref() {
                        *l.as_mut() = List::Compound(Vec::new());
                    }
                    let comp = match l.as_mut() {
                        List::Compound(x) => x,
                        _ => return None,
                    };
                    let index = comp.len();
                    comp.push(Compound::default());
                    let last = comp.get_unchecked_mut(index);
                    ptrs.push(E::new_b(l).unwrap_unchecked());
                    ptrs.push(
                        E::new_a(NonNull::new_unchecked(last as *const _ as _)).unwrap_unchecked(),
                    );
                    on_start = true;
                } else if ch == b'[' {
                    if let Some(arr) = darr(n) {
                        match arr {
                            Tag::ByteArray(b) => {
                                if let List::None = l.as_ref() {
                                    *l.as_mut() = List::ByteArray(Vec::new());
                                }
                                match l.as_mut() {
                                    List::ByteArray(x) => x.push(b),
                                    _ => return None,
                                }
                            }
                            Tag::IntArray(b) => {
                                if let List::None = l.as_ref() {
                                    *l.as_mut() = List::IntArray(Vec::new());
                                }
                                match l.as_mut() {
                                    List::IntArray(x) => x.push(b),
                                    _ => return None,
                                }
                            }
                            Tag::LongArray(b) => {
                                if let List::None = l.as_ref() {
                                    *l.as_mut() = List::LongArray(Vec::new());
                                }
                                match l.as_mut() {
                                    List::LongArray(x) => x.push(b),
                                    _ => return None,
                                }
                            }
                            _ => unreachable_unchecked(),
                        }
                        ptrs.push(E::new_b(l).unwrap_unchecked());
                    } else {
                        if let List::None = l.as_ref() {
                            *l.as_mut() = List::List(Vec::new());
                        }
                        let list = match l.as_mut() {
                            List::List(x) => x,
                            _ => return None,
                        };
                        let index = list.len();
                        list.push(List::None);
                        let last = list.get_unchecked_mut(index);
                        ptrs.push(E::new_b(l).unwrap_unchecked());
                        ptrs.push(
                            E::new_b(NonNull::new_unchecked(last as *const _ as _))
                                .unwrap_unchecked(),
                        );
                        on_start = true;
                    }
                } else {
                    let first = *n.first()?;
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

                            let s = n.slice(i).unwrap_unchecked();
                            match dnum(s) {
                                Some(x) => x,
                                None => {
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
                                    match *n.first()? {
                                        b'+' | b'-' | b'0'..=b'9' => {
                                            let (a, b) = parse_int::<i8>(n);
                                            *n = n.get_unchecked(b..);
                                            if let b'b' | b'B' = n.first()? {
                                                *n = n.get_unchecked(1..);
                                            }
                                            list.push(a as u8);
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
                                        _ => return None,
                                    }
                                }
                                b']' => {
                                    on_end = true;
                                    break;
                                }
                                _ => return None,
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
                                    match *n.first()? {
                                        b'+' | b'-' | b'0'..=b'9' => {}
                                        b']' => {
                                            *n = n.get_unchecked(1..);
                                            on_end = true;
                                            break;
                                        }
                                        _ => return None,
                                    }
                                    let (a, b) = parse_int::<i16>(n);
                                    *n = n.get_unchecked(b..);
                                    if let b's' | b'S' = n.first()? {
                                        *n = n.get_unchecked(1..);
                                    }
                                    list.push(a);
                                }
                                b']' => {
                                    on_end = true;
                                    break;
                                }
                                _ => return None,
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
                                    match *n.first()? {
                                        b'+' | b'-' | b'0'..=b'9' => {}
                                        b']' => {
                                            *n = n.get_unchecked(1..);
                                            on_end = true;
                                            break;
                                        }
                                        _ => return None,
                                    }
                                    let (a, b) = parse_int::<i32>(n);
                                    *n = n.get_unchecked(b..);
                                    list.push(a);
                                }
                                b']' => {
                                    on_end = true;
                                    break;
                                }
                                _ => return None,
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
                                    match *n.first()? {
                                        b'+' | b'-' | b'0'..=b'9' => {}
                                        b']' => {
                                            *n = n.get_unchecked(1..);
                                            on_end = true;
                                            break;
                                        }
                                        _ => return None,
                                    }
                                    let (a, b) = parse_int::<i64>(n);
                                    *n = n.get_unchecked(b..);
                                    if let b'l' | b'L' = n.first()? {
                                        *n = n.get_unchecked(1..);
                                    }
                                    list.push(a);
                                }
                                b']' => {
                                    on_end = true;
                                    break;
                                }
                                _ => return None,
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
                                    match *n.first()? {
                                        b'+' | b'-' | b'0'..=b'9' | b'.' => {}
                                        b']' => {
                                            *n = n.get_unchecked(1..);
                                            on_end = true;
                                            break;
                                        }
                                        _ => return None,
                                    }
                                    let (a, b) = parse_float(n);
                                    *n = n.get_unchecked(b..);
                                    if let b'f' | b'F' = n.first()? {
                                        *n = n.get_unchecked(1..);
                                    }
                                    list.push(a);
                                }
                                b']' => {
                                    on_end = true;
                                    break;
                                }
                                _ => return None,
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
                                    match *n.first()? {
                                        b'+' | b'-' | b'0'..=b'9' | b'.' => {}
                                        b']' => {
                                            *n = n.get_unchecked(1..);
                                            on_end = true;
                                            break;
                                        }
                                        _ => return None,
                                    }
                                    let (a, b) = parse_float(n);
                                    *n = n.get_unchecked(b..);
                                    if let b'd' | b'D' = n.first()? {
                                        *n = n.get_unchecked(1..);
                                    }
                                    list.push(a);
                                }
                                b']' => {
                                    on_end = true;
                                    break;
                                }
                                _ => return None,
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
                                    match *n.first()? {
                                        b']' => {
                                            *n = n.get_unchecked(1..);
                                            on_end = true;
                                            break;
                                        }
                                        _ => {
                                            let x = match n.first()? {
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
                                _ => return None,
                            }
                        }
                        l.replace(List::String(list));
                    } else {
                        unreachable_unchecked()
                    }
                    ptrs.push(E::new_b(l).unwrap_unchecked());
                }
            }
        }
    }
}

fn dnum(n: &[u8]) -> Option<Tag> {
    match *n.first()? {
        b'+' | b'-' | b'0'..=b'9' | b'.' => (),
        b't' | b'T' => unsafe {
            return match n.get_unchecked(1..) {
                [b'r' | b'R', b'u' | b'U', b'e' | b'E'] => Some(Tag::Byte(1)),
                _ => None,
            };
        },
        b'f' | b'F' => unsafe {
            return match n.get_unchecked(1..) {
                [b'a' | b'A', b'l' | b'L', b's' | b'S', b'e' | b'E'] => Some(Tag::Byte(0)),
                _ => None,
            };
        },
        _ => return None,
    }

    if let [rest @ .., a] = n {
        match *a {
            b'B' | b'b' => {
                let (a, b) = parse_int::<i8>(rest);
                if b != rest.len() {
                    None
                } else {
                    Some(Tag::Byte(a as u8))
                }
            }
            b'S' | b's' => {
                let (a, b) = parse_int::<i16>(rest);
                if b != rest.len() {
                    None
                } else {
                    Some(Tag::Short(a))
                }
            }
            b'L' | b'l' => {
                let (a, b) = parse_int::<i64>(rest);
                if b != rest.len() {
                    None
                } else {
                    Some(Tag::Long(a))
                }
            }
            b'F' | b'f' => {
                let (a, b) = parse_float(rest);
                if b != rest.len() {
                    None
                } else {
                    Some(Tag::Float(a))
                }
            }
            b'D' | b'd' => {
                let (a, b) = parse_float(rest);
                if b != rest.len() {
                    None
                } else {
                    Some(Tag::Double(a))
                }
            }
            _ => unsafe {
                if n.get_unchecked(1..).iter().all(|x| x.is_ascii_digit()) {
                    let (a, b) = parse_int::<i32>(n);
                    if b != n.len() {
                        None
                    } else {
                        Some(Tag::Int(a))
                    }
                } else {
                    let (a, b) = parse_float(n);
                    if b != n.len() {
                        None
                    } else {
                        Some(Tag::Double(a))
                    }
                }
            },
        }
    } else {
        None
    }
}

const ESCAPE: u8 = b'\\';

/// decode single quoted string
unsafe fn dqstr1(n: &mut &[u8]) -> Option<SmolStr> {
    *n = n.get_unchecked(1..);
    let mut buf = SmolStrBuilder::new();
    let mut last = 0;
    let mut cur = 0;
    loop {
        let x = *n.get(cur)?;
        if x == ESCAPE {
            let y = *n.get(cur + 1)?;
            if y == ESCAPE {
                buf.push_str(core::str::from_utf8_unchecked(n.get_unchecked(last..cur)));
                buf.push(ESCAPE as char);
                cur += 2;
                last = cur;
            } else if y == b'\'' {
                buf.push_str(core::str::from_utf8_unchecked(n.get_unchecked(last..cur)));
                buf.push('\'');
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
    buf.push_str(core::str::from_utf8_unchecked(n.get_unchecked(last..cur)));
    *n = n.get(1 + cur..)?;
    Some(buf.finish())
}

/// decode a double quoted string
unsafe fn dqstr2(n: &mut &[u8]) -> Option<SmolStr> {
    *n = n.get_unchecked(1..);
    let mut k = Vec::<u8>::new();
    let mut last = 0;
    let mut cur = 0;
    loop {
        let x = *n.get(cur)?;
        if x == b'\\' {
            let y = *n.get(cur + 1)?;
            if y == b'\\' {
                k.extend(n.get_unchecked(last..cur));
                k.push(b'\\');
                cur += 2;
                last = cur;
            } else if y == b'\"' {
                k.extend(n.get_unchecked(last..cur));
                k.push(b'\"');
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
    if k.is_empty() {
        let k = SmolStr::new(core::str::from_utf8_unchecked(n.get(last..cur)?));
        *n = n.get(1 + cur..)?;
        Some(k)
    } else {
        k.extend(n.get(last..cur)?);
        k.shrink_to_fit();
        *n = n.get(1 + cur..)?;
        Some(SmolStr::new(core::str::from_utf8_unchecked(&k)))
    }
}

/// skip whitespace characters
#[inline]
fn dw(n: &mut &[u8]) {
    while let [b' ' | b'\n' | b'\t' | b'\r', rest @ ..] = n {
        *n = rest;
    }
}

const SPACE: &str = "    ";
const DELIMITER: &str = ", ";

unsafe fn encode(buf: &mut String, n: NonNull<Compound>) {
    let mut itoa_buf = itoa::Buffer::new();
    let mut ryu_buf = ryu::Buffer::new();
    let mut ptrs = SmallVec::<[E; CAP]>::new();
    let mut idxs = SmallVec::<[usize; CAP]>::new();
    ptrs.push(E::new_a(n).unwrap_unchecked());
    idxs.push(0);

    buf.push('{');
    loop {
        let ptr = ptrs.pop().unwrap_unchecked();
        let x = idxs.pop().unwrap_unchecked();
        match ptr.into_a() {
            Ok(y) => {
                let (name, tag) = match y.as_ref().get(x) {
                    Some(t) => t,
                    None => {
                        buf.push('\n');
                        for _ in 0..ptrs.len() {
                            buf.push_str(SPACE);
                        }
                        buf.push('}');
                        if ptrs.is_empty() {
                            return;
                        }
                        continue;
                    }
                };
                if x != 0 {
                    buf.push(',');
                }
                buf.push('\n');
                ptrs.push(E::new_a(y).unwrap_unchecked());
                idxs.push(x + 1);
                for _ in 0..ptrs.len() {
                    buf.push_str(SPACE);
                }
                buf.push('"');
                buf.push_str(name);
                buf.push_str("\": ");
                match &tag {
                    Tag::Byte(x) => {
                        let s = itoa_buf.format(*x as i8);
                        buf.push_str(s);
                        buf.push('B');
                    }
                    Tag::Short(x) => {
                        let s = itoa_buf.format(*x);
                        buf.push_str(s);
                        buf.push('S');
                    }
                    Tag::Int(x) => {
                        let s = itoa_buf.format(*x);
                        buf.push_str(s);
                    }
                    Tag::Long(x) => {
                        let s = itoa_buf.format(*x);
                        buf.push_str(s);
                        buf.push('L');
                    }
                    Tag::Float(x) => {
                        let s = ryu_buf.format(*x);
                        buf.push_str(s);
                        buf.push('F');
                    }
                    Tag::Double(x) => {
                        let s = ryu_buf.format(*x);
                        buf.push_str(s);
                        buf.push('D');
                    }
                    Tag::String(x) => {
                        buf.push('"');
                        buf.push_str(x.as_str());
                        buf.push('"');
                    }
                    Tag::ByteArray(x) => {
                        buf.push_str("[B;");
                        let mut flag = false;
                        for &y in x {
                            if flag {
                                buf.push_str(DELIMITER);
                            }
                            flag = true;
                            let s = itoa_buf.format(y as i8);
                            buf.push_str(s);
                            buf.push('b');
                        }
                        buf.push(']');
                    }
                    Tag::IntArray(x) => {
                        buf.push_str("[I;");
                        let mut flag = false;
                        for &y in x {
                            if flag {
                                buf.push_str(DELIMITER);
                            }
                            flag = true;
                            let s = itoa_buf.format(y);
                            buf.push_str(s);
                        }
                        buf.push(']');
                    }
                    Tag::LongArray(x) => {
                        buf.push_str("[L;");
                        let mut flag = false;
                        for &y in x {
                            if flag {
                                buf.push_str(DELIMITER);
                            }
                            flag = true;
                            let s = itoa_buf.format(y);
                            buf.push_str(s);
                            buf.push('l');
                        }
                        buf.push(']');
                    }
                    Tag::List(x) => {
                        buf.push('[');
                        ptrs.push(
                            E::new_b(NonNull::new_unchecked(x as *const _ as _)).unwrap_unchecked(),
                        );
                        idxs.push(0);
                    }
                    Tag::Compound(x) => {
                        buf.push('{');
                        ptrs.push(
                            E::new_a(NonNull::new_unchecked(x as *const _ as _)).unwrap_unchecked(),
                        );
                        idxs.push(0);
                    }
                }
            }
            Err(y) => {
                let y = y.into_b().unwrap_unchecked();
                match y.as_ref() {
                    List::None => {}
                    List::Byte(x) => {
                        let mut flag = false;
                        for &y in x {
                            if flag {
                                buf.push_str(DELIMITER);
                            }
                            flag = true;
                            let s = itoa_buf.format(y as i8);
                            buf.push_str(s);
                            buf.push('b');
                        }
                    }
                    List::Short(x) => {
                        let mut flag = false;
                        for &y in x {
                            if flag {
                                buf.push_str(DELIMITER);
                            }
                            flag = true;
                            let s = itoa_buf.format(y);
                            buf.push_str(s);
                            buf.push('s');
                        }
                    }
                    List::Int(x) => {
                        let mut flag = false;
                        for &y in x {
                            if flag {
                                buf.push_str(DELIMITER);
                            }
                            flag = true;
                            let s = itoa_buf.format(y);
                            buf.push_str(s);
                        }
                    }
                    List::Long(x) => {
                        let mut flag = false;
                        for &y in x {
                            if flag {
                                buf.push_str(DELIMITER);
                            }
                            flag = true;
                            let s = itoa_buf.format(y);
                            buf.push_str(s);
                            buf.push('l');
                        }
                    }
                    List::Float(x) => {
                        let mut flag = false;
                        for &y in x {
                            if flag {
                                buf.push_str(DELIMITER);
                            }
                            flag = true;
                            let s = ryu_buf.format(y);
                            buf.push_str(s);
                            buf.push('f');
                        }
                    }
                    List::Double(x) => {
                        let mut flag = false;
                        for &y in x {
                            if flag {
                                buf.push_str(DELIMITER);
                            }
                            flag = true;
                            let s = ryu_buf.format(y);
                            buf.push_str(s);
                            buf.push('d');
                        }
                    }
                    List::String(x) => {
                        let mut flag = false;
                        for y in x {
                            if flag {
                                buf.push_str(DELIMITER);
                            }
                            flag = true;
                            buf.push('"');
                            buf.push_str(y);
                            buf.push('"');
                        }
                    }
                    List::ByteArray(x) => {
                        let mut flag = false;
                        for y in x {
                            if flag {
                                buf.push_str(DELIMITER);
                            }
                            flag = true;
                            buf.push_str("[B;");
                            let mut flag1 = false;
                            for &z in y {
                                if flag1 {
                                    buf.push_str(DELIMITER);
                                }
                                flag1 = true;
                                let s = itoa_buf.format(z as i8);
                                buf.push_str(s);
                                buf.push('b');
                            }
                            buf.push(']');
                        }
                    }
                    List::IntArray(x) => {
                        let mut flag = false;
                        for y in x {
                            if flag {
                                buf.push_str(DELIMITER);
                            }
                            flag = true;
                            buf.push_str("[I;");
                            let mut flag1 = false;
                            for &z in y {
                                if flag1 {
                                    buf.push_str(DELIMITER);
                                }
                                flag1 = true;
                                let s = itoa_buf.format(z);
                                buf.push_str(s);
                            }
                            buf.push(']');
                        }
                    }
                    List::LongArray(x) => {
                        let mut flag = false;
                        for y in x {
                            if flag {
                                buf.push_str(DELIMITER);
                            }
                            flag = true;
                            buf.push_str("[B;");
                            let mut flag1 = false;
                            for &z in y {
                                if flag1 {
                                    buf.push_str(DELIMITER);
                                }
                                flag1 = true;
                                let s = itoa_buf.format(z);
                                buf.push_str(s);
                                buf.push('l');
                            }
                            buf.push(']');
                        }
                    }
                    List::List(l) => {
                        if let Some(l) = l.get(x) {
                            if x != 0 {
                                buf.push_str(DELIMITER);
                            }
                            ptrs.push(E::new_b(y).unwrap_unchecked());
                            idxs.push(x + 1);
                            ptrs.push(
                                E::new_b(NonNull::new_unchecked(l as *const _ as _))
                                    .unwrap_unchecked(),
                            );
                            idxs.push(0);
                            buf.push('[');
                            continue;
                        }
                    }
                    List::Compound(l) => {
                        if let Some(l) = l.get(x) {
                            if x != 0 {
                                buf.push_str(",\n");
                                for _ in 0..=ptrs.len() {
                                    buf.push_str(SPACE);
                                }
                            }
                            ptrs.push(E::new_b(y).unwrap_unchecked());
                            idxs.push(x + 1);
                            ptrs.push(
                                E::new_a(NonNull::new_unchecked(l as *const _ as _))
                                    .unwrap_unchecked(),
                            );
                            idxs.push(0);
                            buf.push_str("{\n");
                            for _ in 0..=ptrs.len() {
                                buf.push_str(SPACE);
                            }
                            continue;
                        }
                    }
                }
                buf.push(']');
                if ptrs.is_empty() {
                    return;
                }
            }
        }
    }
}
