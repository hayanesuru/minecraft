use super::{Compound, List, Tag};
use crate::{parse_float, parse_int, Bytes};
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::hint::unreachable_unchecked;
use kstring::KString;

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
        unsafe { decode(&mut n.as_bytes()).map(Self) }
    }

    #[inline]
    pub fn encode(&self, buf: &mut String) {
        unsafe { encode(buf, &self.0) }
    }
}

#[derive(Clone, Copy)]
enum E {
    C(*mut Compound),
    L(*mut List),
}

#[derive(Clone, Copy)]
enum F {
    C(usize, *const Compound),
    L(usize, *const List),
}

#[inline]
fn find(n: &[u8], mut p: impl FnMut(u8) -> bool) -> Option<usize> {
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

unsafe fn decode(n: &mut &[u8]) -> Option<Compound> {
    let mut root = Compound::default();
    let mut ptrs = vec![E::C(&mut root)];
    let mut on_start = true;
    let mut on_end = false;

    loop {
        let ptr = *ptrs.get_unchecked(ptrs.len() - 1);
        dw(n);

        if on_start {
            on_start = false;
            match ptr {
                E::C(_) => {
                    if n.u8()? != b'{' {
                        return None;
                    }
                    dw(n);
                    if *n.first()? == b'}' {
                        *n = n.get_unchecked(1..);
                        on_end = true;
                    }
                }
                E::L(_) => {
                    if n.u8()? != b'[' {
                        return None;
                    }
                    dw(n);
                    if *n.first()? == b']' {
                        *n = n.get_unchecked(1..);
                        on_end = true;
                    }
                }
            }
        } else if on_end
            || match ptr {
                E::C(_) => match n.u8()? {
                    b'}' => true,
                    b',' => false,
                    _ => return None,
                },
                E::L(_) => match n.u8()? {
                    b']' => true,
                    b',' => false,
                    _ => return None,
                },
            }
        {
            on_end = true;
        } else {
            dw(n);
        }
        if !on_end {
            match ptr {
                E::C(_) => {
                    if *n.first()? == b'}' {
                        on_end = true;
                        *n = n.get_unchecked(1..);
                    }
                }
                E::L(_) => {
                    if *n.first()? == b']' {
                        on_end = true;
                        *n = n.get_unchecked(1..);
                    }
                }
            }
        }
        if on_end {
            on_end = false;
            match ptrs.pop()? {
                E::C(x) => {
                    let x = &mut *x;
                    x.shrink_to_fit();
                }
                E::L(x) => {
                    let x = &mut *x;
                    match x {
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
                    }
                }
            }
            if ptrs.is_empty() {
                dw(n);
                return if !n.is_empty() { None } else { Some(root) };
            } else {
                continue;
            }
        }
        match ptr {
            E::C(c) => {
                let c = &mut *c;
                let k = match *n.first()? {
                    b'\"' => dqstr2(n)?,
                    b'\'' => dqstr1(n)?,
                    _ => {
                        let x = find(n, |x| matches!(x, b':' | b' ' | b'\n' | b'\t' | b'\r'))?;
                        let m = unsafe {
                            KString::from_ref(core::str::from_utf8_unchecked(n.get_unchecked(0..x)))
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
                        let index = c.len();
                        c.push(k, Compound::default());
                        let (_, Tag::Compound(last)) = c.get_unchecked_mut(index) else {
                            unreachable_unchecked()
                        };
                        ptrs.push(E::C(last));
                        on_start = true;
                    }
                    b'[' => {
                        if let Some(arr) = darr(n) {
                            c.push(k, arr);
                        } else {
                            on_start = true;
                            let index = c.len();
                            c.push(k, List::None);
                            let (_, Tag::List(last)) = c.get_unchecked_mut(index) else {
                                unreachable_unchecked()
                            };
                            ptrs.push(E::L(last));
                        }
                    }
                    b'"' => {
                        let s = dqstr2(n)?;
                        c.push(k, s);
                    }
                    b'\'' => {
                        let s = dqstr1(n)?;
                        c.push(k, s);
                    }
                    _ => {
                        let s = n
                            .slice(find(n, |x| {
                                matches!(x, b',' | b'}' | b' ' | b'\n' | b'\t' | b'\r')
                            })?)
                            .unwrap_unchecked();
                        let v = match dnum(s) {
                            Some(x) => x,
                            None => {
                                Tag::String(KString::from_ref(core::str::from_utf8_unchecked(s)))
                            }
                        };
                        c.push(k, v);
                    }
                }
            }
            E::L(ptr) => match *n.first()? {
                b'{' => {
                    let l = match &mut *ptr {
                        List::Compound(x) => x,
                        _ => {
                            core::ptr::replace(ptr, List::Compound(Vec::new()));
                            match &mut *ptr {
                                List::Compound(c) => c,
                                _ => unreachable_unchecked(),
                            }
                        }
                    };
                    let index = l.len();
                    l.push(Compound::default());
                    let last = l.get_unchecked_mut(index);
                    ptrs.push(E::C(last));
                    on_start = true;
                }
                b'[' => {
                    if let List::List(l) = &mut *ptr {
                        on_start = true;
                        let index = l.len();
                        l.push(List::None);
                        let last = l.get_unchecked_mut(index);
                        ptrs.push(E::L(last));
                    } else if let Some(arr) = darr(n) {
                        match arr {
                            Tag::ByteArray(b) => {
                                let l = match &mut *ptr {
                                    List::ByteArray(x) => x,
                                    _ => {
                                        core::ptr::replace(ptr, List::ByteArray(Vec::new()));
                                        match &mut *ptr {
                                            List::ByteArray(c) => c,
                                            _ => unreachable_unchecked(),
                                        }
                                    }
                                };
                                l.push(b);
                            }
                            Tag::IntArray(b) => {
                                let l = match &mut *ptr {
                                    List::IntArray(x) => x,
                                    _ => {
                                        core::ptr::replace(ptr, List::IntArray(Vec::new()));
                                        match &mut *ptr {
                                            List::IntArray(c) => c,
                                            _ => unreachable_unchecked(),
                                        }
                                    }
                                };
                                l.push(b);
                            }
                            Tag::LongArray(b) => {
                                let l = match &mut *ptr {
                                    List::LongArray(x) => x,
                                    _ => {
                                        core::ptr::replace(ptr, List::LongArray(Vec::new()));
                                        match &mut *ptr {
                                            List::LongArray(c) => c,
                                            _ => unreachable_unchecked(),
                                        }
                                    }
                                };
                                l.push(b);
                            }
                            _ => unreachable_unchecked(),
                        }
                    } else {
                        core::ptr::replace(ptr, List::List(Vec::new()));
                        let l = match &mut *ptr {
                            List::List(c) => c,
                            _ => unreachable_unchecked(),
                        };
                        on_start = true;
                        let index = l.len();
                        l.push(List::None);
                        let last = l.get_unchecked_mut(index);
                        ptrs.push(E::L(last));
                    }
                }
                f => {
                    let tag = match f {
                        b'"' => {
                            let s = dqstr2(n)?;
                            Tag::String(s)
                        }
                        b'\'' => {
                            let s = dqstr1(n)?;
                            Tag::String(s)
                        }
                        _ => {
                            let i = find(n, |x| {
                                matches!(x, b',' | b']' | b' ' | b'\n' | b'\t' | b'\r')
                            })?;

                            let s = n.slice(i).unwrap_unchecked();
                            match dnum(s) {
                                Some(x) => x,
                                None => Tag::String(KString::from_ref(
                                    core::str::from_utf8_unchecked(s),
                                )),
                            }
                        }
                    };
                    if let Tag::Byte(x) = tag {
                        let mut l = vec![x];
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
                                            l.push(a as u8);
                                        }
                                        b't' | b'T' => {
                                            n.slice(4)?;
                                            l.push(1);
                                        }
                                        b'f' | b'F' => {
                                            n.slice(5)?;
                                            l.push(0);
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
                        core::ptr::replace(ptr, List::from(l));
                    } else if let Tag::Short(x) = tag {
                        let mut l = vec![x];
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
                                    l.push(a);
                                }
                                b']' => {
                                    on_end = true;
                                    break;
                                }
                                _ => return None,
                            }
                        }
                        core::ptr::replace(ptr, List::from(l));
                    } else if let Tag::Int(x) = tag {
                        let mut l = vec![x];
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
                                    l.push(a);
                                }
                                b']' => {
                                    on_end = true;
                                    break;
                                }
                                _ => return None,
                            }
                        }
                        core::ptr::replace(ptr, List::from(l));
                    } else if let Tag::Long(x) = tag {
                        let mut l = vec![x];
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
                                    l.push(a);
                                }
                                b']' => {
                                    on_end = true;
                                    break;
                                }
                                _ => return None,
                            }
                        }
                        core::ptr::replace(ptr, List::from(l));
                    } else if let Tag::Float(x) = tag {
                        let mut l = vec![x];
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
                                    l.push(a);
                                }
                                b']' => {
                                    on_end = true;
                                    break;
                                }
                                _ => return None,
                            }
                        }
                        core::ptr::replace(ptr, List::from(l));
                    } else if let Tag::Double(x) = tag {
                        let mut l = vec![x];
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
                                    l.push(a);
                                }
                                b']' => {
                                    on_end = true;
                                    break;
                                }
                                _ => return None,
                            }
                        }
                        core::ptr::replace(ptr, List::from(l));
                    } else if let Tag::String(x) = tag {
                        let mut l = vec![x];
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
                                                    let x = find(n, |x| {
                                                        matches!(
                                                            x,
                                                            b',' | b']'
                                                                | b' '
                                                                | b'\n'
                                                                | b'\t'
                                                                | b'\r'
                                                        )
                                                    })?;
                                                    let m = KString::from_ref(
                                                        core::str::from_utf8_unchecked(
                                                            n.get_unchecked(0..x),
                                                        ),
                                                    );
                                                    *n = n.get_unchecked(x..);
                                                    m
                                                }
                                            };
                                            l.push(x);
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
                        core::ptr::replace(ptr, List::from(l));
                    } else {
                        unreachable_unchecked()
                    }
                }
            },
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

unsafe fn dqstr1(n: &mut &[u8]) -> Option<KString> {
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
            } else if y == b'\'' {
                k.extend(n.get_unchecked(last..cur));
                k.push(b'\'');
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
    if k.is_empty() {
        let k = KString::from_ref(core::str::from_utf8_unchecked(n.get(last..cur)?));
        *n = n.get(1 + cur..)?;
        Some(k)
    } else {
        k.extend(n.get(last..cur)?);
        k.shrink_to_fit();
        *n = n.get(1 + cur..)?;
        Some(KString::from_string(String::from_utf8_unchecked(k)))
    }
}

unsafe fn dqstr2(n: &mut &[u8]) -> Option<KString> {
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
            } else if y == b'\'' {
                k.extend(n.get_unchecked(last..cur));
                k.push(b'\'');
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
        let k = KString::from_ref(core::str::from_utf8_unchecked(n.get(last..cur)?));
        *n = n.get(1 + cur..)?;
        Some(k)
    } else {
        k.extend(n.get(last..cur)?);
        k.shrink_to_fit();
        *n = n.get(1 + cur..)?;
        Some(KString::from_string(String::from_utf8_unchecked(k)))
    }
}

#[inline]
fn dw(n: &mut &[u8]) {
    while let [b' ' | b'\n' | b'\t' | b'\r', rest @ ..] = n {
        *n = rest;
    }
}

const SPACE: &str = "    ";
const DLIST: &str = ", ";

unsafe fn encode(buf: &mut String, n: *const Compound) {
    let mut itoa_buf = itoa::Buffer::new();
    let mut ryu_buf = ryu::Buffer::new();
    let mut ptrs = vec![F::C(0, n)];
    buf.push_str("{\n");
    buf.push_str(SPACE);

    loop {
        let index = ptrs.len() - 1;
        match ptrs.get_unchecked_mut(index) {
            F::C(x, y) => {
                let y = &**y;
                let (name, tag) = match y.get(*x) {
                    Some(t) => t,
                    None => {
                        buf.push('\n');
                        for _ in 0..index {
                            buf.push_str(SPACE);
                        }
                        buf.push('}');
                        ptrs.pop();
                        if ptrs.is_empty() {
                            return;
                        }
                        continue;
                    }
                };
                if *x != 0 {
                    buf.push_str(",\n");
                    for _ in 0..index + 1 {
                        buf.push_str(SPACE);
                    }
                }
                *x += 1;
                buf.push('"');
                buf.push_str(name);
                buf.push_str("\": ");
                match tag {
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
                        buf.push_str(x);
                        buf.push('"');
                    }
                    Tag::ByteArray(x) => {
                        buf.push_str("[B;");
                        let mut i = 0;
                        while let Some(&y) = x.get(i) {
                            if i != 0 {
                                buf.push_str(DLIST);
                            }
                            i += 1;
                            let s = itoa_buf.format(y as i8);
                            buf.push_str(s);
                            buf.push('b');
                        }
                        buf.push(']');
                    }
                    Tag::IntArray(x) => {
                        buf.push_str("[I;");
                        let mut i = 0;
                        while let Some(&y) = x.get(i) {
                            if i != 0 {
                                buf.push_str(DLIST);
                            }
                            i += 1;
                            let s = itoa_buf.format(y);
                            buf.push_str(s);
                        }
                        buf.push(']');
                    }
                    Tag::LongArray(x) => {
                        buf.push_str("[L;");
                        let mut i = 0;
                        while let Some(&y) = x.get(i) {
                            if i != 0 {
                                buf.push_str(DLIST);
                            }
                            i += 1;
                            let s = itoa_buf.format(y);
                            buf.push_str(s);
                            buf.push('l');
                        }
                        buf.push(']');
                    }
                    Tag::List(x) => {
                        buf.push_str("[\n");
                        for _ in 0..index + 2 {
                            buf.push_str(SPACE);
                        }
                        ptrs.push(F::L(0, x));
                    }
                    Tag::Compound(x) => {
                        buf.push_str("{\n");
                        for _ in 0..index + 2 {
                            buf.push_str(SPACE);
                        }
                        ptrs.push(F::C(0, x));
                    }
                }
            }
            F::L(x, y) => {
                let y = &**y;
                match y {
                    List::None => {}
                    List::Byte(x) => {
                        let mut i = 0;
                        while let Some(&y) = x.get(i) {
                            if i != 0 {
                                buf.push_str(DLIST);
                            }
                            i += 1;
                            let s = itoa_buf.format(y as i8);
                            buf.push_str(s);
                            buf.push('b');
                        }
                    }
                    List::Short(x) => {
                        let mut i = 0;
                        while let Some(&y) = x.get(i) {
                            if i != 0 {
                                buf.push_str(DLIST);
                            }
                            i += 1;
                            let s = itoa_buf.format(y);
                            buf.push_str(s);
                            buf.push('s');
                        }
                    }
                    List::Int(x) => {
                        let mut i = 0;
                        while let Some(&y) = x.get(i) {
                            if i != 0 {
                                buf.push_str(DLIST);
                            }
                            i += 1;
                            let s = itoa_buf.format(y);
                            buf.push_str(s);
                        }
                    }
                    List::Long(x) => {
                        let mut i = 0;
                        while let Some(&y) = x.get(i) {
                            if i != 0 {
                                buf.push_str(DLIST);
                            }
                            i += 1;
                            let s = itoa_buf.format(y);
                            buf.push_str(s);
                            buf.push('l');
                        }
                    }
                    List::Float(x) => {
                        let mut i = 0;
                        while let Some(&y) = x.get(i) {
                            if i != 0 {
                                buf.push_str(DLIST);
                            }
                            i += 1;
                            let s = ryu_buf.format(y);
                            buf.push_str(s);
                            buf.push('f');
                        }
                    }
                    List::Double(x) => {
                        let mut i = 0;
                        while let Some(&y) = x.get(i) {
                            if i != 0 {
                                buf.push_str(DLIST);
                            }
                            i += 1;
                            let s = ryu_buf.format(y);
                            buf.push_str(s);
                            buf.push('d');
                        }
                    }
                    List::String(x) => {
                        let mut i = 0;
                        while let Some(y) = x.get(i) {
                            if i != 0 {
                                buf.push_str(DLIST);
                            }
                            i += 1;
                            buf.push('"');
                            buf.push_str(y);
                            buf.push('"');
                        }
                    }
                    List::ByteArray(x) => {
                        let mut i = 0;
                        while let Some(y) = x.get(i) {
                            if i != 0 {
                                buf.push_str(DLIST);
                            }
                            i += 1;
                            buf.push_str("[B;");
                            let mut j = 0;

                            while let Some(&z) = y.get(j) {
                                if j != 0 {
                                    buf.push_str(DLIST);
                                }
                                j += 1;
                                let s = itoa_buf.format(z as i8);
                                buf.push_str(s);
                                buf.push('b');
                            }
                            buf.push(']');
                        }
                    }
                    List::IntArray(x) => {
                        let mut i = 0;
                        while let Some(y) = x.get(i) {
                            if i != 0 {
                                buf.push_str(DLIST);
                            }
                            i += 1;
                            buf.push_str("[I;");
                            let mut j = 0;

                            while let Some(&z) = y.get(j) {
                                if j != 0 {
                                    buf.push_str(DLIST);
                                }
                                j += 1;
                                let s = itoa_buf.format(z);
                                buf.push_str(s);
                            }
                            buf.push(']');
                        }
                    }
                    List::LongArray(x) => {
                        let mut i = 0;
                        while let Some(y) = x.get(i) {
                            if i != 0 {
                                buf.push_str(DLIST);
                            }
                            i += 1;
                            buf.push_str("[L;");
                            let mut j = 0;

                            while let Some(&z) = y.get(j) {
                                if j != 0 {
                                    buf.push_str(DLIST);
                                }
                                j += 1;
                                let s = itoa_buf.format(z);
                                buf.push_str(s);
                                buf.push('l');
                            }
                            buf.push(']');
                        }
                    }
                    List::List(l) => {
                        if let Some(l) = l.get(*x) {
                            if *x != 0 {
                                buf.push(',');
                            }
                            *x += 1;
                            ptrs.push(F::L(0, l));
                            buf.push('[');
                            continue;
                        }
                    }
                    List::Compound(l) => {
                        if let Some(l) = l.get(*x) {
                            if *x != 0 {
                                buf.push_str(",\n");
                                for _ in 0..index + 1 {
                                    buf.push_str(SPACE);
                                }
                            }
                            *x += 1;
                            ptrs.push(F::C(0, l));
                            buf.push_str("{\n");
                            for _ in 0..index + 2 {
                                buf.push_str(SPACE);
                            }
                            continue;
                        }
                    }
                }
                buf.push('\n');
                for _ in 0..index {
                    buf.push_str(SPACE);
                }
                buf.push(']');
                ptrs.pop();
                if ptrs.is_empty() {
                    return;
                }
            }
        }
    }
}
