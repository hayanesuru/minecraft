mod list;
mod mutf8;
mod string;
mod stringify;

pub use self::list::List;
pub use self::string::{MUTF8Tag, UTF8Tag};
pub use self::stringify::StringifyCompound;
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
            unsafe {
                UTF8Tag::new_unchecked(name.as_bytes()).write(w);
            }
            match tag {
                Tag::Byte(x) => x.write(w),
                Tag::Short(x) => x.write(w),
                Tag::Int(x) => x.write(w),
                Tag::Long(x) => x.write(w),
                Tag::Float(x) => x.write(w),
                Tag::Double(x) => x.write(w),
                Tag::String(x) => unsafe { UTF8Tag::new_unchecked(x.as_bytes()).write(w) },
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

    fn sz(&self) -> usize {
        let mut w = 1 + self.0.len();
        for (name, tag) in &self.0 {
            w += unsafe { UTF8Tag::new_unchecked(name.as_bytes()).sz() };
            w += match tag {
                Tag::Byte(_) => 1,
                Tag::Short(_) => 2,
                Tag::Int(_) => 4,
                Tag::Long(_) => 8,
                Tag::Float(_) => 4,
                Tag::Double(_) => 8,
                Tag::String(x) => unsafe { UTF8Tag::new_unchecked(x.as_bytes()).sz() },
                Tag::ByteArray(x) => 4 + x.len(),
                Tag::IntArray(x) => 4 + x.len() * 4,
                Tag::LongArray(x) => 4 + x.len() * 8,
                Tag::List(x) => x.sz(),
                Tag::Compound(x) => Write::sz(x),
            };
        }
        w
    }
}

#[derive(Clone)]
pub struct NamedCompound(pub String, pub Compound);

impl Read for NamedCompound {
    #[inline]
    fn read(n: &mut &[u8]) -> Option<Self> {
        if n.u8()? != COMPOUND {
            return None;
        }
        Some(Self(decode_string(n)?, decode1(n)?))
    }
}

impl Write for NamedCompound {
    #[inline]
    fn write(&self, w: &mut UnsafeWriter) {
        w.write_byte(COMPOUND);
        unsafe {
            UTF8Tag::new_unchecked(self.0.as_bytes()).write(w);
        }
        self.1.write(w);
    }

    #[inline]
    fn sz(&self) -> usize {
        1 + unsafe { Write::sz(&UTF8Tag::new_unchecked(self.0.as_bytes())) } + Write::sz(&self.1)
    }
}

#[derive(Clone)]
pub struct UnamedCompound(pub Compound);

impl Read for UnamedCompound {
    #[inline]
    fn read(n: &mut &[u8]) -> Option<Self> {
        if n.u8()? != COMPOUND {
            return None;
        }
        Some(Self(decode1(n)?))
    }
}

impl Write for UnamedCompound {
    #[inline]
    fn write(&self, w: &mut UnsafeWriter) {
        w.write_byte(COMPOUND);
        self.0.write(w);
    }

    #[inline]
    fn sz(&self) -> usize {
        1 + Write::sz(&self.0)
    }
}

impl From<Compound> for UnamedCompound {
    #[inline]
    fn from(value: Compound) -> Self {
        Self(value)
    }
}

impl From<Compound> for NamedCompound {
    #[inline]
    fn from(value: Compound) -> Self {
        Self(String::new(), value)
    }
}

impl<K: ToString, V> FromIterator<(K, V)> for Compound
where
    Tag: From<V>,
{
    #[inline]
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
    pub fn iter(&self) -> core::slice::Iter<'_, (Box<str>, Tag)> {
        self.0.iter()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> core::slice::IterMut<'_, (Box<str>, Tag)> {
        self.0.iter_mut()
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
    ///
    /// `index` < [`len`]
    ///
    /// [`len`]: Self::len
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

    #[deprecated]
    #[inline]
    pub fn decode(buf: &mut &[u8]) -> Option<Self> {
        match UnamedCompound::read(buf) {
            Some(x) => Some(x.0),
            None => None,
        }
    }

    #[deprecated]
    #[inline]
    pub fn decode_named(buf: &mut &[u8]) -> Option<(String, Self)> {
        match NamedCompound::read(buf) {
            Some(x) => Some((x.0, x.1)),
            None => None,
        }
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
        decode1(buf)
    }
}

impl From<Vec<(Box<str>, Tag)>> for Compound {
    #[inline]
    fn from(value: Vec<(Box<str>, Tag)>) -> Self {
        Self(value)
    }
}

fn decode1(n: &mut &[u8]) -> Option<Compound> {
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
                compound.push(k, list::decode2(n, id, len)?);
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

#[inline]
pub fn decode_string(b: &mut &[u8]) -> Option<String> {
    let len = b.u16()? as usize;
    let a = b.slice(len)?;
    match simdutf8::basic::from_utf8(a) {
        Ok(n) => Some(String::from(n)),
        Err(_) => mutf8::decode(a),
    }
}
