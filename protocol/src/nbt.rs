mod byte_array;
mod compound;
mod int_array;
mod list;
mod long_array;
mod string;
mod stringify;

use self::byte_array::u8_to_i8_slice;
pub use self::compound::{Compound, CompoundNamed, CompoundUnamed};
use self::int_array::IntArray;
pub use self::list::{List, ListInfo};
use self::long_array::LongArray;
use self::string::DecodeMutf8;
pub use self::string::{IdentifierTag, RefStringTag, StringTag, StringTagRaw};
pub use self::stringify::StringifyCompound;
use crate::str::BoxStr;
use crate::{Error, Ident, Identifier, Read, UnsafeWriter, Write};
use alloc::boxed::Box;
use alloc::vec::Vec;
use mser::{decode_mutf8_len, is_ascii_mutf8};
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

pub trait MapReader<T: Sized = Self>: Sized {
    fn visit(&mut self, ty: TagType, k: &[u8], buf: &mut &[u8]) -> Result<(), Error>;
    fn end(self) -> Result<T, Error>;
    fn read_map(mut self, buf: &mut &[u8]) -> Result<T, Error> {
        let mut temp = Vec::new();
        loop {
            let ty = TagType::read(buf)?;
            if matches!(ty, TagType::End) {
                mser::cold_path();
                return self.end();
            }
            let len = u16::read(buf)? as usize;
            let a = match buf.split_at_checked(len) {
                Some((x, y)) => {
                    *buf = y;
                    x
                }
                None => return Err(Error),
            };
            let k = if is_ascii_mutf8(a) {
                a
            } else {
                let len = decode_mutf8_len(a)?;
                temp.clear();
                temp.reserve_exact(len);
                unsafe {
                    mser::write_unchecked(temp.as_mut_ptr(), &(DecodeMutf8(a, len)));
                    temp.set_len(len);
                    &temp
                }
            };
            self.visit(ty, k, buf)?;
        }
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

impl TagType {
    pub fn bool(self, buf: &mut &[u8]) -> Result<bool, Error> {
        match self {
            Self::Byte => Ok(i8::read(buf)? != 0),
            Self::Short => Ok(i16::read(buf)? != 0),
            Self::Int => Ok(i32::read(buf)? != 0),
            Self::Long => Ok(i64::read(buf)? != 0),
            Self::Float => Ok(f32::read(buf)? != 0.0),
            Self::Double => Ok(f64::read(buf)? != 0.0),
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
            Self::IntArray => Ok(IntArray::read(buf)?.0),
            Self::List => {
                let ListInfo(tag, len) = ListInfo::read(buf)?;
                let len = len as usize;
                match tag {
                    TagType::Int => match buf.split_at_checked(len * 4) {
                        Some((slice, y)) => unsafe {
                            *buf = y;
                            Ok(list::int_list(len, slice))
                        },
                        None => Err(Error),
                    },
                    _ => Err(Error),
                }
            }
            _ => Err(Error),
        }
    }

    pub fn tag(self, n: &mut &[u8]) -> Result<Tag, Error> {
        Ok(match self {
            Self::Byte => Tag::from(i8::read(n)?),
            Self::Short => Tag::from(i16::read(n)?),
            Self::Int => Tag::from(i32::read(n)?),
            Self::Long => Tag::from(i64::read(n)?),
            Self::Float => Tag::from(f32::read(n)?),
            Self::Double => Tag::from(f64::read(n)?),
            Self::ByteArray => match n.split_at_checked(i32::read(n)? as usize) {
                Some((x, y)) => {
                    *n = y;
                    Tag::from(Vec::from(u8_to_i8_slice(x)))
                }
                None => return Err(Error),
            },
            Self::String => Tag::from(StringTag::read(n)?.0),
            Self::List => {
                let info = ListInfo::read(n)?;
                Tag::from(info.list(n)?)
            }
            Self::Compound => Tag::from(Compound::read(n)?),
            Self::IntArray => Tag::from(IntArray::read(n)?.0),
            Self::LongArray => Tag::from(LongArray::read(n)?.0),
            Self::End => unsafe { core::hint::unreachable_unchecked() },
        })
    }
}

impl Read<'_> for TagType {
    #[inline]
    fn read(buf: &mut &'_ [u8]) -> Result<Self, Error> {
        let t = u8::read(buf)?;
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
        let s = match self {
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
        };
        self.id().len_s() + s
    }
}

impl Tag {
    #[inline]
    pub const fn id(&self) -> TagType {
        match self {
            Self::Byte(_) => TagType::Byte,
            Self::Short(_) => TagType::Short,
            Self::Int(_) => TagType::Int,
            Self::Long(_) => TagType::Long,
            Self::Float(_) => TagType::Float,
            Self::Double(_) => TagType::Double,
            Self::String(_) => TagType::String,
            Self::ByteArray(_) => TagType::ByteArray,
            Self::IntArray(_) => TagType::IntArray,
            Self::LongArray(_) => TagType::LongArray,
            Self::List(_) => TagType::List,
            Self::Compound(_) => TagType::Compound,
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

impl Read<'_> for Tag {
    fn read(buf: &mut &'_ [u8]) -> Result<Self, Error> {
        TagType::read(buf)?.tag(buf)
    }
}
