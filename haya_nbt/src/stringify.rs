use crate::list::ListPrimitive;
use crate::number::dec_num;
use crate::{
    Compound, CompoundStringify, Error, ListTag, Name, Read as _, Tag, TagArray, TagPrimitive,
};
use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use core::str::from_utf8_unchecked;
use haya_str::{hex_to_u8, u8_to_hex};
use mser::Reader;

const BYTE_ARRAY_PREFIX: &[u8; 3] = b"[B;";
const INT_ARRAY_PREFIX: &[u8; 3] = b"[I;";
const LONG_ARRAY_PREFIX: &[u8; 3] = b"[L;";

impl From<Compound> for CompoundStringify {
    #[inline]
    fn from(value: Compound) -> Self {
        Self(value)
    }
}

impl CompoundStringify {
    #[inline]
    pub fn decode(n: &str) -> Result<Self, Error> {
        unsafe { decode(&mut Reader::new(n.as_bytes()), 512).map(Self) }
    }

    #[inline]
    pub fn encode(&self, buf: &mut Vec<u8>) {
        encode(buf, &self.0)
    }
}

fn find_next_value<'a>(n: &mut Reader<'a>) -> Result<&'a [u8], Error> {
    let i = n.position(b",]} \n\t\r");
    if n.end_from(i) {
        Err(Error)
    } else {
        unsafe { Ok(n.read_slice_from(i)) }
    }
}

fn dec_arr_peek(n: &mut Reader, tmp: &mut TBuf) -> Result<TagArray, Error> {
    match n.peek_array::<2>()? {
        [b'B', b';'] => unsafe {
            n.advance(2);
            skip_ws(n);

            let mut vec = Vec::new();
            if b']' == n.peek_byte()? {
                n.advance(1);
                return Ok(TagArray::Byte(vec));
            }
            loop {
                skip_ws(n);
                let value = find_next_value(n)?;
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
        },
        [b'I', b';'] => unsafe {
            n.advance(2);
            skip_ws(n);

            let mut vec = Vec::new();
            if b']' == n.peek_byte()? {
                n.advance(1);
                return Ok(TagArray::Int(vec));
            }
            loop {
                skip_ws(n);
                let value = find_next_value(n)?;
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
        },
        [b'L', b';'] => unsafe {
            n.advance(2);
            skip_ws(n);

            let mut vec = Vec::new();
            if b']' == n.peek_byte()? {
                n.advance(1);
                return Ok(TagArray::Long(vec));
            }
            loop {
                skip_ws(n);
                let value = find_next_value(n)?;
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
        },
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

unsafe fn decode(n: &mut Reader, max_depth: usize) -> Result<Compound, Error> {
    enum Bl {
        C(Compound),
        L(ListTag),
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
            match bl {
                Bl::C(..) => {
                    if b'}' == n.peek_byte()? {
                        unsafe { n.advance(1) }
                        on_end = true;
                    }
                }
                Bl::L(..) => {
                    if b']' == n.peek_byte()? {
                        unsafe { n.advance(1) }
                        on_end = true;
                    }
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
            match bl {
                Bl::C(..) => {
                    if b'}' == n.peek_byte()? {
                        unsafe { n.advance(1) }
                        on_end = true;
                    }
                }
                Bl::L(..) => {
                    if b']' == n.peek_byte()? {
                        unsafe { n.advance(1) }
                        on_end = true;
                    }
                }
            }
        }
        if on_end {
            on_end = false;
            match &mut bl {
                Bl::C(x) => x.shrink_to_fit(),
                Bl::L(x) => match x {
                    ListTag::List(x) => x.shrink_to_fit(),
                    ListTag::Compound(x) => x.shrink_to_fit(),
                    ListTag::String(x) => x.shrink_to_fit(),
                    ListTag::Int(x) => x.shrink_to_fit(),
                    ListTag::Double(x) => x.shrink_to_fit(),
                    ListTag::Byte(x) => x.shrink_to_fit(),
                    ListTag::Short(x) => x.shrink_to_fit(),
                    ListTag::Long(x) => x.shrink_to_fit(),
                    ListTag::Float(x) => x.shrink_to_fit(),
                    ListTag::ByteArray(x) => x.shrink_to_fit(),
                    ListTag::IntArray(x) => x.shrink_to_fit(),
                    ListTag::LongArray(x) => x.shrink_to_fit(),
                    ListTag::None => (),
                },
            }
            match (blocks.last_mut(), bl) {
                (Some(Bl::C(c)), bl2) => match names[..] {
                    [ref rest @ .., l1, l2, l3, l4] => {
                        let len = u32::from_le_bytes([l1, l2, l3, l4]) as usize;
                        let new_len = rest.len() - len;
                        c.push(
                            unsafe {
                                Name::new(from_utf8_unchecked(
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
                    if let ListTag::None = l {
                        *l = ListTag::Compound(Vec::new())
                    }
                    if let ListTag::Compound(l) = l {
                        l.push(x);
                        continue;
                    } else {
                        return Err(Error);
                    }
                }
                (Some(Bl::L(l)), Bl::L(x)) => {
                    if let ListTag::None = l {
                        *l = ListTag::List(Vec::new())
                    }
                    if let ListTag::List(l) = l {
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
                let k = match n.peek_byte()? {
                    b'\"' => {
                        unsafe { n.advance(1) }
                        dec_quoted_str(n, &mut names, b'\"')?
                    }
                    b'\'' => {
                        unsafe { n.advance(1) }
                        dec_quoted_str(n, &mut names, b'\'')?
                    }
                    _ => unsafe {
                        let x = n.position(b": \n\t\r");
                        if n.end_from(x) {
                            return Err(Error);
                        }
                        let m = n.read_slice_from(x);
                        let a = names.len();
                        names.extend(m);
                        names.get_unchecked(a..)
                    },
                };
                let kl = (k.len() as u32).to_le_bytes();
                skip_ws(n);
                if b':' == n.peek_byte()? {
                    unsafe { n.advance(1) }
                } else {
                    return Err(Error);
                }
                skip_ws(n);
                let t = match n.peek_byte()? {
                    b'{' => {
                        unsafe { n.advance(1) }
                        names.extend(kl);
                        blocks.push(Bl::C(c));
                        blocks.push(Bl::C(Compound::new()));
                        on_start = true;
                        continue;
                    }
                    b'[' => {
                        unsafe { n.advance(1) }
                        match dec_arr_peek(n, &mut tmp) {
                            Ok(TagArray::Byte(x)) => Tag::ByteArray(x),
                            Ok(TagArray::Int(x)) => Tag::IntArray(x),
                            Ok(TagArray::Long(x)) => Tag::LongArray(x),
                            Err(_) => {
                                names.extend(kl);
                                blocks.push(Bl::C(c));
                                blocks.push(Bl::L(ListTag::None));
                                on_start = true;
                                continue;
                            }
                        }
                    }
                    b'"' => unsafe {
                        n.advance(1);
                        let s = dec_quoted_str(n, tmp.next(), b'"')?;
                        Tag::String(Box::from(from_utf8_unchecked(s)))
                    },
                    b'\'' => unsafe {
                        n.advance(1);
                        let s = dec_quoted_str(n, tmp.next(), b'\'')?;
                        Tag::String(Box::from(from_utf8_unchecked(s)))
                    },
                    _ => unsafe {
                        let value = find_next_value(n)?;
                        match dec_num(value, tmp.next()) {
                            Ok(x) => Tag::from(x),
                            Err(_) => Tag::String(Box::from(from_utf8_unchecked(value))),
                        }
                    },
                };
                unsafe {
                    c.push(Name::new(from_utf8_unchecked(k)), t);
                }
            }
            Bl::L(mut l) => match n.peek_byte()? {
                b'{' => unsafe {
                    n.advance(1);
                    if let ListTag::None = &l {
                        l = ListTag::Compound(Vec::new());
                    }
                    blocks.push(Bl::L(l));
                    blocks.push(Bl::C(Compound::new()));
                    on_start = true;
                },
                b'[' => unsafe {
                    n.advance(1);
                    match dec_arr_peek(n, &mut tmp) {
                        Ok(arr) => {
                            match arr {
                                TagArray::Byte(b) => {
                                    if let ListTag::None = &l {
                                        l = ListTag::ByteArray(Vec::new());
                                    }
                                    match &mut l {
                                        ListTag::ByteArray(x) => x.push(b),
                                        _ => return Err(Error),
                                    }
                                }
                                TagArray::Int(b) => {
                                    if let ListTag::None = &l {
                                        l = ListTag::IntArray(Vec::new());
                                    }
                                    match &mut l {
                                        ListTag::IntArray(x) => x.push(b),
                                        _ => return Err(Error),
                                    }
                                }
                                TagArray::Long(b) => {
                                    if let ListTag::None = &l {
                                        l = ListTag::LongArray(Vec::new());
                                    }
                                    match &mut l {
                                        ListTag::LongArray(x) => x.push(b),
                                        _ => return Err(Error),
                                    }
                                }
                            }
                            blocks.push(Bl::L(l));
                        }
                        Err(_) => {
                            match &l {
                                ListTag::None => {
                                    l = ListTag::List(Vec::new());
                                }
                                ListTag::List(_) => (),
                                _ => return Err(Error),
                            }
                            blocks.push(Bl::L(l));
                            blocks.push(Bl::L(ListTag::None));
                            on_start = true;
                        }
                    }
                },
                first => unsafe {
                    let tag = match first {
                        b'"' => {
                            n.advance(1);
                            let s = dec_quoted_str(n, tmp.next(), b'"')?;
                            Err(Box::from(from_utf8_unchecked(s)))
                        }
                        b'\'' => {
                            n.advance(1);
                            let s = dec_quoted_str(n, tmp.next(), b'\'')?;
                            Err(Box::from(from_utf8_unchecked(s)))
                        }
                        _ => {
                            let value = find_next_value(n)?;
                            match dec_num(value, tmp.next()) {
                                Ok(x) => Ok(x),
                                Err(_) => Err(Box::from(from_utf8_unchecked(value))),
                            }
                        }
                    };
                    let l = match tag {
                        Ok(x) => ListTag::from(dec_list_primitive(n, &mut tmp, x)?),
                        Err(e) => {
                            let mut list = vec![e];
                            dec_list_string(n, &mut tmp, &mut list)?;
                            ListTag::String(list)
                        }
                    };
                    blocks.push(Bl::L(l));
                },
            },
        }
    }
}

unsafe fn dec_list_string(
    n: &mut Reader,
    tmp: &mut TBuf,
    list: &mut Vec<Box<str>>,
) -> Result<(), Error> {
    loop {
        skip_ws(n);
        match u8::read(n)? {
            b',' => {
                skip_ws(n);
                let x = match n.peek_byte()? {
                    b'\"' => unsafe {
                        n.advance(1);
                        Box::from(from_utf8_unchecked(dec_quoted_str(n, tmp.next(), b'\"')?))
                    },
                    b'\'' => unsafe {
                        n.advance(1);
                        Box::from(from_utf8_unchecked(dec_quoted_str(n, tmp.next(), b'\'')?))
                    },
                    _ => unsafe { Box::from(from_utf8_unchecked(find_next_value(n)?)) },
                };
                list.push(x);
            }
            b']' => return Ok(()),
            _ => return Err(Error),
        }
    }
}

unsafe fn dec_list_primitive(
    n: &mut Reader,
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
                let value = find_next_value(n)?;
                let num = match unsafe { dec_num(value, tmp.next())? } {
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

const ESCAPE: u8 = b'\\';

fn dec_quoted_str<'a>(n: &mut Reader, buf: &'a mut Vec<u8>, quote: u8) -> Result<&'a [u8], Error> {
    let begin = buf.len();
    loop {
        let cur = n.memchr2(ESCAPE, quote);
        if n.end_from(cur) {
            return Err(Error);
        }
        unsafe { buf.extend(n.read_slice_from(cur)) }
        if n.read_byte()? != ESCAPE {
            break;
        }
        let ch = escape_quoted(n)?;
        buf.reserve(ch.len_utf8());
        let ch =
            ch.encode_utf8(unsafe { core::slice::from_raw_parts_mut(buf.as_mut_ptr(), buf.len()) });
        unsafe { buf.set_len(buf.len() + ch.len()) }
    }
    unsafe { Ok(buf.get_unchecked(begin..)) }
}

fn escape_quoted(n: &mut Reader) -> Result<char, Error> {
    match n.read_byte()? {
        ESCAPE => Ok('\\'),
        b'\'' => Ok('\''),
        b'"' => Ok('"'),
        b'b' => Ok('\x08'),
        b't' => Ok('\t'),
        b'r' => Ok('\r'),
        b'f' => Ok('\x0c'),
        b'n' => Ok('\n'),
        b'x' => match dec_char2(*n.read_array()?) {
            Some(x) => Ok(x),
            None => Err(Error),
        },
        b'u' => match dec_char4(*n.read_array()?) {
            Some(x) => Ok(x),
            None => Err(Error),
        },
        b'U' => match dec_char8(*n.read_array()?) {
            Some(x) => Ok(x),
            None => Err(Error),
        },
        b'N' => unsafe {
            if b'{' == n.peek_byte()? {
                n.advance(1);
                let ptr = n.memchr(b'}');
                if !n.end_from(ptr) {
                    let name = from_utf8_unchecked(n.read_slice_from(ptr)).trim_ascii();
                    n.advance(1);
                    if let Some(x) = unicode_names2::character(name) {
                        Ok(x)
                    } else {
                        Err(Error)
                    }
                } else {
                    Err(Error)
                }
            } else {
                Err(Error)
            }
        },
        _ => Err(Error),
    }
}

fn dec_char2([a, b]: [u8; 2]) -> Option<char> {
    let ch = (hex_to_u8(a)? as u32) << 4 | (hex_to_u8(b)? as u32);
    char::from_u32(ch)
}

fn dec_char4([a, b, c, d]: [u8; 4]) -> Option<char> {
    let ch = (hex_to_u8(a)? as u32) << 12
        | (hex_to_u8(b)? as u32) << 8
        | (hex_to_u8(c)? as u32) << 4
        | (hex_to_u8(d)? as u32);
    char::from_u32(ch)
}

fn dec_char8([a, b, c, d, e, f, g, h]: [u8; 8]) -> Option<char> {
    let ch = (hex_to_u8(a)? as u32) << 28
        | (hex_to_u8(b)? as u32) << 24
        | (hex_to_u8(c)? as u32) << 20
        | (hex_to_u8(d)? as u32) << 16
        | (hex_to_u8(e)? as u32) << 12
        | (hex_to_u8(f)? as u32) << 8
        | (hex_to_u8(g)? as u32) << 4
        | (hex_to_u8(h)? as u32);
    char::from_u32(ch)
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
fn skip_ws(n: &mut Reader) {
    while let Ok(b' ' | b'\n' | b'\t' | b'\r') = n.peek_byte() {
        unsafe { n.advance(1) }
    }
}

const SPACE: &[u8] = b"    ";
const DELIMITER: &[u8] = b", ";

fn encode(buf: &mut Vec<u8>, n: &Compound) {
    #[derive(Clone, Copy)]
    enum Bl<'a> {
        C(&'a [(Name, Tag)]),
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
        List(&'a [ListTag]),
        Compound(&'a [Compound]),
    }
    impl<'a> From<&'a ListTag> for Bl<'a> {
        fn from(value: &'a ListTag) -> Self {
            match value {
                ListTag::None => Self::None,
                ListTag::Byte(items) => Self::Byte(items),
                ListTag::Short(items) => Self::Short(items),
                ListTag::Int(items) => Self::Int(items),
                ListTag::Long(items) => Self::Long(items),
                ListTag::Float(items) => Self::Float(items),
                ListTag::Double(items) => Self::Double(items),
                ListTag::String(box_strs) => Self::String(box_strs),
                ListTag::ByteArray(items) => Self::ByteArray(items),
                ListTag::IntArray(items) => Self::IntArray(items),
                ListTag::LongArray(items) => Self::LongArray(items),
                ListTag::List(lists) => Self::List(lists),
                ListTag::Compound(compounds) => Self::Compound(compounds),
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
