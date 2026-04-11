#![no_std]

extern crate alloc;

mod byte_array;
mod compound;
mod int_array;
mod list;
mod long_array;
mod string;
//mod stringify;

use self::byte_array::ByteArray;
use self::int_array::IntArray;
use self::long_array::LongArray;
// pub use self::stringify::CompoundStringify;
use alloc::boxed::Box;
use alloc::vec::Vec;
use haya_str::HayaStr;
use mser::{Error, Read, Reader, Write, Writer};

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

#[derive(Clone)]
pub struct StringTag(pub Box<str>);

#[derive(Clone, Copy)]
pub struct RefStringTag<'a>(pub &'a str);

#[derive(Clone, Copy)]
pub struct RawStringTag<'a>(&'a str);

#[derive(Clone, Debug)]
pub struct Compound(Vec<(Name, Tag)>);

#[derive(Clone)]
pub struct CompoundStringify(pub Compound);

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
    pub fn bool(self, buf: &mut Reader) -> Result<bool, Error> {
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

    pub fn string(self, buf: &mut Reader) -> Result<Box<str>, Error> {
        match self {
            Self::String => match StringTag::read(buf) {
                Ok(x) => Ok(x.0),
                Err(e) => Err(e),
            },
            _ => Err(Error),
        }
    }

    pub fn int_list(self, buf: &mut Reader) -> Result<Vec<i32>, Error> {
        match self {
            Self::IntArray => Ok(IntArray::read(buf)?.0),
            Self::List => {
                let ListInfo(tag, len) = ListInfo::read(buf)?;
                let len = len as usize;
                match tag {
                    TagType::Int => unsafe {
                        Ok(list::int_list(
                            len,
                            buf.read_slice(len.checked_mul(4).ok_or(Error)?)?,
                        ))
                    },
                    _ => Err(Error),
                }
            }
            _ => Err(Error),
        }
    }

    fn tag_no_rec(self, n: &mut Reader) -> Result<Tag, Error> {
        Ok(match self {
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
            Self::List => Tag::List(ListTag::None),
            Self::Compound => Tag::Compound(Compound::new()),
            Self::End => return Err(Error),
        })
    }
}

impl TagType {
    pub fn tag(self, n: &mut Reader) -> Result<Tag, Error> {
        let t = match self.tag_no_rec(n)? {
            Tag::List(ListTag::None) => {
                let l = ListInfo::read(n)?;
                match l.list_no_rec(n)? {
                    ListTag::Compound(x) => ReadEntry::ListCompound(x, l.1),
                    ListTag::List(x) => ReadEntry::ListList(x, 1),
                    x => return Ok(Tag::List(x)),
                }
            }
            Tag::Compound(c) => ReadEntry::Compound(c),
            x => return Ok(x),
        };
        read_tag(n, t, 512)
    }
}

#[derive(Clone)]
enum ReadEntry {
    Compound(Compound),
    ListCompound(Vec<Compound>, u32),
    ListList(Vec<ListTag>, u32),
}

fn read_tag(buf: &mut Reader, next: ReadEntry, max_depth: usize) -> Result<Tag, Error> {
    let mut blocks = Vec::<ReadEntry>::with_capacity(4);
    let mut names = Vec::<Name>::with_capacity(4);
    blocks.push(next);
    loop {
        blocks.reserve(1);
        names.reserve(1);
        let next = unsafe { blocks.pop().unwrap_unchecked() };
        let next = match next {
            ReadEntry::Compound(mut compound) => match TagType::read(buf)? {
                TagType::End => match blocks.pop() {
                    Some(ReadEntry::Compound(mut c)) => {
                        let k = match names.pop() {
                            Some(x) => x,
                            None => return Err(Error),
                        };
                        c.push(k, Tag::Compound(compound));
                        ReadEntry::Compound(c)
                    }
                    Some(ReadEntry::ListCompound(mut l, len)) => {
                        l.push(compound);
                        ReadEntry::ListCompound(l, len)
                    }
                    Some(ReadEntry::ListList(_, _)) => return Err(Error),
                    None => return Ok(Tag::Compound(compound)),
                },
                ty => {
                    let name = Name::read(buf)?;
                    match ty.tag_no_rec(buf)? {
                        Tag::Compound(c) => {
                            names.push(name);
                            blocks.push(ReadEntry::Compound(compound));
                            ReadEntry::Compound(c)
                        }
                        Tag::List(ListTag::None) => {
                            read_list1(buf, &mut blocks, &mut names, compound, name)?
                        }
                        t => {
                            compound.push(name, t);
                            ReadEntry::Compound(compound)
                        }
                    }
                }
            },
            ReadEntry::ListCompound(compounds, len) => {
                if len != 0 {
                    blocks.push(ReadEntry::ListCompound(compounds, len - 1));
                    ReadEntry::Compound(Compound::new())
                } else {
                    match blocks.pop() {
                        Some(ReadEntry::Compound(mut x)) => {
                            let k = match names.pop() {
                                Some(x) => x,
                                None => return Err(Error),
                            };
                            x.push(k, Tag::List(ListTag::Compound(compounds)));
                            ReadEntry::Compound(x)
                        }
                        Some(ReadEntry::ListList(mut lists, len)) => {
                            lists.push(ListTag::Compound(compounds));
                            ReadEntry::ListList(lists, len)
                        }
                        Some(ReadEntry::ListCompound(_, _)) => return Err(Error),
                        None => return Ok(Tag::List(ListTag::Compound(compounds))),
                    }
                }
            }
            ReadEntry::ListList(lists, len) => {
                if len != 0 {
                    read_list2(buf, &mut blocks, lists, len)?
                } else {
                    match blocks.pop() {
                        Some(ReadEntry::Compound(mut x)) => {
                            let k = match names.pop() {
                                Some(x) => x,
                                None => return Err(Error),
                            };
                            x.push(k, Tag::List(ListTag::List(lists)));
                            ReadEntry::Compound(x)
                        }
                        Some(ReadEntry::ListList(mut x, len)) => {
                            x.push(ListTag::List(lists));
                            ReadEntry::ListList(x, len)
                        }
                        Some(ReadEntry::ListCompound(_, _)) => return Err(Error),
                        None => return Ok(Tag::List(ListTag::List(lists))),
                    }
                }
            }
        };
        blocks.push(next);
        if max_depth == blocks.len() {
            return Err(Error);
        }
    }
}

#[inline]
fn read_list2(
    buf: &mut Reader<'_>,
    blocks: &mut Vec<ReadEntry>,
    mut lists: Vec<ListTag>,
    len: u32,
) -> Result<ReadEntry, Error> {
    let l = ListInfo::read(buf)?;
    Ok(match l.list_no_rec(buf)? {
        ListTag::List(x) => {
            blocks.push(ReadEntry::ListList(lists, len - 1));
            ReadEntry::ListList(x, l.1)
        }
        ListTag::Compound(x) => {
            blocks.push(ReadEntry::ListList(lists, len - 1));
            ReadEntry::ListCompound(x, l.1)
        }
        x => {
            lists.push(x);
            ReadEntry::ListList(lists, len - 1)
        }
    })
}

#[inline]
fn read_list1(
    buf: &mut Reader<'_>,
    blocks: &mut Vec<ReadEntry>,
    names: &mut Vec<Name>,
    mut compound: Compound,
    name: Name,
) -> Result<ReadEntry, Error> {
    let l = ListInfo::read(buf)?;
    Ok(match l.list_no_rec(buf)? {
        ListTag::List(x) => {
            names.push(name);
            blocks.push(ReadEntry::Compound(compound));
            ReadEntry::ListList(x, l.1)
        }
        ListTag::Compound(x) => {
            names.push(name);
            blocks.push(ReadEntry::Compound(compound));
            ReadEntry::ListCompound(x, l.1)
        }
        x => {
            compound.push(name, Tag::List(x));
            ReadEntry::Compound(compound)
        }
    })
}

#[derive(Clone, Copy)]
enum WriteEntry<'a> {
    Compound(&'a [(Name, Tag)]),
    ListCompound(&'a [Compound]),
    ListList(&'a [ListTag]),
}

unsafe fn write_no_rec(w: &mut Writer, t: &Tag) {
    unsafe {
        match t {
            Tag::Byte(x) => x.write(w),
            Tag::Short(x) => x.write(w),
            Tag::Int(x) => x.write(w),
            Tag::Long(x) => x.write(w),
            Tag::Float(x) => x.write(w),
            Tag::Double(x) => x.write(w),
            Tag::String(x) => RefStringTag(x).write(w),
            Tag::ByteArray(x) => {
                (x.len() as u32).write(w);
                w.write(byte_array::i8_to_u8_slice(x.as_slice()));
            }
            Tag::IntArray(x) => {
                (x.len() as u32).write(w);
                x.iter().for_each(|i| i.write(w));
            }
            Tag::LongArray(x) => {
                (x.len() as u32).write(w);
                x.iter().for_each(|i| i.write(w));
            }
            Tag::List(_) => {}
            Tag::Compound(_) => {}
        }
    }
}

fn len_no_rec(t: &Tag) -> usize {
    match t {
        Tag::Byte(x) => x.len_s(),
        Tag::Short(x) => x.len_s(),
        Tag::Int(x) => x.len_s(),
        Tag::Long(x) => x.len_s(),
        Tag::Float(x) => x.len_s(),
        Tag::Double(x) => x.len_s(),
        Tag::String(x) => RefStringTag(x).len_s(),
        Tag::ByteArray(x) => (x.len() as u32).len_s() + x.len(),
        Tag::IntArray(x) => (x.len() as u32).len_s() + 4 * x.len(),
        Tag::LongArray(x) => (x.len() as u32).len_s() + 8 * x.len(),
        Tag::List(_) => 0,
        Tag::Compound(_) => 0,
    }
}

fn len_list_no_rec(list: &ListTag) -> usize {
    list.list_info().len_s()
        + match list {
            ListTag::None => 0,
            ListTag::Byte(x) => x.len(),
            ListTag::Short(x) => x.len() * 2,
            ListTag::Int(x) => x.len() * 4,
            ListTag::Long(x) => x.len() * 8,
            ListTag::Float(x) => x.len() * 4,
            ListTag::Double(x) => x.len() * 8,
            ListTag::String(x) => x.iter().map(|x| RefStringTag(x).len_s()).sum::<usize>(),
            ListTag::ByteArray(x) => x.len() * 4 + x.iter().map(|x| x.len()).sum::<usize>(),
            ListTag::IntArray(x) => x.len() * 4 + x.iter().map(|x| x.len()).sum::<usize>() * 4,
            ListTag::LongArray(x) => x.len() * 4 + x.iter().map(|x| x.len()).sum::<usize>() * 8,
            ListTag::List(_) => 0,
            ListTag::Compound(_) => 0,
        }
}

unsafe fn write_list_no_rec(w: &mut Writer, list: &ListTag) {
    unsafe {
        list.list_info().write(w);
        match list {
            ListTag::None => {}
            ListTag::Byte(x) => {
                w.write(crate::byte_array::i8_to_u8_slice(x));
            }
            ListTag::Short(x) => {
                x.iter().for_each(|x| x.write(w));
            }
            ListTag::Int(x) => {
                x.iter().for_each(|x| x.write(w));
            }
            ListTag::Long(x) => {
                x.iter().for_each(|x| x.write(w));
            }
            ListTag::Float(x) => {
                x.iter().for_each(|x| x.write(w));
            }
            ListTag::Double(x) => {
                x.iter().for_each(|x| x.write(w));
            }
            ListTag::String(x) => {
                x.iter().for_each(|x| RefStringTag(x).write(w));
            }
            ListTag::ByteArray(x) => {
                x.iter().for_each(|y| {
                    (y.len() as u32).write(w);
                    crate::byte_array::i8_to_u8_slice(y).write(w);
                });
            }
            ListTag::IntArray(x) => {
                x.iter().for_each(|y| {
                    (y.len() as u32).write(w);
                    y.iter().for_each(|z| z.write(w));
                });
            }
            ListTag::LongArray(x) => {
                x.iter().for_each(|y| {
                    (y.len() as u32).write(w);
                    y.iter().for_each(|z| z.write(w));
                });
            }
            ListTag::List(_) => {}
            ListTag::Compound(_) => {}
        }
    }
}

fn write_tag(w: &mut Writer, next: WriteEntry) {
    let mut blocks = Vec::<WriteEntry>::with_capacity(4);
    blocks.push(next);
    loop {
        unsafe {
            blocks.reserve(1);
            let next = blocks.pop().unwrap_unchecked();
            let next = match next {
                WriteEntry::Compound(compound) => match compound.split_first() {
                    Some((x, y)) => {
                        x.0.write(w);
                        match &x.1 {
                            Tag::List(ListTag::Compound(cl)) => {
                                blocks.push(WriteEntry::Compound(y));
                                WriteEntry::ListCompound(cl)
                            }
                            Tag::List(ListTag::List(ll)) => {
                                blocks.push(WriteEntry::Compound(y));
                                WriteEntry::ListList(ll)
                            }
                            Tag::List(list) => {
                                write_list_no_rec(w, list);
                                WriteEntry::Compound(y)
                            }
                            Tag::Compound(c) => {
                                blocks.push(WriteEntry::Compound(y));
                                WriteEntry::Compound(c.0.as_slice())
                            }
                            t => {
                                write_no_rec(w, t);
                                WriteEntry::Compound(y)
                            }
                        }
                    }
                    None => {
                        TagType::End.write(w);
                        match blocks.pop() {
                            Some(x) => x,
                            None => return,
                        }
                    }
                },
                WriteEntry::ListCompound(compounds) => {
                    if let Some((x, y)) = compounds.split_first() {
                        blocks.push(WriteEntry::ListCompound(y));
                        WriteEntry::Compound(x.as_ref())
                    } else {
                        match blocks.pop() {
                            Some(x) => x,
                            None => return,
                        }
                    }
                }
                WriteEntry::ListList(lists) => {
                    if let Some((x, y)) = lists.split_first() {
                        x.list_info().write(w);
                        match x {
                            ListTag::List(x) => {
                                blocks.push(WriteEntry::ListList(y));
                                WriteEntry::ListList(x)
                            }
                            ListTag::Compound(x) => {
                                blocks.push(WriteEntry::ListList(y));
                                WriteEntry::ListCompound(x)
                            }
                            x => {
                                write_list_no_rec(w, x);
                                WriteEntry::ListList(y)
                            }
                        }
                    } else {
                        match blocks.pop() {
                            Some(x) => x,
                            None => return,
                        }
                    }
                }
            };
            blocks.push(next);
        }
    }
}

fn len_tag(next: WriteEntry) -> usize {
    let mut blocks = Vec::<WriteEntry>::with_capacity(4);
    blocks.push(next);
    let mut w = 0usize;
    loop {
        unsafe {
            blocks.reserve(1);
            let next = blocks.pop().unwrap_unchecked();
            let next = match next {
                WriteEntry::Compound(compound) => match compound.split_first() {
                    Some((x, y)) => {
                        w += x.0.len_s();
                        match &x.1 {
                            Tag::List(ListTag::Compound(cl)) => {
                                blocks.push(WriteEntry::Compound(y));
                                WriteEntry::ListCompound(cl)
                            }
                            Tag::List(ListTag::List(ll)) => {
                                blocks.push(WriteEntry::Compound(y));
                                WriteEntry::ListList(ll)
                            }
                            Tag::List(list) => {
                                w += len_list_no_rec(list);
                                WriteEntry::Compound(y)
                            }
                            Tag::Compound(c) => {
                                blocks.push(WriteEntry::Compound(y));
                                WriteEntry::Compound(c.0.as_slice())
                            }
                            t => {
                                w += len_no_rec(t);
                                WriteEntry::Compound(y)
                            }
                        }
                    }
                    None => {
                        w += TagType::End.len_s();
                        match blocks.pop() {
                            Some(x) => x,
                            None => return w,
                        }
                    }
                },
                WriteEntry::ListCompound(compounds) => {
                    if let Some((x, y)) = compounds.split_first() {
                        blocks.push(WriteEntry::ListCompound(y));
                        WriteEntry::Compound(x.as_ref())
                    } else {
                        match blocks.pop() {
                            Some(x) => x,
                            None => return w,
                        }
                    }
                }
                WriteEntry::ListList(lists) => {
                    if let Some((x, y)) = lists.split_first() {
                        w += x.list_info().len_s();
                        match x {
                            ListTag::List(x) => {
                                blocks.push(WriteEntry::ListList(y));
                                WriteEntry::ListList(x)
                            }
                            ListTag::Compound(x) => {
                                blocks.push(WriteEntry::ListList(y));
                                WriteEntry::ListCompound(x)
                            }
                            x => {
                                w += len_list_no_rec(x);
                                WriteEntry::ListList(y)
                            }
                        }
                    } else {
                        match blocks.pop() {
                            Some(x) => x,
                            None => return w,
                        }
                    }
                }
            };
            blocks.push(next);
        }
    }
}

impl Read<'_> for TagType {
    #[inline]
    fn read(buf: &mut Reader) -> Result<Self, Error> {
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
    unsafe fn write(&self, w: &mut Writer) {
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
#[allow(dead_code)]
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
#[allow(dead_code)]
pub(crate) enum TagArray {
    Byte(Vec<i8>),
    Int(Vec<i32>),
    Long(Vec<i64>),
}

impl Write for Tag {
    unsafe fn write(&self, w: &mut Writer) {
        unsafe {
            self.id().write(w);
            write_tag(
                w,
                match self {
                    Self::List(ListTag::Compound(cl)) => WriteEntry::ListCompound(cl),
                    Self::List(ListTag::List(ll)) => WriteEntry::ListList(ll),
                    Self::List(list) => {
                        write_list_no_rec(w, list);
                        return;
                    }
                    Self::Compound(c) => WriteEntry::Compound(c.0.as_slice()),
                    t => {
                        write_no_rec(w, t);
                        return;
                    }
                },
            );
        }
    }

    fn len_s(&self) -> usize {
        let id = self.id().len_s();
        let e = match self {
            Self::List(ListTag::Compound(cl)) => WriteEntry::ListCompound(cl),
            Self::List(ListTag::List(ll)) => WriteEntry::ListList(ll),
            Self::List(list) => {
                return id + len_list_no_rec(list);
            }
            Self::Compound(c) => WriteEntry::Compound(c.0.as_slice()),
            t => {
                return id + len_no_rec(t);
            }
        };
        id + len_tag(e)
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

impl<'a> Read<'a> for Tag {
    fn read(buf: &mut Reader<'a>) -> Result<Self, Error> {
        TagType::read(buf)?.tag(buf)
    }
}

#[derive(Clone)]
pub struct CompoundNamed(pub Name, pub Compound);

impl Read<'_> for CompoundNamed {
    #[inline]
    fn read(n: &mut Reader) -> Result<Self, Error> {
        if matches!(TagType::read(n)?, TagType::Compound) {
            Ok(Self(Name::read(n)?, Compound::read(n)?))
        } else {
            Err(Error)
        }
    }
}

impl Write for CompoundNamed {
    #[inline]
    unsafe fn write(&self, w: &mut Writer) {
        unsafe {
            TagType::Compound.write(w);
            self.0.write(w);
            self.1.write(w);
        }
    }

    #[inline]
    fn len_s(&self) -> usize {
        1 + Write::len_s(&self.0) + Write::len_s(&self.1)
    }
}
