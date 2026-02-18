use crate::{
    ByteArray, Compound, IntArray, ListInfo, ListTag, LongArray, RefStringTag, StringTag, TagType,
};
use alloc::boxed::Box;
use alloc::vec::Vec;
use mser::{Error, Read, UnsafeWriter, Write};

#[derive(Clone)]
pub(crate) enum ListPrimitive {
    Byte(Vec<i8>),
    Short(Vec<i16>),
    Int(Vec<i32>),
    Long(Vec<i64>),
    Float(Vec<f32>),
    Double(Vec<f64>),
}

impl From<ListPrimitive> for ListTag {
    fn from(value: ListPrimitive) -> Self {
        match value {
            ListPrimitive::Byte(x) => Self::Byte(x),
            ListPrimitive::Short(x) => Self::Short(x),
            ListPrimitive::Int(x) => Self::Int(x),
            ListPrimitive::Long(x) => Self::Long(x),
            ListPrimitive::Float(x) => Self::Float(x),
            ListPrimitive::Double(x) => Self::Double(x),
        }
    }
}

impl<'a> Read<'a> for ListInfo {
    fn read(buf: &mut &'a [u8]) -> Result<Self, Error> {
        let t = TagType::read(buf)?;
        let l = u32::read(buf)?;
        Ok(Self(t, l))
    }
}

impl Write for ListInfo {
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            self.0.write(w);
            self.1.write(w);
        }
    }

    fn len_s(&self) -> usize {
        self.0.len_s() + self.1.len_s()
    }
}

impl From<Vec<u8>> for ListTag {
    #[inline]
    fn from(value: Vec<u8>) -> Self {
        let mut me = core::mem::ManuallyDrop::new(value);
        Self::Byte(unsafe {
            Vec::from_raw_parts(me.as_mut_ptr().cast::<i8>(), me.len(), me.capacity())
        })
    }
}

impl From<Vec<i8>> for ListTag {
    #[inline]
    fn from(value: Vec<i8>) -> Self {
        Self::Byte(value)
    }
}

impl From<Vec<i16>> for ListTag {
    #[inline]
    fn from(value: Vec<i16>) -> Self {
        Self::Short(value)
    }
}

impl From<Vec<i32>> for ListTag {
    #[inline]
    fn from(value: Vec<i32>) -> Self {
        Self::Int(value)
    }
}

impl From<Vec<i64>> for ListTag {
    #[inline]
    fn from(value: Vec<i64>) -> Self {
        Self::Long(value)
    }
}

impl From<Vec<f32>> for ListTag {
    #[inline]
    fn from(value: Vec<f32>) -> Self {
        Self::Float(value)
    }
}

impl From<Vec<f64>> for ListTag {
    #[inline]
    fn from(value: Vec<f64>) -> Self {
        Self::Double(value)
    }
}

impl From<Vec<Box<str>>> for ListTag {
    #[inline]
    fn from(value: Vec<Box<str>>) -> Self {
        Self::String(value)
    }
}

impl From<Vec<Vec<i8>>> for ListTag {
    #[inline]
    fn from(value: Vec<Vec<i8>>) -> Self {
        Self::ByteArray(value)
    }
}

impl From<Vec<Vec<i32>>> for ListTag {
    #[inline]
    fn from(value: Vec<Vec<i32>>) -> Self {
        Self::IntArray(value)
    }
}

impl From<Vec<Vec<i64>>> for ListTag {
    #[inline]
    fn from(value: Vec<Vec<i64>>) -> Self {
        Self::LongArray(value)
    }
}

impl From<Vec<ListTag>> for ListTag {
    #[inline]
    fn from(value: Vec<ListTag>) -> Self {
        Self::List(value)
    }
}

impl From<Vec<Compound>> for ListTag {
    #[inline]
    fn from(value: Vec<Compound>) -> Self {
        Self::Compound(value)
    }
}

impl ListTag {
    pub fn list_info(&self) -> ListInfo {
        match self {
            Self::None => ListInfo(TagType::End, 0),
            Self::Byte(items) => ListInfo(TagType::Byte, items.len() as _),
            Self::Short(items) => ListInfo(TagType::Short, items.len() as _),
            Self::Int(items) => ListInfo(TagType::Int, items.len() as _),
            Self::Long(items) => ListInfo(TagType::Long, items.len() as _),
            Self::Float(items) => ListInfo(TagType::Float, items.len() as _),
            Self::Double(items) => ListInfo(TagType::Double, items.len() as _),
            Self::String(items) => ListInfo(TagType::String, items.len() as _),
            Self::ByteArray(items) => ListInfo(TagType::ByteArray, items.len() as _),
            Self::IntArray(items) => ListInfo(TagType::IntArray, items.len() as _),
            Self::LongArray(items) => ListInfo(TagType::LongArray, items.len() as _),
            Self::List(items) => ListInfo(TagType::List, items.len() as _),
            Self::Compound(items) => ListInfo(TagType::Compound, items.len() as _),
        }
    }
}

impl Write for ListTag {
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            self.list_info().write(w);
            match self {
                Self::None => {}
                Self::Byte(x) => {
                    w.write(crate::byte_array::i8_to_u8_slice(x));
                }
                Self::Short(x) => {
                    x.iter().for_each(|x| x.write(w));
                }
                Self::Int(x) => {
                    x.iter().for_each(|x| x.write(w));
                }
                Self::Long(x) => {
                    x.iter().for_each(|x| x.write(w));
                }
                Self::Float(x) => {
                    x.iter().for_each(|x| x.write(w));
                }
                Self::Double(x) => {
                    x.iter().for_each(|x| x.write(w));
                }
                Self::String(x) => {
                    x.iter().for_each(|x| RefStringTag(x).write(w));
                }
                Self::ByteArray(x) => {
                    x.iter().for_each(|y| {
                        (y.len() as u32).write(w);
                        crate::byte_array::i8_to_u8_slice(y).write(w);
                    });
                }
                Self::IntArray(x) => {
                    x.iter().for_each(|y| {
                        (y.len() as u32).write(w);
                        y.iter().for_each(|z| z.write(w));
                    });
                }
                Self::LongArray(x) => {
                    x.iter().for_each(|y| {
                        (y.len() as u32).write(w);
                        y.iter().for_each(|z| z.write(w));
                    });
                }
                Self::List(x) => {
                    x.iter().for_each(|y| y.write(w));
                }
                Self::Compound(x) => {
                    x.iter().for_each(|y| y.write(w));
                }
            }
        }
    }

    fn len_s(&self) -> usize {
        5 + match self {
            Self::None => 0,
            Self::Byte(x) => x.len(),
            Self::Short(x) => x.len() * 2,
            Self::Int(x) => x.len() * 4,
            Self::Long(x) => x.len() * 8,
            Self::Float(x) => x.len() * 4,
            Self::Double(x) => x.len() * 8,
            Self::String(x) => x.iter().map(|x| RefStringTag(x).len_s()).sum::<usize>(),
            Self::ByteArray(x) => x.len() * 4 + x.iter().map(|x| x.len()).sum::<usize>(),
            Self::IntArray(x) => x.len() * 4 + x.iter().map(|x| x.len()).sum::<usize>() * 4,
            Self::LongArray(x) => x.len() * 4 + x.iter().map(|x| x.len()).sum::<usize>() * 8,
            Self::List(x) => x.iter().map(|x| x.len_s()).sum::<usize>(),
            Self::Compound(x) => x.iter().map(|x| x.len_s()).sum::<usize>(),
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum ListRec {
    Compound,
    List,
}

impl ListInfo {
    pub(crate) fn list_no_rec(self, n: &mut &[u8]) -> Result<Result<ListTag, ListRec>, Error> {
        let len = self.1 as usize;
        match self.0 {
            TagType::End => Ok(Ok(ListTag::None)),
            TagType::Byte => match n.split_at_checked(len) {
                Some((x, y)) => {
                    *n = y;
                    Ok(Ok(ListTag::Byte(Vec::from(
                        crate::byte_array::u8_to_i8_slice(x),
                    ))))
                }
                None => Err(Error),
            },
            TagType::Short => match n.split_at_checked(len.checked_mul(2).ok_or(Error)?) {
                Some((slice, y)) => unsafe {
                    *n = y;
                    Ok(Ok(ListTag::Short(short_list(len, slice))))
                },
                None => Err(Error),
            },
            TagType::Int => match n.split_at_checked(len.checked_mul(4).ok_or(Error)?) {
                Some((slice, y)) => unsafe {
                    *n = y;
                    Ok(Ok(ListTag::Int(int_list(len, slice))))
                },
                None => Err(Error),
            },
            TagType::Long => match n.split_at_checked(len.checked_mul(8).ok_or(Error)?) {
                Some((slice, y)) => unsafe {
                    *n = y;
                    Ok(Ok(ListTag::Long(long_list(len, slice))))
                },
                None => Err(Error),
            },
            TagType::Float => match n.split_at_checked(len.checked_mul(4).ok_or(Error)?) {
                Some((slice, y)) => unsafe {
                    *n = y;
                    Ok(Ok(ListTag::Float(f32_list(len, slice))))
                },
                None => Err(Error),
            },
            TagType::Double => match n.split_at_checked(len.checked_mul(8).ok_or(Error)?) {
                Some((slice, y)) => unsafe {
                    *n = y;
                    Ok(Ok(ListTag::Double(f64_list(len, slice))))
                },
                None => Err(Error),
            },
            TagType::ByteArray => {
                if len.checked_mul(4).ok_or(Error)? > n.len() {
                    return Err(Error);
                }
                let mut list = Vec::with_capacity(len);
                for _ in 0..len {
                    list.push(ByteArray::read(n)?.0);
                }
                Ok(Ok(ListTag::ByteArray(list)))
            }
            TagType::String => {
                if len.checked_mul(2).ok_or(Error)? > n.len() {
                    return Err(Error);
                }
                let mut list = Vec::with_capacity(len);
                for _ in 0..len {
                    list.push(StringTag::read(n)?.0);
                }
                Ok(Ok(ListTag::String(list)))
            }
            TagType::List => Ok(Err(ListRec::List)),
            TagType::Compound => Ok(Err(ListRec::Compound)),
            TagType::IntArray => {
                if len.checked_mul(4).ok_or(Error)? > n.len() {
                    return Err(Error);
                }
                let mut list = Vec::with_capacity(len);
                for _ in 0..len {
                    list.push(IntArray::read(n)?.0);
                }
                Ok(Ok(ListTag::IntArray(list)))
            }
            TagType::LongArray => {
                if len.checked_mul(4).ok_or(Error)? > n.len() {
                    return Err(Error);
                }
                let mut list = Vec::with_capacity(len);
                for _ in 0..len {
                    list.push(LongArray::read(n)?.0);
                }
                Ok(Ok(ListTag::LongArray(list)))
            }
        }
    }
}

pub(crate) unsafe fn long_list(len: usize, slice: &[u8]) -> Vec<i64> {
    debug_assert_eq!(len * 8, slice.len());

    let mut v = Vec::<i64>::with_capacity(len);
    let s = v.as_mut_ptr();
    for index in 0..len {
        unsafe {
            *s.add(index) = i64::from_be_bytes(*slice.as_ptr().add(index * 8).cast::<[u8; 8]>());
        }
    }
    unsafe { v.set_len(len) }
    v
}

pub(crate) unsafe fn int_list(len: usize, slice: &[u8]) -> Vec<i32> {
    debug_assert_eq!(len * 4, slice.len());

    let mut v = Vec::<i32>::with_capacity(len);
    let s = v.as_mut_ptr();
    for index in 0..len {
        unsafe {
            *s.add(index) = i32::from_be_bytes(*slice.as_ptr().add(index * 4).cast::<[u8; 4]>());
        }
    }
    unsafe { v.set_len(len) }
    v
}

pub(crate) unsafe fn short_list(len: usize, slice: &[u8]) -> Vec<i16> {
    debug_assert_eq!(len * 2, slice.len());

    let mut v = Vec::<i16>::with_capacity(len);
    let s = v.as_mut_ptr();
    for index in 0..len {
        unsafe {
            *s.add(index) = i16::from_be_bytes(*slice.as_ptr().add(index * 2).cast::<[u8; 2]>());
        }
    }
    unsafe { v.set_len(len) }
    v
}

pub(crate) unsafe fn f32_list(len: usize, slice: &[u8]) -> Vec<f32> {
    debug_assert_eq!(len * 4, slice.len());

    let mut v = Vec::<f32>::with_capacity(len);
    let s = v.as_mut_ptr();
    for index in 0..len {
        unsafe {
            *s.add(index) = f32::from_be_bytes(*slice.as_ptr().add(index * 4).cast::<[u8; 4]>());
        }
    }
    unsafe { v.set_len(len) }
    v
}

pub(crate) unsafe fn f64_list(len: usize, slice: &[u8]) -> Vec<f64> {
    debug_assert_eq!(len * 8, slice.len());

    let mut v = Vec::<f64>::with_capacity(len);
    let s = v.as_mut_ptr();
    for index in 0..len {
        unsafe {
            *s.add(index) = f64::from_be_bytes(*slice.as_ptr().add(index * 8).cast::<[u8; 8]>());
        }
    }
    unsafe { v.set_len(len) }
    v
}
