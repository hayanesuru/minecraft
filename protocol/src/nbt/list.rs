use super::*;
use crate::SmolStr;

#[derive(Clone)]
pub enum List<A: Allocator = Global> {
    None,
    Byte(Vec<i8, A>),
    Short(Vec<i16, A>),
    Int(Vec<i32, A>),
    Long(Vec<i64, A>),
    Float(Vec<f32, A>),
    Double(Vec<f64, A>),
    String(Vec<SmolStr<A>, A>),
    ByteArray(Vec<Vec<i8>, A>),
    IntArray(Vec<Vec<i32>, A>),
    LongArray(Vec<Vec<i64>, A>),
    List(Vec<List<A>, A>),
    Compound(Vec<Compound<A>, A>),
}

impl From<Vec<u8>> for List {
    #[inline]
    fn from(value: Vec<u8>) -> Self {
        let mut me = core::mem::ManuallyDrop::new(value);
        Self::Byte(unsafe {
            Vec::from_raw_parts(me.as_mut_ptr().cast::<i8>(), me.len(), me.capacity())
        })
    }
}

impl From<Vec<i8>> for List {
    #[inline]
    fn from(value: Vec<i8>) -> Self {
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

impl From<Vec<SmolStr>> for List {
    #[inline]
    fn from(value: Vec<SmolStr>) -> Self {
        Self::String(value)
    }
}

impl From<Vec<Vec<i8>>> for List {
    #[inline]
    fn from(value: Vec<Vec<i8>>) -> Self {
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
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            match self {
                Self::None => {
                    TagType::End.write(w);
                    w.write(&[0, 0, 0, 0]);
                }

                Self::Byte(x) => {
                    TagType::Byte.write(w);
                    (x.len() as u32).write(w);
                    w.write(&*(x.as_slice() as *const [i8] as *const [u8]));
                }
                Self::Short(x) => {
                    TagType::Short.write(w);
                    (x.len() as u32).write(w);
                    x.iter().write(w);
                }
                Self::Int(x) => {
                    TagType::Int.write(w);
                    (x.len() as u32).write(w);
                    x.iter().write(w);
                }
                Self::Long(x) => {
                    TagType::Long.write(w);
                    (x.len() as u32).write(w);
                    x.iter().write(w);
                }
                Self::Float(x) => {
                    TagType::Float.write(w);
                    (x.len() as u32).write(w);
                    x.iter().write(w);
                }
                Self::Double(x) => {
                    TagType::Double.write(w);
                    (x.len() as u32).write(w);
                    x.iter().write(w);
                }
                Self::String(x) => {
                    TagType::String.write(w);
                    (x.len() as u32).write(w);
                    x.iter().for_each(|x| StringTagWriter(x).write(w));
                }
                Self::ByteArray(x) => {
                    TagType::ByteArray.write(w);
                    (x.len() as u32).write(w);
                    x.iter().for_each(|y| {
                        (y.len() as u32).write(w);
                        y.iter().write(w);
                    });
                }
                Self::IntArray(x) => {
                    TagType::IntArray.write(w);
                    (x.len() as u32).write(w);
                    x.iter().for_each(|y| {
                        (y.len() as u32).write(w);
                        y.iter().write(w);
                    });
                }
                Self::LongArray(x) => {
                    TagType::LongArray.write(w);
                    (x.len() as u32).write(w);
                    x.iter().for_each(|y| {
                        (y.len() as u32).write(w);
                        y.iter().write(w);
                    });
                }
                Self::List(x) => {
                    TagType::List.write(w);
                    (x.len() as u32).write(w);
                    x.iter().write(w);
                }
                Self::Compound(x) => {
                    TagType::Compound.write(w);
                    (x.len() as u32).write(w);
                    x.iter().write(w);
                }
            }
        }
    }

    fn sz(&self) -> usize {
        5 + match self {
            Self::None => 0,
            Self::Byte(x) => x.len(),
            Self::Short(x) => x.len() * 2,
            Self::Int(x) => x.len() * 4,
            Self::Long(x) => x.len() * 8,
            Self::Float(x) => x.len() * 4,
            Self::Double(x) => x.len() * 8,
            Self::String(x) => x.iter().map(|x| StringTagWriter(x).sz()).sum::<usize>(),
            Self::ByteArray(x) => x.len() * 4 + x.iter().map(|x| x.len()).sum::<usize>(),
            Self::IntArray(x) => x.len() * 4 + x.iter().map(|x| x.len()).sum::<usize>() * 4,
            Self::LongArray(x) => x.len() * 4 + x.iter().map(|x| x.len()).sum::<usize>() * 8,
            Self::List(x) => x.iter().sz(),
            Self::Compound(x) => x.iter().sz(),
        }
    }
}

pub fn decode_raw(n: &mut &[u8], id: TagType, len: usize) -> Result<List, Error> {
    match id {
        TagType::End => Ok(List::None),
        TagType::Byte => unsafe {
            Ok(List::Byte(Vec::from(
                &*(n.slice(len)? as *const [u8] as *const [i8]),
            )))
        },
        TagType::Short => {
            let mut slice = n.slice(len << 1)?;
            let mut v = Vec::with_capacity(len);
            for _ in 0..len {
                v.push(slice.i16()?);
            }
            Ok(List::Short(v))
        }
        TagType::Int => {
            let mut slice = n.slice(len << 2)?;
            let mut v = Vec::with_capacity(len);
            for _ in 0..len {
                v.push(slice.i32()?);
            }
            Ok(List::Int(v))
        }
        TagType::Long => {
            let mut slice = n.slice(len << 3)?;
            let mut v = Vec::with_capacity(len);
            for _ in 0..len {
                v.push(slice.i64()?);
            }
            Ok(List::Long(v))
        }
        TagType::Float => {
            let mut slice = n.slice(len << 2)?;
            let mut v = Vec::with_capacity(len);
            for _ in 0..len {
                v.push(slice.f32()?);
            }
            Ok(List::Float(v))
        }
        TagType::Double => {
            let mut slice = n.slice(len << 3)?;
            let mut v = Vec::with_capacity(len);
            for _ in 0..len {
                v.push(slice.f64()?);
            }
            Ok(List::Double(v))
        }
        TagType::ByteArray => unsafe {
            if len * 4 > n.len() {
                return Err(Error);
            }
            let mut list = Vec::with_capacity(len);
            for _ in 0..len {
                let len = n.i32()? as usize;
                let slice = &*(n.slice(len)? as *const [u8] as *const [i8]);
                list.push(Vec::from(slice));
            }
            Ok(List::ByteArray(list))
        },
        TagType::String => {
            if len * 2 > n.len() {
                return Err(Error);
            }
            let mut list = Vec::with_capacity(len);
            for _ in 0..len {
                list.push(StringTag::read(n)?.0);
            }
            Ok(List::String(list))
        }
        TagType::List => {
            if len << 2 > n.len() {
                return Err(Error);
            }
            let mut list = Vec::with_capacity(len);
            for _ in 0..len {
                let id = TagType::read(n)?;
                let len = n.i32()? as usize;
                list.push(decode_raw(n, id, len)?);
            }
            Ok(List::List(list))
        }
        TagType::Compound => {
            if len > n.len() {
                return Err(Error);
            }
            let mut list = Vec::with_capacity(len);
            for _ in 0..len {
                list.push(super::decode_raw(n)?);
            }
            Ok(List::Compound(list))
        }
        TagType::IntArray => {
            if len * 4 > n.len() {
                return Err(Error);
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
            Ok(List::IntArray(list))
        }
        TagType::LongArray => {
            if len * 4 > n.len() {
                return Err(Error);
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
            Ok(List::LongArray(list))
        }
    }
}
