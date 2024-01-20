use crate::{Bytes, Read, UnsafeWriter, Write};
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

pub const END: u8 = 0;
pub const BYTE: u8 = 1;
pub const SHORT: u8 = 2;
pub const INT: u8 = 3;
pub const LONG: u8 = 4;
pub const FLOAT: u8 = 5;
pub const DOUBLE: u8 = 6;
pub const BYTE_ARRAY: u8 = 7;
pub const STRING: u8 = 8;
pub const LIST: u8 = 9;
pub const COMPOUND: u8 = 10;
pub const INT_ARRAY: u8 = 11;
pub const LONG_ARRAY: u8 = 12;

#[derive(Clone)]
pub enum Tag {
    Byte(u8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    String(Box<str>),
    ByteArray(Vec<u8>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
    List(List),
    Compound(Compound),
}

impl From<bool> for Tag {
    #[inline]
    fn from(value: bool) -> Self {
        Self::Byte(value as u8)
    }
}

impl From<u8> for Tag {
    #[inline]
    fn from(value: u8) -> Self {
        Self::Byte(value)
    }
}

impl From<i16> for Tag {
    #[inline]
    fn from(value: i16) -> Self {
        Self::Short(value)
    }
}

impl From<i32> for Tag {
    #[inline]
    fn from(value: i32) -> Self {
        Self::Int(value)
    }
}

impl From<i64> for Tag {
    #[inline]
    fn from(value: i64) -> Self {
        Self::Long(value)
    }
}

impl From<f32> for Tag {
    #[inline]
    fn from(value: f32) -> Self {
        Self::Float(value)
    }
}

impl From<f64> for Tag {
    #[inline]
    fn from(value: f64) -> Self {
        Self::Double(value)
    }
}

impl From<Vec<u8>> for Tag {
    #[inline]
    fn from(value: Vec<u8>) -> Self {
        Self::ByteArray(value)
    }
}

impl From<String> for Tag {
    #[inline]
    fn from(value: String) -> Self {
        Self::String(value.into_boxed_str())
    }
}

impl From<Box<str>> for Tag {
    #[inline]
    fn from(value: Box<str>) -> Self {
        Self::String(value)
    }
}

impl From<List> for Tag {
    #[inline]
    fn from(value: List) -> Self {
        Self::List(value)
    }
}

impl From<Compound> for Tag {
    #[inline]
    fn from(value: Compound) -> Self {
        Self::Compound(value)
    }
}

impl From<Vec<i32>> for Tag {
    #[inline]
    fn from(value: Vec<i32>) -> Self {
        Self::IntArray(value)
    }
}

impl From<Vec<i64>> for Tag {
    #[inline]
    fn from(value: Vec<i64>) -> Self {
        Self::LongArray(value)
    }
}

#[inline]
pub fn decode_string(b: &mut &[u8]) -> Option<String> {
    let len = b.u16()? as usize;
    let a = b.slice(len)?;
    match core::str::from_utf8(a) {
        Ok(n) => Some(String::from(n)),
        Err(_) => super::mutf8::decode(a),
    }
}

pub struct MUTF8Tag<'a>(pub &'a [u8]);

impl<'a> Write for MUTF8Tag<'a> {
    #[inline]
    fn write(&self, w: &mut UnsafeWriter) {
        (self.0.len() as u16).write(w);
        w.write(self.0);
    }

    #[inline]
    fn len(&self) -> usize {
        2 + self.0.len()
    }
}

pub struct UTF8Tag<'a>(pub &'a [u8]);

impl<'a> Write for UTF8Tag<'a> {
    #[inline]
    fn write(&self, w: &mut UnsafeWriter) {
        if super::mutf8::is_valid(self.0) {
            (self.0.len() as u16).write(w);
            w.write(self.0);
        } else {
            unsafe {
                (super::mutf8::len(self.0) as u16).write(w);
                super::mutf8::encode(self.0, w);
            }
        }
    }

    #[inline]
    fn len(&self) -> usize {
        if super::mutf8::is_valid(self.0) {
            2 + self.0.len()
        } else {
            unsafe { 2 + super::mutf8::len(self.0) }
        }
    }
}

#[derive(Clone, Default)]
#[repr(transparent)]
pub struct Compound(Vec<(Box<str>, Tag)>);

impl AsRef<[(Box<str>, Tag)]> for Compound {
    #[inline]
    fn as_ref(&self) -> &[(Box<str>, Tag)] {
        self.0.as_slice()
    }
}

impl AsMut<[(Box<str>, Tag)]> for Compound {
    #[inline]
    fn as_mut(&mut self) -> &mut [(Box<str>, Tag)] {
        self.0.as_mut_slice()
    }
}

impl Write for Compound {
    fn write(&self, w: &mut UnsafeWriter) {
        for (name, tag) in &self.0 {
            w.write_byte(match tag {
                Tag::Byte(_) => BYTE,
                Tag::Short(_) => SHORT,
                Tag::Int(_) => INT,
                Tag::Long(_) => LONG,
                Tag::Float(_) => FLOAT,
                Tag::Double(_) => DOUBLE,
                Tag::String(_) => STRING,
                Tag::ByteArray(_) => BYTE_ARRAY,
                Tag::IntArray(_) => INT_ARRAY,
                Tag::LongArray(_) => LONG_ARRAY,
                Tag::List(_) => LIST,
                Tag::Compound(_) => COMPOUND,
            });
            MUTF8Tag(name.as_bytes()).write(w);
            match tag {
                Tag::Byte(x) => x.write(w),
                Tag::Short(x) => x.write(w),
                Tag::Int(x) => x.write(w),
                Tag::Long(x) => x.write(w),
                Tag::Float(x) => x.write(w),
                Tag::Double(x) => x.write(w),
                Tag::String(x) => MUTF8Tag(x.as_bytes()).write(w),
                Tag::ByteArray(x) => {
                    (x.len() as u32).write(w);
                    w.write(x)
                }
                Tag::IntArray(x) => {
                    (x.len() as u32).write(w);
                    x.iter().for_each(|&x| x.write(w));
                }
                Tag::LongArray(x) => {
                    (x.len() as u32).write(w);
                    x.iter().for_each(|&x| x.write(w));
                }
                Tag::List(x) => x.write(w),
                Tag::Compound(x) => x.write(w),
            }
        }
        w.write_byte(END);
    }

    fn len(&self) -> usize {
        let mut w = 1 + self.0.len();
        for (name, tag) in &self.0 {
            w += UTF8Tag(name.as_bytes()).len();
            w += match tag {
                Tag::Byte(_) => 1,
                Tag::Short(_) => 2,
                Tag::Int(_) => 4,
                Tag::Long(_) => 8,
                Tag::Float(_) => 4,
                Tag::Double(_) => 8,
                Tag::String(x) => MUTF8Tag(x.as_bytes()).len(),
                Tag::ByteArray(x) => 4 + x.len(),
                Tag::IntArray(x) => 4 + x.len() * 4,
                Tag::LongArray(x) => 4 + x.len() * 8,
                Tag::List(x) => x.len(),
                Tag::Compound(x) => Write::len(x),
            };
        }
        w
    }
}

impl<K: ToString, V> FromIterator<(K, V)> for Compound
where
    Tag: From<V>,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        Self(
            iter.into_iter()
                .map(|(x, y)| (x.to_string().into_boxed_str(), Tag::from(y)))
                .collect(),
        )
    }
}

impl Compound {
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.0.shrink_to_fit()
    }

    #[inline]
    pub fn iter(&self) -> core::slice::Iter<(Box<str>, Tag)> {
        self.0.iter()
    }

    #[inline]
    pub fn sort(&mut self) {
        self.0.sort_unstable_by(|x, y| (*x.0).cmp(&*y.0));
    }

    #[inline]
    #[must_use]
    pub fn get(&self, index: usize) -> Option<(&str, &Tag)> {
        #[allow(clippy::manual_map)]
        match self.0.get(index) {
            Some((x, y)) => Some((x, y)),
            None => None,
        }
    }

    #[inline]
    #[must_use]
    /// # Safety
    pub unsafe fn get_unchecked_mut(&mut self, index: usize) -> (&mut str, &mut Tag) {
        let (x, y) = self.0.get_unchecked_mut(index);
        (&mut *x, y)
    }

    #[inline]
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    #[inline]
    pub fn push(&mut self, k: impl Into<Box<str>>, v: impl Into<Tag>) {
        self.0.push((k.into(), v.into()));
    }

    #[inline]
    pub fn decode(n: &mut &[u8]) -> Option<Self> {
        if n.u8()? != COMPOUND {
            return None;
        }

        decode1(n)
    }

    #[inline]
    pub fn decode_named(n: &mut &[u8]) -> Option<(String, Self)> {
        if n.u8()? != COMPOUND {
            return None;
        }
        Some((decode_string(n)?, decode1(n)?))
    }

    #[inline]
    pub fn find(&self, name: &str) -> Option<&Tag> {
        for (x, y) in &self.0 {
            let x = &**x;
            if x == name {
                return Some(y);
            }
        }
        None
    }

    #[inline]
    pub fn find_remove(&mut self, name: &str) -> Option<Tag> {
        for (i, (x, _)) in self.0.iter_mut().enumerate() {
            let x = &**x;
            if x == name {
                return Some(self.0.swap_remove(i).1);
            }
        }
        None
    }

    #[inline]
    pub fn find_mut(&mut self, name: &str) -> Option<&mut Tag> {
        for (x, y) in &mut self.0 {
            let x = &**x;
            if x == name {
                return Some(y);
            }
        }
        None
    }

    #[inline]
    pub fn into_inner(self) -> Vec<(Box<str>, Tag)> {
        self.0
    }
}

impl Read for Compound {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        Self::decode(buf)
    }
}

impl From<Vec<(Box<str>, Tag)>> for Compound {
    #[inline]
    fn from(value: Vec<(Box<str>, Tag)>) -> Self {
        Self(value)
    }
}

pub fn decode1(n: &mut &[u8]) -> Option<Compound> {
    let mut compound = Compound(Default::default());
    loop {
        match n.u8()? {
            END => {
                compound.0.shrink_to_fit();
                return Some(compound);
            }
            BYTE => {
                let k = decode_string(n)?;
                compound.push(k, n.u8()?);
            }
            SHORT => {
                let k = decode_string(n)?;
                compound.push(k, n.i16()?)
            }
            INT => {
                let k = decode_string(n)?;
                compound.push(k, n.i32()?)
            }
            LONG => {
                let k = decode_string(n)?;
                compound.push(k, n.i64()?)
            }
            FLOAT => {
                let k = decode_string(n)?;
                compound.push(k, n.f32()?)
            }
            DOUBLE => {
                let k = decode_string(n)?;
                compound.push(k, n.f64()?)
            }
            BYTE_ARRAY => {
                let k = decode_string(n)?;
                let len = n.i32()? as usize;
                let v = Vec::from(n.slice(len)?);
                compound.push(k, v);
            }
            STRING => {
                let k = decode_string(n)?;
                compound.push(k, decode_string(n)?)
            }
            LIST => {
                let k = decode_string(n)?;
                let id = n.u8()?;
                let len = n.i32()? as usize;
                compound.push(k, decode2(n, id, len)?);
            }
            COMPOUND => {
                let k = decode_string(n)?;
                let v = decode1(n)?;
                compound.push(k, v);
            }
            INT_ARRAY => {
                let k = decode_string(n)?;
                let len = n.i32()? as usize;
                let mut slice = n.slice(len * 4)?;
                let mut v = Vec::with_capacity(len);
                for _ in 0..len {
                    v.push(slice.i32()?);
                }
                compound.push(k, v);
            }
            LONG_ARRAY => {
                let k = decode_string(n)?;
                let len = n.i32()? as usize;
                let mut slice = n.slice(len * 8)?;
                let mut v = Vec::with_capacity(len);
                for _ in 0..len {
                    v.push(slice.i64()?);
                }
                compound.push(k, v);
            }
            _ => return None,
        }
    }
}

#[derive(Clone)]
pub enum List {
    None,
    Byte(Vec<u8>),
    Short(Vec<i16>),
    Int(Vec<i32>),
    Long(Vec<i64>),
    Float(Vec<f32>),
    Double(Vec<f64>),
    String(Vec<Box<str>>),
    ByteArray(Vec<Vec<u8>>),
    IntArray(Vec<Vec<i32>>),
    LongArray(Vec<Vec<i64>>),
    List(Vec<Self>),
    Compound(Vec<Compound>),
}

impl From<Vec<u8>> for List {
    #[inline]
    fn from(value: Vec<u8>) -> Self {
        Self::Byte(value)
    }
}

impl From<Vec<i16>> for List {
    #[inline]
    fn from(value: Vec<i16>) -> Self {
        Self::Short(value)
    }
}

impl From<Vec<i32>> for List {
    #[inline]
    fn from(value: Vec<i32>) -> Self {
        Self::Int(value)
    }
}

impl From<Vec<i64>> for List {
    #[inline]
    fn from(value: Vec<i64>) -> Self {
        Self::Long(value)
    }
}

impl From<Vec<f32>> for List {
    #[inline]
    fn from(value: Vec<f32>) -> Self {
        Self::Float(value)
    }
}

impl From<Vec<f64>> for List {
    #[inline]
    fn from(value: Vec<f64>) -> Self {
        Self::Double(value)
    }
}

impl From<Vec<Box<str>>> for List {
    #[inline]
    fn from(value: Vec<Box<str>>) -> Self {
        Self::String(value)
    }
}

impl From<Vec<Vec<u8>>> for List {
    #[inline]
    fn from(value: Vec<Vec<u8>>) -> Self {
        Self::ByteArray(value)
    }
}

impl From<Vec<Vec<i32>>> for List {
    #[inline]
    fn from(value: Vec<Vec<i32>>) -> Self {
        Self::IntArray(value)
    }
}

impl From<Vec<Vec<i64>>> for List {
    #[inline]
    fn from(value: Vec<Vec<i64>>) -> Self {
        Self::LongArray(value)
    }
}

impl From<Vec<List>> for List {
    #[inline]
    fn from(value: Vec<List>) -> Self {
        Self::List(value)
    }
}

impl From<Vec<Compound>> for List {
    #[inline]
    fn from(value: Vec<Compound>) -> Self {
        Self::Compound(value)
    }
}

impl Write for List {
    fn write(&self, w: &mut UnsafeWriter) {
        match self {
            Self::None => w.write(&[END, 0, 0, 0, 0]),
            Self::Byte(x) => {
                w.write_byte(BYTE);
                (x.len() as u32).write(w);
                w.write(x);
            }
            Self::Short(x) => {
                w.write_byte(SHORT);
                (x.len() as u32).write(w);
                x.iter().for_each(|&x| x.write(w));
            }
            Self::Int(x) => {
                w.write_byte(INT);
                (x.len() as u32).write(w);
                x.iter().for_each(|&x| x.write(w));
            }
            Self::Long(x) => {
                w.write_byte(LONG);
                (x.len() as u32).write(w);
                x.iter().for_each(|&x| x.write(w));
            }
            Self::Float(x) => {
                w.write_byte(FLOAT);
                (x.len() as u32).write(w);
                x.iter().for_each(|&x| x.write(w));
            }
            Self::Double(x) => {
                w.write_byte(DOUBLE);
                (x.len() as u32).write(w);
                x.iter().for_each(|&x| x.write(w));
            }
            Self::String(x) => {
                (STRING).write(w);
                (x.len() as u32).write(w);
                x.iter().for_each(|x| MUTF8Tag(x.as_bytes()).write(w));
            }
            Self::ByteArray(x) => {
                w.write_byte(BYTE_ARRAY);
                (x.len() as u32).write(w);
                x.iter().for_each(|y| {
                    (y.len() as u32).write(w);
                    y.iter().for_each(|&z| z.write(w))
                });
            }
            Self::IntArray(x) => {
                w.write_byte(INT_ARRAY);
                (x.len() as u32).write(w);
                x.iter().for_each(|y| {
                    (y.len() as u32).write(w);
                    y.iter().for_each(|&z| z.write(w))
                });
            }
            Self::LongArray(x) => {
                w.write_byte(LONG_ARRAY);
                (x.len() as u32).write(w);
                x.iter().for_each(|y| {
                    (y.len() as u32).write(w);
                    y.iter().for_each(|&z| z.write(w))
                });
            }
            Self::List(x) => {
                w.write_byte(LIST);
                (x.len() as u32).write(w);
                x.iter().for_each(|x| x.write(w));
            }
            Self::Compound(x) => {
                (COMPOUND).write(w);
                (x.len() as u32).write(w);
                x.iter().for_each(|x| x.write(w));
            }
        }
    }

    fn len(&self) -> usize {
        5 + match self {
            Self::None => 0,
            Self::Byte(x) => x.len(),
            Self::Short(x) => x.len() * 2,
            Self::Int(x) => x.len() * 4,
            Self::Long(x) => x.len() * 8,
            Self::Float(x) => x.len() * 4,
            Self::Double(x) => x.len() * 8,
            Self::String(x) => x
                .iter()
                .map(|x| MUTF8Tag(x.as_bytes()).len())
                .sum::<usize>(),
            Self::ByteArray(x) => x.len() * 4 + x.iter().map(|x| x.len()).sum::<usize>(),
            Self::IntArray(x) => x.len() * 4 + x.iter().map(|x| x.len()).sum::<usize>() * 4,
            Self::LongArray(x) => x.len() * 4 + x.iter().map(|x| x.len()).sum::<usize>() * 8,
            Self::List(x) => x.iter().map(|x| x.len()).sum::<usize>(),
            Self::Compound(x) => x.iter().map(Write::len).sum::<usize>(),
        }
    }
}

pub fn decode2(n: &mut &[u8], id: u8, len: usize) -> Option<List> {
    match id {
        END => Some(List::None),
        BYTE => Some(List::Byte(n.slice(len)?.to_vec())),
        SHORT => {
            let mut slice = n.slice(len << 1)?;
            let mut v = Vec::with_capacity(len);
            for _ in 0..len {
                v.push(slice.i16()?);
            }
            Some(List::Short(v))
        }
        INT => {
            let mut slice = n.slice(len << 2)?;
            let mut v = Vec::with_capacity(len);
            for _ in 0..len {
                v.push(slice.i32()?);
            }
            Some(List::Int(v))
        }
        LONG => {
            let mut slice = n.slice(len << 3)?;
            let mut v = Vec::with_capacity(len);
            for _ in 0..len {
                v.push(slice.i64()?);
            }
            Some(List::Long(v))
        }
        FLOAT => {
            let mut slice = n.slice(len << 2)?;
            let mut v = Vec::with_capacity(len);
            for _ in 0..len {
                v.push(slice.f32()?);
            }
            Some(List::Float(v))
        }
        DOUBLE => {
            let mut slice = n.slice(len << 3)?;
            let mut v = Vec::with_capacity(len);
            for _ in 0..len {
                v.push(slice.f64()?);
            }
            Some(List::Double(v))
        }
        BYTE_ARRAY => {
            if len * 4 > n.len() {
                return None;
            }
            let mut list = Vec::with_capacity(len);
            for _ in 0..len {
                let len = n.i32()? as usize;
                let slice = n.slice(len)?;
                list.push(slice.to_vec());
            }
            Some(List::ByteArray(list))
        }
        STRING => {
            if len * 2 > n.len() {
                return None;
            }
            let mut list = Vec::with_capacity(len);
            for _ in 0..len {
                let v = decode_string(n)?.into_boxed_str();
                list.push(v);
            }
            Some(List::String(list))
        }
        LIST => {
            if len << 2 > n.len() {
                return None;
            }
            let mut list = Vec::with_capacity(len);
            for _ in 0..len {
                let id = n.u8()?;
                let len = n.i32()? as usize;
                list.push(decode2(n, id, len)?);
            }
            Some(List::List(list))
        }
        COMPOUND => {
            if len > n.len() {
                return None;
            }
            let mut list = Vec::with_capacity(len);
            for _ in 0..len {
                list.push(decode1(n)?);
            }
            Some(List::Compound(list))
        }
        INT_ARRAY => {
            if len * 4 > n.len() {
                return None;
            }
            let mut list = Vec::with_capacity(len);
            for _ in 0..len {
                let len = n.i32()? as usize;
                let mut slice = n.slice(len << 2)?;
                let mut v = Vec::with_capacity(len);
                for _ in 0..len {
                    v.push(slice.i32()?);
                }
                list.push(v);
            }
            Some(List::IntArray(list))
        }
        LONG_ARRAY => {
            if len * 4 > n.len() {
                return None;
            }
            let mut list = Vec::with_capacity(len);
            for _ in 0..len {
                let len = n.i32()? as usize;
                let mut slice = n.slice(len * 8)?;
                let mut v = Vec::with_capacity(len);
                for _ in 0..len {
                    v.push(slice.i64()?);
                }
                list.push(v);
            }
            Some(List::LongArray(list))
        }
        _ => None,
    }
}
