mod list;
mod string;
mod stringify;

pub use self::list::{List, ListInfo};
pub use self::string::{IdentifierTag, RefStringTag, StringTagRaw};
pub use self::stringify::StringifyCompound;
use crate::profile::Property;
use crate::str::BoxStr;
use crate::{Bytes, Error, Ident, Identifier, Read, UnsafeWriter, Write};
use alloc::boxed::Box;
use alloc::vec::Vec;
use mser::{decode_mutf8, decode_mutf8_len, is_ascii_mutf8, unlikely};
use uuid::Uuid;

pub struct Map<T>(pub T);

impl<'a, T: MapCodec> Read<'a> for Map<T> {
    fn read(buf: &mut &'a [u8]) -> Result<Self, Error> {
        match TagType::read(buf) {
            Ok(TagType::Compound) => match T::read_kv(buf) {
                Ok(x) => Ok(Map(x)),
                Err(e) => Err(e),
            },
            _ => Err(Error),
        }
    }
}

impl<T: MapCodec> Write for Map<T> {
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            TagType::Compound.write(w);
            self.0.write_kv(w);
        }
    }

    fn len_s(&self) -> usize {
        TagType::Compound.len_s() + self.0.len_kv()
    }
}

pub trait MapCodec: Sized {
    fn read_kv(buf: &mut &[u8]) -> Result<Self, Error>;
    /// # Safety
    ///
    /// Must write [`len_kv`] bytes exactly.
    ///
    /// [`len_kv`]: MapCodec::len_kv
    unsafe fn write_kv(&self, w: &mut UnsafeWriter);
    fn len_kv(&self) -> usize;
}

pub trait MapReader<T: Sized = Self> {
    fn visit(&mut self, ty: TagType, k: &str, buf: &mut &[u8]) -> Result<(), Error>;
    fn end(self) -> Result<T, Error>;
}

pub fn read_map<T, R: MapReader<T>>(mut r: R, buf: &mut &[u8]) -> Result<T, Error> {
    let mut temp = Vec::new();
    loop {
        let ty = TagType::read(buf)?;
        if unlikely(matches!(ty, TagType::End)) {
            return r.end();
        }
        let len = buf.u16()? as usize;
        let a = buf.slice(len)?;
        let k = if is_ascii_mutf8(a) {
            unsafe { core::str::from_utf8_unchecked(a) }
        } else {
            let len = decode_mutf8_len(a)?;
            temp.clear();
            temp.reserve_exact(len);
            unsafe {
                mser::write_unchecked(temp.as_mut_ptr(), &(DecodeMutf8(a, len)));
                temp.set_len(len);
                core::str::from_utf8_unchecked(&temp)
            }
        };
        r.visit(ty, k, buf)?;
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TagType {
    End,
    Byte,
    Short,
    Int,
    Long,
    Float,
    Double,
    ByteArray,
    String,
    List,
    Compound,
    IntArray,
    LongArray,
}

#[must_use]
pub struct Kv<'a, T>(pub &'a [u8], pub T);

pub struct End;

impl Write for End {
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe { TagType::End.write(w) }
    }

    fn len_s(&self) -> usize {
        TagType::End.len_s()
    }
}

impl<'a> Read<'a> for End {
    fn read(buf: &mut &'a [u8]) -> Result<Self, Error> {
        match TagType::read(buf) {
            Ok(TagType::End) => Ok(Self),
            _ => Err(Error),
        }
    }
}

impl<'a> Write for Kv<'a, &'a BoxStr> {
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            Kv(self.0, self.1.as_str()).write(w);
        }
    }

    fn len_s(&self) -> usize {
        Kv(self.0, self.1.as_str()).len_s()
    }
}

impl<'a> Write for Kv<'a, &'a str> {
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            TagType::String.write(w);
            StringTagRaw::new_unchecked(self.0).write(w);
            RefStringTag(self.1).write(w);
        }
    }

    fn len_s(&self) -> usize {
        TagType::String.len_s()
            + StringTagRaw::new_unchecked(self.0).len_s()
            + RefStringTag(self.1).len_s()
    }
}

impl<'a> Write for Kv<'a, ListInfo> {
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            TagType::List.write(w);
            StringTagRaw::new_unchecked(self.0).write(w);
            self.1.write(w);
        }
    }

    fn len_s(&self) -> usize {
        TagType::List.len_s() + StringTagRaw::new_unchecked(self.0).len_s() + self.1.len_s()
    }
}

impl<'a> Write for Kv<'a, StringTagRaw<'a>> {
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            TagType::String.write(w);
            StringTagRaw::new_unchecked(self.0).write(w);
            self.1.write(w);
        }
    }

    fn len_s(&self) -> usize {
        TagType::String.len_s() + StringTagRaw::new_unchecked(self.0).len_s() + self.1.len_s()
    }
}

impl<'a> Write for Kv<'a, bool> {
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            TagType::Byte.write(w);
            StringTagRaw::new_unchecked(self.0).write(w);
            self.1.write(w);
        }
    }

    fn len_s(&self) -> usize {
        TagType::Byte.len_s() + StringTagRaw::new_unchecked(self.0).len_s() + self.1.len_s()
    }
}

impl<'a> Write for Kv<'a, Ident<'a>> {
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            TagType::String.write(w);
            StringTagRaw::new_unchecked(self.0).write(w);
            IdentifierTag(self.1).write(w);
        }
    }

    fn len_s(&self) -> usize {
        TagType::String.len_s()
            + StringTagRaw::new_unchecked(self.0).len_s()
            + IdentifierTag(self.1).len_s()
    }
}

impl<'a> Write for Kv<'a, &'a Identifier> {
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            TagType::String.write(w);
            StringTagRaw::new_unchecked(self.0).write(w);
            IdentifierTag(self.1.as_ident()).write(w);
        }
    }

    fn len_s(&self) -> usize {
        TagType::String.len_s()
            + StringTagRaw::new_unchecked(self.0).len_s()
            + IdentifierTag(self.1.as_ident()).len_s()
    }
}

impl<'a, T: MapCodec> Write for Kv<'a, &'a T> {
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            TagType::Compound.write(w);
            StringTagRaw::new_unchecked(self.0).write(w);
            self.1.write_kv(w);
        }
    }

    fn len_s(&self) -> usize {
        TagType::Compound.len_s() + StringTagRaw::new_unchecked(self.0).len_s() + self.1.len_kv()
    }
}

impl<'a> Write for Kv<'a, Uuid> {
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            TagType::IntArray.write(w);
            StringTagRaw::new_unchecked(self.0).write(w);
            4u32.write(w);
            self.1.as_bytes().write(w);
        }
    }

    fn len_s(&self) -> usize {
        TagType::Compound.len_s() + StringTagRaw::new_unchecked(self.0).len_s() + 4u32.len_s() + 16
    }
}

impl<'a> Write for Kv<'a, &'a [Property]> {
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            TagType::Compound.write(w);
            StringTagRaw::new_unchecked(self.0).write(w);
            for p in self.1 {
                Kv(b"name", &p.name).write(w);
                Kv(b"value", &p.value).write(w);
                if let Some(ref signature) = p.signature {
                    Kv(b"signature", signature).write(w);
                }
            }
            TagType::End.write(w);
        }
    }

    fn len_s(&self) -> usize {
        let mut w = 0;
        for p in self.1 {
            w += Kv(b"name", &p.name).len_s();
            w += Kv(b"value", &p.value).len_s();
            if let Some(ref signature) = p.signature {
                w += Kv(b"signature", signature).len_s();
            }
        }
        w + TagType::Compound.len_s()
            + StringTagRaw::new_unchecked(self.0).len_s()
            + TagType::End.len_s()
    }
}

impl TagType {
    pub fn bool(self, buf: &mut &[u8]) -> Result<bool, Error> {
        match self {
            Self::Byte => Ok(buf.i8()? != 0),
            Self::Short => Ok(buf.i16()? != 0),
            Self::Int => Ok(buf.i32()? != 0),
            Self::Long => Ok(buf.i64()? != 0),
            Self::Float => Ok(buf.f32()? != 0.0),
            Self::Double => Ok(buf.f64()? != 0.0),
            _ => Err(Error),
        }
    }

    pub fn string(self, buf: &mut &[u8]) -> Result<BoxStr, Error> {
        match self {
            Self::String => match StringTag::read(buf) {
                Ok(x) => Ok(x.0),
                Err(e) => Err(e),
            },
            _ => Err(Error),
        }
    }

    pub fn ident(self, buf: &mut &[u8]) -> Result<Identifier, Error> {
        match self {
            Self::String => match IdentifierTag::read(buf) {
                Ok(x) => unsafe {
                    Ok(Identifier {
                        namespace: if x.0.namespace == Ident::MINECRAFT {
                            None
                        } else {
                            Some(BoxStr::new_unchecked(Box::from(x.0.namespace.as_bytes())))
                        },
                        path: BoxStr::new_unchecked(Box::from(x.0.path.as_bytes())),
                    })
                },
                Err(e) => Err(e),
            },
            _ => Err(Error),
        }
    }

    pub fn int_list(self, buf: &mut &[u8]) -> Result<Vec<i32>, Error> {
        match self {
            Self::IntArray => {
                let len = u32::read(buf)? as usize;
                let mut data = buf.slice(len * 4)?;
                let mut vec = Vec::with_capacity(len);
                let mut ptr = vec.as_mut_ptr();
                for _ in 0..len {
                    unsafe {
                        *ptr = i32::from_be_bytes(*data.array::<4>().unwrap_unchecked());
                        ptr = ptr.add(1);
                    }
                }
                unsafe { vec.set_len(len) }
                Ok(vec)
            }
            Self::List => {
                let ListInfo(tag, len) = ListInfo::read(buf)?;
                let len = len as usize;
                match tag {
                    TagType::Int => {
                        let mut data = buf.slice(len * 4)?;
                        let mut vec = Vec::with_capacity(len);
                        let mut ptr = vec.as_mut_ptr();
                        for _ in 0..len {
                            unsafe {
                                *ptr = i32::from_be_bytes(*data.array::<4>().unwrap_unchecked());
                                ptr = ptr.add(1);
                            }
                        }
                        unsafe { vec.set_len(len) }
                        Ok(vec)
                    }
                    _ => Err(Error),
                }
            }
            _ => Err(Error),
        }
    }
}

impl Read<'_> for TagType {
    #[inline]
    fn read(buf: &mut &'_ [u8]) -> Result<Self, Error> {
        let t = buf.u8()?;
        if t <= 12 {
            unsafe { Ok(core::mem::transmute::<u8, Self>(t)) }
        } else {
            Err(Error)
        }
    }
}

impl Write for TagType {
    #[inline]
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe { w.write_byte(*self as u8) }
    }

    #[inline]
    fn len_s(&self) -> usize {
        1
    }
}

#[derive(Clone)]
pub enum Tag {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    String(BoxStr),
    ByteArray(Vec<i8>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
    List(List),
    Compound(Compound),
}

impl Write for Tag {
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            self.id().write(w);
            match self {
                Tag::Byte(x) => x.write(w),
                Tag::Short(x) => x.write(w),
                Tag::Int(x) => x.write(w),
                Tag::Long(x) => x.write(w),
                Tag::Float(x) => x.write(w),
                Tag::Double(x) => x.write(w),
                Tag::String(x) => RefStringTag(x).write(w),
                Tag::ByteArray(x) => {
                    (x.len() as u32).write(w);
                    w.write(&*(x.as_slice() as *const [i8] as *const [u8]));
                }
                Tag::IntArray(x) => {
                    (x.len() as u32).write(w);
                    x.iter().write(w);
                }
                Tag::LongArray(x) => {
                    (x.len() as u32).write(w);
                    x.iter().write(w);
                }
                Tag::List(x) => x.write(w),
                Tag::Compound(x) => x.write(w),
            }
        }
    }

    fn len_s(&self) -> usize {
        1 + match self {
            Tag::Byte(_) => 1,
            Tag::Short(_) => 2,
            Tag::Int(_) => 4,
            Tag::Long(_) => 8,
            Tag::Float(_) => 4,
            Tag::Double(_) => 8,
            Tag::String(x) => RefStringTag(x).len_s(),
            Tag::ByteArray(x) => 4 + x.len(),
            Tag::IntArray(x) => 4 + x.len() * 4,
            Tag::LongArray(x) => 4 + x.len() * 8,
            Tag::List(x) => x.len_s(),
            Tag::Compound(x) => x.len_s(),
        }
    }
}

impl Tag {
    #[inline]
    pub const fn id(&self) -> TagType {
        match self {
            Tag::Byte(_) => TagType::Byte,
            Tag::Short(_) => TagType::Short,
            Tag::Int(_) => TagType::Int,
            Tag::Long(_) => TagType::Long,
            Tag::Float(_) => TagType::Float,
            Tag::Double(_) => TagType::Double,
            Tag::String(_) => TagType::String,
            Tag::ByteArray(_) => TagType::ByteArray,
            Tag::IntArray(_) => TagType::IntArray,
            Tag::LongArray(_) => TagType::LongArray,
            Tag::List(_) => TagType::List,
            Tag::Compound(_) => TagType::Compound,
        }
    }
}

impl From<bool> for Tag {
    #[inline]
    fn from(value: bool) -> Self {
        Self::Byte(value as i8)
    }
}

impl From<u8> for Tag {
    #[inline]
    fn from(value: u8) -> Self {
        Self::Byte(value as i8)
    }
}

impl From<i8> for Tag {
    fn from(value: i8) -> Self {
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
        let mut me = core::mem::ManuallyDrop::new(value);
        Self::ByteArray(unsafe {
            Vec::from_raw_parts(me.as_mut_ptr().cast::<i8>(), me.len(), me.capacity())
        })
    }
}

impl From<Vec<i8>> for Tag {
    #[inline]
    fn from(value: Vec<i8>) -> Self {
        Self::ByteArray(value)
    }
}

impl<'a> From<&'a str> for Tag {
    #[inline]
    fn from(value: &'a str) -> Self {
        unsafe { Self::String(BoxStr::new_unchecked(Box::from(value.as_bytes()))) }
    }
}

impl<'a> From<&'a mut str> for Tag {
    #[inline]
    fn from(value: &'a mut str) -> Self {
        unsafe { Self::String(BoxStr::new_unchecked(Box::from(value.as_bytes()))) }
    }
}

impl From<BoxStr> for Tag {
    #[inline]
    fn from(value: BoxStr) -> Self {
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

#[derive(Clone)]
#[repr(transparent)]
pub struct Compound(Vec<(BoxStr, Tag)>);

impl AsRef<[(BoxStr, Tag)]> for Compound {
    #[inline]
    fn as_ref(&self) -> &[(BoxStr, Tag)] {
        self.0.as_slice()
    }
}

impl AsMut<[(BoxStr, Tag)]> for Compound {
    #[inline]
    fn as_mut(&mut self) -> &mut [(BoxStr, Tag)] {
        self.0.as_mut_slice()
    }
}

impl Write for Compound {
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            for (name, tag) in &self.0 {
                tag.id().write(w);
                RefStringTag(name).write(w);
                match tag {
                    Tag::Byte(x) => x.write(w),
                    Tag::Short(x) => x.write(w),
                    Tag::Int(x) => x.write(w),
                    Tag::Long(x) => x.write(w),
                    Tag::Float(x) => x.write(w),
                    Tag::Double(x) => x.write(w),
                    Tag::String(x) => RefStringTag(x).write(w),
                    Tag::ByteArray(x) => {
                        (x.len() as u32).write(w);
                        w.write(&*(x.as_slice() as *const [i8] as *const [u8]))
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
            TagType::End.write(w);
        }
    }

    fn len_s(&self) -> usize {
        let mut w = 1 + self.0.len();
        for (name, tag) in &self.0 {
            w += RefStringTag(name).len_s();
            w += match tag {
                Tag::Byte(_) => 1,
                Tag::Short(_) => 2,
                Tag::Int(_) => 4,
                Tag::Long(_) => 8,
                Tag::Float(_) => 4,
                Tag::Double(_) => 8,
                Tag::String(x) => RefStringTag(x).len_s(),
                Tag::ByteArray(x) => 4 + x.len(),
                Tag::IntArray(x) => 4 + x.len() * 4,
                Tag::LongArray(x) => 4 + x.len() * 8,
                Tag::List(x) => x.len_s(),
                Tag::Compound(x) => Write::len_s(x),
            };
        }
        w
    }
}

#[derive(Clone)]
pub struct NamedCompound(pub BoxStr, pub Compound);

impl Read<'_> for NamedCompound {
    #[inline]
    fn read(n: &mut &[u8]) -> Result<Self, Error> {
        if matches!(TagType::read(n)?, TagType::Compound) {
            Ok(Self(StringTag::read(n)?.0, decode_raw(n)?))
        } else {
            Err(Error)
        }
    }
}

impl Write for NamedCompound {
    #[inline]
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            TagType::Compound.write(w);
            RefStringTag(self.0.as_str()).write(w);
            self.1.write(w);
        }
    }

    #[inline]
    fn len_s(&self) -> usize {
        1 + Write::len_s(&RefStringTag(self.0.as_str())) + Write::len_s(&self.1)
    }
}

#[derive(Clone)]
pub struct UnamedCompound(pub Compound);

impl Read<'_> for UnamedCompound {
    #[inline]
    fn read(n: &mut &[u8]) -> Result<Self, Error> {
        if matches!(TagType::read(n)?, TagType::Compound) {
            Ok(Self(decode_raw(n)?))
        } else {
            Err(Error)
        }
    }
}

impl Write for UnamedCompound {
    #[inline]
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            TagType::Compound.write(w);
            self.0.write(w);
        }
    }

    #[inline]
    fn len_s(&self) -> usize {
        1 + Write::len_s(&self.0)
    }
}

impl From<Compound> for UnamedCompound {
    #[inline]
    fn from(value: Compound) -> Self {
        Self(value)
    }
}

impl<K: Into<BoxStr>, V> FromIterator<(K, V)> for Compound
where
    Tag: From<V>,
{
    #[inline]
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        Self(
            iter.into_iter()
                .map(|(x, y)| (x.into(), Tag::from(y)))
                .collect(),
        )
    }
}

impl Default for Compound {
    fn default() -> Self {
        Self::new()
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
    pub fn iter(&self) -> core::slice::Iter<'_, (BoxStr, Tag)> {
        self.0.iter()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> core::slice::IterMut<'_, (BoxStr, Tag)> {
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
    pub unsafe fn get_unchecked_mut(&mut self, index: usize) -> (&str, &mut Tag) {
        let (x, y) = unsafe { self.0.get_unchecked_mut(index) };
        (x.as_str(), y)
    }

    #[inline]
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    #[inline]
    pub fn push(&mut self, k: impl Into<BoxStr>, v: impl Into<Tag>) {
        self.push_(k.into(), v.into());
    }

    #[inline]
    fn push_(&mut self, k: BoxStr, v: Tag) {
        self.0.push((k, v));
    }

    #[deprecated]
    #[inline]
    pub fn decode(buf: &mut &[u8]) -> Result<Self, Error> {
        match UnamedCompound::read(buf) {
            Ok(x) => Ok(x.0),
            Err(e) => Err(e),
        }
    }

    #[deprecated]
    #[inline]
    pub fn decode_named(buf: &mut &[u8]) -> Result<(BoxStr, Self), Error> {
        match NamedCompound::read(buf) {
            Ok(x) => Ok((x.0, x.1)),
            Err(e) => Err(e),
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
    pub fn into_inner(self) -> Vec<(BoxStr, Tag)> {
        self.0
    }
}

impl Read<'_> for Compound {
    #[inline]
    fn read(buf: &mut &[u8]) -> Result<Self, Error> {
        decode_raw(buf)
    }
}

impl From<Vec<(BoxStr, Tag)>> for Compound {
    #[inline]
    fn from(value: Vec<(BoxStr, Tag)>) -> Self {
        Self(value)
    }
}

fn decode_raw(n: &mut &[u8]) -> Result<Compound, Error> {
    let mut compound = Compound(Default::default());
    loop {
        let ty = TagType::read(n)?;
        if mser::unlikely(matches!(ty, TagType::End)) {
            compound.0.shrink_to_fit();
            return Ok(compound);
        }
        let k = StringTag::read(n)?.0;
        let v = match ty {
            TagType::Byte => Tag::from(n.u8()?),
            TagType::Short => Tag::from(n.i16()?),
            TagType::Int => Tag::from(n.i32()?),
            TagType::Long => Tag::from(n.i64()?),
            TagType::Float => Tag::from(n.f32()?),
            TagType::Double => Tag::from(n.f64()?),
            TagType::ByteArray => {
                let len = n.i32()? as usize;
                let v = Vec::from(n.slice(len)?);
                Tag::from(v)
            }
            TagType::String => Tag::from(StringTag::read(n)?.0),
            TagType::List => {
                let info = ListInfo::read(n)?;
                Tag::from(list::decode_raw(n, info)?)
            }
            TagType::Compound => Tag::from(decode_raw(n)?),
            TagType::IntArray => {
                let len = n.i32()? as usize;
                let mut slice = n.slice(len * 4)?;
                let mut v = Vec::with_capacity(len);
                for _ in 0..len {
                    v.push(slice.i32()?);
                }
                Tag::from(v)
            }
            TagType::LongArray => {
                let len = n.i32()? as usize;
                let mut slice = n.slice(len * 8)?;
                let mut v = Vec::with_capacity(len);
                for _ in 0..len {
                    v.push(slice.i64()?);
                }
                Tag::from(v)
            }
            TagType::End => unsafe { core::hint::unreachable_unchecked() },
        };
        compound.push_(k, v);
    }
}

#[derive(Clone)]
#[repr(transparent)]
pub struct StringTag(pub BoxStr);

impl Read<'_> for StringTag {
    #[inline]
    fn read(buf: &mut &'_ [u8]) -> Result<Self, Error> {
        let len = buf.u16()? as usize;
        let a = buf.slice(len)?;

        if is_ascii_mutf8(a) {
            unsafe { Ok(Self(BoxStr::new_unchecked(Box::from(a)))) }
        } else {
            let len = decode_mutf8_len(a)?;
            let mut x = Vec::with_capacity(len);
            unsafe {
                mser::write_unchecked(x.as_mut_ptr(), &(DecodeMutf8(a, len)));
                x.set_len(len);
                Ok(Self(BoxStr::new_unchecked(x.into_boxed_slice())))
            }
        }
    }
}

struct DecodeMutf8<'a>(&'a [u8], usize);

impl Write for DecodeMutf8<'_> {
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe { decode_mutf8(self.0, w).unwrap_unchecked() }
    }

    fn len_s(&self) -> usize {
        self.1
    }
}

impl Write for StringTag {
    #[inline]
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe { RefStringTag(&self.0).write(w) }
    }

    #[inline]
    fn len_s(&self) -> usize {
        RefStringTag(&self.0).len_s()
    }
}
