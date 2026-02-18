#![no_std]

extern crate alloc;

mod byte_array;
mod compound;
mod int_array;
mod list;
mod long_array;
mod string;
mod stringify;

use self::byte_array::ByteArray;
use self::int_array::IntArray;
use self::list::ListRec;
use self::long_array::LongArray;
pub use self::string::{RefStringTag, StringTag, StringTagRaw};
pub use self::stringify::CompoundStringify;
use alloc::boxed::Box;
use alloc::vec::Vec;
use haya_str::HayaStr;
use mser::{Error, Read, UnsafeWriter, Write};

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

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct Compound(Vec<(Name, Tag)>);

#[derive(Clone, Debug)]
pub enum Name {
    Thin(HayaStr),
    Heap(Box<str>),
}

#[derive(Clone, Copy)]
pub struct ListInfo(pub TagType, pub u32);

#[derive(Clone, Debug)]
pub enum ListTag {
    None,
    Byte(Vec<i8>),
    Short(Vec<i16>),
    Int(Vec<i32>),
    Long(Vec<i64>),
    Float(Vec<f32>),
    Double(Vec<f64>),
    String(Vec<Box<str>>),
    ByteArray(Vec<Vec<i8>>),
    IntArray(Vec<Vec<i32>>),
    LongArray(Vec<Vec<i64>>),
    List(Vec<ListTag>),
    Compound(Vec<Compound>),
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

    pub fn string(self, buf: &mut &[u8]) -> Result<Box<str>, Error> {
        match self {
            Self::String => match StringTag::read(buf) {
                Ok(x) => Ok(x.0),
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
                    TagType::Int => match buf.split_at_checked(len.checked_mul(4).ok_or(Error)?) {
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

    fn tag_no_rec(self, n: &mut &[u8]) -> Result<Result<Tag, TagRec>, Error> {
        Ok(Ok(match self {
            Self::Byte => Tag::from(i8::read(n)?),
            Self::Short => Tag::from(i16::read(n)?),
            Self::Int => Tag::from(i32::read(n)?),
            Self::Long => Tag::from(i64::read(n)?),
            Self::Float => Tag::from(f32::read(n)?),
            Self::Double => Tag::from(f64::read(n)?),
            Self::ByteArray => Tag::from(ByteArray::read(n)?.0),
            Self::String => Tag::from(StringTag::read(n)?.0),
            Self::IntArray => Tag::from(IntArray::read(n)?.0),
            Self::LongArray => Tag::from(LongArray::read(n)?.0),
            Self::List => return Ok(Err(TagRec::List)),
            Self::Compound => return Ok(Err(TagRec::Compound)),
            Self::End => return Err(Error),
        }))
    }
}

impl TagType {
    pub fn tag(self, n: &mut &[u8]) -> Result<Tag, Error> {
        let t = match self.tag_no_rec(n)? {
            Ok(x) => return Ok(x),
            Err(TagRec::List) => {
                let l = ListInfo::read(n)?;
                match l.list_no_rec(n)? {
                    Ok(x) => return Ok(Tag::List(x)),
                    Err(e) => match e {
                        ListRec::Compound => Entry::ListCompound(Vec::new(), l.1),
                        ListRec::List => Entry::ListList(Vec::new(), l.1),
                    },
                }
            }
            Err(TagRec::Compound) => Entry::Compound(Compound::new()),
        };
        read_tag(n, t, 512)
    }
}

enum Entry {
    Compound(Compound),
    ListCompound(Vec<Compound>, u32),
    ListList(Vec<ListTag>, u32),
}

fn read_tag(buf: &mut &[u8], mut next: Entry, max_depth: usize) -> Result<Tag, Error> {
    let mut blocks = Vec::<Entry>::new();
    let mut names = Vec::<Name>::new();
    loop {
        if max_depth == blocks.len() {
            return Err(Error);
        }
        match next {
            Entry::Compound(mut compound) => {
                let ty = TagType::read(buf)?;
                if let TagType::End = ty {
                    next = match blocks.pop() {
                        Some(Entry::Compound(mut c)) => {
                            let k = match names.pop() {
                                Some(x) => x,
                                None => return Err(Error),
                            };
                            c.push(k, Tag::Compound(compound));
                            Entry::Compound(c)
                        }
                        Some(Entry::ListCompound(mut l, len)) => {
                            l.push(compound);
                            Entry::ListCompound(l, len)
                        }
                        Some(Entry::ListList(_, _)) => return Err(Error),
                        None => return Ok(Tag::Compound(compound)),
                    };
                } else {
                    let name = Name::read(buf)?;
                    match ty.tag_no_rec(buf)? {
                        Ok(t) => {
                            compound.push(name, t);
                            next = Entry::Compound(compound);
                        }
                        Err(TagRec::Compound) => {
                            names.push(name);
                            blocks.push(Entry::Compound(compound));
                            next = Entry::Compound(Compound::new());
                        }
                        Err(TagRec::List) => {
                            let l = ListInfo::read(buf)?;
                            match l.list_no_rec(buf)? {
                                Ok(x) => {
                                    compound.push(name, Tag::List(x));
                                    next = Entry::Compound(compound);
                                }
                                Err(e) => {
                                    names.push(name);
                                    blocks.push(Entry::Compound(compound));
                                    next = match e {
                                        ListRec::Compound => Entry::ListCompound(Vec::new(), l.1),
                                        ListRec::List => Entry::ListList(Vec::new(), l.1),
                                    };
                                }
                            }
                        }
                    };
                }
            }
            Entry::ListCompound(compounds, len) => {
                if len == 0 {
                    next = match blocks.pop() {
                        Some(Entry::Compound(mut x)) => {
                            let k = match names.pop() {
                                Some(x) => x,
                                None => return Err(Error),
                            };
                            x.push(k, Tag::List(ListTag::Compound(compounds)));
                            Entry::Compound(x)
                        }
                        Some(Entry::ListList(mut lists, len)) => {
                            lists.push(ListTag::Compound(compounds));
                            Entry::ListList(lists, len)
                        }
                        Some(Entry::ListCompound(_, _)) => return Err(Error),
                        None => return Ok(Tag::List(ListTag::Compound(compounds))),
                    };
                } else {
                    blocks.push(Entry::ListCompound(compounds, len - 1));
                    next = Entry::Compound(Compound::new());
                }
            }
            Entry::ListList(mut lists, len) => {
                if len == 0 {
                    next = match blocks.pop() {
                        Some(Entry::Compound(mut x)) => {
                            let k = match names.pop() {
                                Some(x) => x,
                                None => return Err(Error),
                            };
                            x.push(k, Tag::List(ListTag::List(lists)));
                            Entry::Compound(x)
                        }
                        Some(Entry::ListList(mut x, len)) => {
                            x.push(ListTag::List(lists));
                            Entry::ListList(x, len)
                        }
                        Some(Entry::ListCompound(_, _)) => return Err(Error),
                        None => return Ok(Tag::List(ListTag::List(lists))),
                    };
                } else {
                    let l = ListInfo::read(buf)?;
                    match l.list_no_rec(buf)? {
                        Ok(x) => {
                            lists.push(x);
                            next = Entry::ListList(lists, len - 1);
                        }
                        Err(e) => {
                            blocks.push(Entry::ListList(lists, len - 1));
                            next = match e {
                                ListRec::Compound => Entry::ListCompound(Vec::new(), l.1),
                                ListRec::List => Entry::ListList(Vec::new(), l.1),
                            };
                        }
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
enum TagRec {
    List,
    Compound,
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

#[derive(Clone, Debug)]
pub enum Tag {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    String(Box<str>),
    ByteArray(Vec<i8>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
    List(ListTag),
    Compound(Compound),
}

#[derive(Clone)]
pub(crate) enum TagPrimitive {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
}

impl From<TagPrimitive> for Tag {
    fn from(value: TagPrimitive) -> Self {
        match value {
            TagPrimitive::Byte(x) => Self::Byte(x),
            TagPrimitive::Short(x) => Self::Short(x),
            TagPrimitive::Int(x) => Self::Int(x),
            TagPrimitive::Long(x) => Self::Long(x),
            TagPrimitive::Float(x) => Self::Float(x),
            TagPrimitive::Double(x) => Self::Double(x),
        }
    }
}

#[derive(Clone)]
pub(crate) enum TagArray {
    Byte(Vec<i8>),
    Int(Vec<i32>),
    Long(Vec<i64>),
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
                    x.iter().for_each(|i| i.write(w));
                }
                Tag::LongArray(x) => {
                    (x.len() as u32).write(w);
                    x.iter().for_each(|i| i.write(w));
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

impl From<&Tag> for TagType {
    fn from(value: &Tag) -> Self {
        value.id()
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
        Self::String(Box::from(value))
    }
}

impl<'a> From<&'a mut str> for Tag {
    #[inline]
    fn from(value: &'a mut str) -> Self {
        Self::String(Box::from(value))
    }
}

impl From<Box<str>> for Tag {
    #[inline]
    fn from(value: Box<str>) -> Self {
        Self::String(value)
    }
}

impl From<ListTag> for Tag {
    #[inline]
    fn from(value: ListTag) -> Self {
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

#[derive(Clone)]
pub struct CompoundNamed(pub Name, pub Compound);

impl Read<'_> for CompoundNamed {
    #[inline]
    fn read(n: &mut &[u8]) -> Result<Self, Error> {
        if matches!(TagType::read(n)?, TagType::Compound) {
            Ok(Self(Name::read(n)?, Compound::read(n)?))
        } else {
            Err(Error)
        }
    }
}

impl Write for CompoundNamed {
    #[inline]
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            TagType::Compound.write(w);
            RefStringTag(&self.0).write(w);
            self.1.write(w);
        }
    }

    #[inline]
    fn len_s(&self) -> usize {
        1 + Write::len_s(&RefStringTag(&self.0)) + Write::len_s(&self.1)
    }
}
