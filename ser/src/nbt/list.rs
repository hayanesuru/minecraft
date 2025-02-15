use super::{
    decode_string, Compound, UTF8Tag, BYTE, BYTE_ARRAY, COMPOUND, DOUBLE, END, FLOAT, INT,
    INT_ARRAY, LIST, LONG, LONG_ARRAY, SHORT, STRING,
};
use crate::{Bytes, UnsafeWriter, Write};
use alloc::vec::Vec;

#[derive(Clone)]
pub enum List {
    None,
    Byte(Vec<u8>),
    Short(Vec<i16>),
    Int(Vec<i32>),
    Long(Vec<i64>),
    Float(Vec<f32>),
    Double(Vec<f64>),
    String(Vec<flexstr::SharedStr>),
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

impl From<Vec<flexstr::SharedStr>> for List {
    #[inline]
    fn from(value: Vec<flexstr::SharedStr>) -> Self {
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

unsafe impl Write for List {
    unsafe fn write(&self, w: &mut UnsafeWriter) {
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
                x.iter().write(w);
            }
            Self::Int(x) => {
                w.write_byte(INT);
                (x.len() as u32).write(w);
                x.iter().write(w);
            }
            Self::Long(x) => {
                w.write_byte(LONG);
                (x.len() as u32).write(w);
                x.iter().write(w);
            }
            Self::Float(x) => {
                w.write_byte(FLOAT);
                (x.len() as u32).write(w);
                x.iter().write(w);
            }
            Self::Double(x) => {
                w.write_byte(DOUBLE);
                (x.len() as u32).write(w);
                x.iter().write(w);
            }
            Self::String(x) => unsafe {
                (STRING).write(w);
                (x.len() as u32).write(w);
                x.iter()
                    .for_each(|x| UTF8Tag::new_unchecked(x.as_bytes()).write(w));
            },
            Self::ByteArray(x) => {
                w.write_byte(BYTE_ARRAY);
                (x.len() as u32).write(w);
                x.iter().for_each(|y| {
                    (y.len() as u32).write(w);
                    y.iter().write(w);
                });
            }
            Self::IntArray(x) => {
                w.write_byte(INT_ARRAY);
                (x.len() as u32).write(w);
                x.iter().for_each(|y| {
                    (y.len() as u32).write(w);
                    y.iter().write(w);
                });
            }
            Self::LongArray(x) => {
                w.write_byte(LONG_ARRAY);
                (x.len() as u32).write(w);
                x.iter().for_each(|y| {
                    (y.len() as u32).write(w);
                    y.iter().write(w);
                });
            }
            Self::List(x) => {
                w.write_byte(LIST);
                (x.len() as u32).write(w);
                x.iter().write(w);
            }
            Self::Compound(x) => {
                (COMPOUND).write(w);
                (x.len() as u32).write(w);
                x.iter().write(w);
            }
        }
    }

    unsafe fn sz(&self) -> usize {
        5 + match self {
            Self::None => 0,
            Self::Byte(x) => x.len(),
            Self::Short(x) => x.len() * 2,
            Self::Int(x) => x.len() * 4,
            Self::Long(x) => x.len() * 8,
            Self::Float(x) => x.len() * 4,
            Self::Double(x) => x.len() * 8,
            Self::String(x) => unsafe {
                x.iter()
                    .map(|x| UTF8Tag::new_unchecked(x.as_bytes()).sz())
                    .sum::<usize>()
            },
            Self::ByteArray(x) => x.len() * 4 + x.iter().map(|x| x.len()).sum::<usize>(),
            Self::IntArray(x) => x.len() * 4 + x.iter().map(|x| x.len()).sum::<usize>() * 4,
            Self::LongArray(x) => x.len() * 4 + x.iter().map(|x| x.len()).sum::<usize>() * 8,
            Self::List(x) => x.iter().sz(),
            Self::Compound(x) => x.iter().sz(),
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
                let v = decode_string(n)?;
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
                list.push(super::decode1(n)?);
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
