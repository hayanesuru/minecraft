#![no_std]
#![feature(coroutines, coroutine_trait, stmt_expr_attributes)]

extern crate alloc;

mod byte_array;
mod compound;
mod int_array;
mod list;
mod long_array;
mod string;
mod stringify;

use self::byte_array::ByteArray;
pub use self::compound::{Compound, CompoundNamed};
use self::int_array::IntArray;
use self::list::ListRec;
pub use self::list::{List, ListInfo};
use self::long_array::LongArray;
pub use self::string::{RefStringTag, StringTag, StringTagRaw};
pub use self::stringify::StringifyCompound;
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::ops::{Coroutine, CoroutineState};
use core::pin::Pin;
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
        match read_tag(Step::T(self, n), 512) {
            Ret::Ok(x, m) => {
                *n = m;
                Ok(x)
            }
            Ret::Err(e, m) => {
                *n = m;
                Err(e)
            }
        }
    }
}

#[derive(Clone, Copy)]
enum Step<'a> {
    T(TagType, &'a [u8]),
    L(ListInfo, &'a [u8]),
}

#[derive(Clone)]
enum Ret<'a> {
    Ok(Tag, &'a [u8]),
    Err(Error, &'a [u8]),
}

fn read_tag<'a>(init: Step<'a>, max_depth: usize) -> Ret<'a> {
    let f = |step| {
        #[coroutine]
        move |_| match step {
            Step::T(s, mut n) => match match s.tag_no_rec(&mut n) {
                Ok(x) => x,
                Err(e) => return Ret::Err(e, n),
            } {
                Ok(x) => Ret::Ok(x, n),
                Err(TagRec::List) => {
                    let info = match ListInfo::read(&mut n) {
                        Ok(x) => x,
                        Err(e) => return Ret::Err(e, n),
                    };
                    match yield Step::L(info, n) {
                        Ret::Ok(x, y) => Ret::Ok(x, y),
                        Ret::Err(e, n) => Ret::Err(e, n),
                    }
                }
                Err(TagRec::Compound) => {
                    let mut compound = Vec::new();
                    loop {
                        let ty = match TagType::read(&mut n) {
                            Ok(x) => x,
                            Err(e) => return Ret::Err(e, n),
                        };
                        if let TagType::End = ty {
                            compound.shrink_to_fit();
                            return Ret::Ok(Tag::Compound(Compound::from(compound)), n);
                        }
                        let k = match StringTag::read(&mut n) {
                            Ok(x) => x.0,
                            Err(e) => return Ret::Err(e, n),
                        };
                        let v = match yield Step::T(ty, n) {
                            Ret::Ok(x, y) => {
                                n = y;
                                x
                            }
                            Ret::Err(e, n) => return Ret::Err(e, n),
                        };
                        compound.push((k, v));
                    }
                }
            },
            Step::L(s, mut n) => match match s.list_no_rec(&mut n) {
                Ok(x) => x,
                Err(e) => return Ret::Err(e, n),
            } {
                Ok(x) => Ret::Ok(Tag::List(x), n),
                Err(ListRec::List) => {
                    let len = s.1 as usize;
                    if len << 2 > n.len() {
                        return Ret::Err(Error, n);
                    }
                    let mut list = Vec::with_capacity(len);
                    for _ in 0..len {
                        let info = match ListInfo::read(&mut n) {
                            Ok(x) => x,
                            Err(e) => return Ret::Err(e, n),
                        };
                        let t = match yield Step::L(info, n) {
                            Ret::Ok(x, y) => {
                                n = y;
                                x
                            }
                            Ret::Err(e, n) => return Ret::Err(e, n),
                        };
                        match t {
                            Tag::List(x) => list.push(x),
                            _ => return Ret::Err(Error, n),
                        }
                    }
                    Ret::Ok(Tag::List(List::List(list)), n)
                }
                Err(ListRec::Compound) => {
                    let len = s.1 as usize;
                    if len > n.len() {
                        return Ret::Err(Error, n);
                    }
                    let mut list = Vec::with_capacity(len);
                    for _ in 0..len {
                        let mut compound = Vec::new();
                        loop {
                            let ty = match TagType::read(&mut n) {
                                Ok(x) => x,
                                Err(e) => return Ret::Err(e, n),
                            };
                            if let TagType::End = ty {
                                compound.shrink_to_fit();
                                break;
                            }
                            let k = match StringTag::read(&mut n) {
                                Ok(x) => x.0,
                                Err(e) => return Ret::Err(e, n),
                            };
                            let v = match yield Step::T(ty, n) {
                                Ret::Ok(x, y) => {
                                    n = y;
                                    x
                                }
                                Ret::Err(e, n) => return Ret::Err(e, n),
                            };
                            compound.push((k, v));
                        }
                        list.push(Compound::from(compound));
                    }
                    Ret::Ok(Tag::List(List::Compound(list)), n)
                }
            },
        }
    };

    let mut depth = Vec::new();
    let mut current = f(init);
    let mut ret = Ret::Err(Error, &[]);

    loop {
        match Pin::new(&mut current).resume(ret) {
            CoroutineState::Yielded(arg) => {
                if depth.len() == max_depth {
                    match arg {
                        Step::T(_, n) => return Ret::Err(Error, n),
                        Step::L(_, n) => return Ret::Err(Error, n),
                    }
                }
                depth.push(current);
                current = f(arg);
                ret = Ret::Err(Error, &[]);
            }
            CoroutineState::Complete(real_res) => match depth.pop() {
                None => return real_res,
                Some(top) => {
                    current = top;
                    ret = real_res;
                }
            },
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

#[derive(Clone)]
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
    List(List),
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
