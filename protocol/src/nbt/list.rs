use super::*;

#[derive(Clone)]
pub enum List {
    None,
    Byte(Vec<i8>),
    Short(Vec<i16>),
    Int(Vec<i32>),
    Long(Vec<i64>),
    Float(Vec<f32>),
    Double(Vec<f64>),
    String(Vec<BoxStr>),
    ByteArray(Vec<Vec<i8>>),
    IntArray(Vec<Vec<i32>>),
    LongArray(Vec<Vec<i64>>),
    List(Vec<List>),
    Compound(Vec<Compound>),
}

#[derive(Clone)]
pub enum ListPrimitive {
    Byte(Vec<i8>),
    Short(Vec<i16>),
    Int(Vec<i32>),
    Long(Vec<i64>),
    Float(Vec<f32>),
    Double(Vec<f64>),
}

impl From<ListPrimitive> for List {
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
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct ListInfo(pub TagType, pub u32);

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

impl From<Vec<BoxStr>> for List {
    #[inline]
    fn from(value: Vec<BoxStr>) -> Self {
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

impl List {
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

impl Write for List {
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            self.list_info().write(w);
            match self {
                Self::None => {}
                Self::Byte(x) => {
                    w.write(byte_array::i8_to_u8_slice(x));
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
                        y.iter().write(w);
                    });
                }
                Self::IntArray(x) => {
                    x.iter().for_each(|y| {
                        (y.len() as u32).write(w);
                        y.iter().write(w);
                    });
                }
                Self::LongArray(x) => {
                    x.iter().for_each(|y| {
                        (y.len() as u32).write(w);
                        y.iter().write(w);
                    });
                }
                Self::List(x) => {
                    x.iter().write(w);
                }
                Self::Compound(x) => {
                    x.iter().write(w);
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
            Self::List(x) => x.iter().len_s(),
            Self::Compound(x) => x.iter().len_s(),
        }
    }
}

impl ListInfo {
    pub fn list(self, n: &mut &[u8]) -> Result<List, Error> {
        let len = self.1 as usize;
        match self.0 {
            TagType::End => Ok(List::None),
            TagType::Byte => match n.split_at_checked(len) {
                Some((x, y)) => {
                    *n = y;
                    Ok(List::Byte(Vec::from(byte_array::u8_to_i8_slice(x))))
                }
                None => Err(Error),
            },
            TagType::Short => match n.split_at_checked(len * 2) {
                Some((slice, y)) => {
                    *n = y;
                    let mut v = Vec::with_capacity(len);
                    let s = unsafe { v.spare_capacity_mut().assume_init_mut() };
                    for index in 0..len {
                        unsafe {
                            *s.get_unchecked_mut(index) = i16::from_be_bytes(
                                *slice.as_ptr().add(index * 2).cast::<[u8; 2]>(),
                            );
                        }
                    }
                    unsafe { v.set_len(len) }
                    Ok(List::Short(v))
                }
                None => Err(Error),
            },
            TagType::Int => match n.split_at_checked(len * 4) {
                Some((slice, y)) => unsafe {
                    *n = y;
                    Ok(List::Int(int_list(len, slice)))
                },
                None => Err(Error),
            },
            TagType::Long => match n.split_at_checked(len * 8) {
                Some((slice, y)) => {
                    *n = y;
                    let mut v = Vec::with_capacity(len);
                    let s = unsafe { v.spare_capacity_mut().assume_init_mut() };
                    for index in 0..len {
                        unsafe {
                            *s.get_unchecked_mut(index) = i64::from_be_bytes(
                                *slice.as_ptr().add(index * 8).cast::<[u8; 8]>(),
                            );
                        }
                    }
                    unsafe { v.set_len(len) }
                    Ok(List::Long(v))
                }
                None => Err(Error),
            },
            TagType::Float => match n.split_at_checked(len * 4) {
                Some((slice, y)) => {
                    *n = y;
                    let mut v = Vec::with_capacity(len);
                    let s = unsafe { v.spare_capacity_mut().assume_init_mut() };
                    for index in 0..len {
                        unsafe {
                            *s.get_unchecked_mut(index) = f32::from_be_bytes(
                                *slice.as_ptr().add(index * 4).cast::<[u8; 4]>(),
                            );
                        }
                    }
                    unsafe { v.set_len(len) }
                    Ok(List::Float(v))
                }
                None => Err(Error),
            },
            TagType::Double => match n.split_at_checked(len * 8) {
                Some((slice, y)) => {
                    *n = y;
                    let mut v = Vec::with_capacity(len);
                    let s = unsafe { v.spare_capacity_mut().assume_init_mut() };
                    for index in 0..len {
                        unsafe {
                            *s.get_unchecked_mut(index) = f64::from_be_bytes(
                                *slice.as_ptr().add(index * 8).cast::<[u8; 8]>(),
                            );
                        }
                    }
                    unsafe { v.set_len(len) }
                    Ok(List::Double(v))
                }
                None => Err(Error),
            },
            TagType::ByteArray => {
                if len * 4 > n.len() {
                    return Err(Error);
                }
                let mut list = Vec::with_capacity(len);
                for _ in 0..len {
                    list.push(byte_array::ByteArray::read(n)?.0);
                }
                Ok(List::ByteArray(list))
            }
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
                    let info = ListInfo::read(n)?;
                    list.push(info.list(n)?);
                }
                Ok(List::List(list))
            }
            TagType::Compound => {
                if len > n.len() {
                    return Err(Error);
                }
                let mut list = Vec::with_capacity(len);
                for _ in 0..len {
                    list.push(Compound::read(n)?);
                }
                Ok(List::Compound(list))
            }
            TagType::IntArray => {
                if len * 4 > n.len() {
                    return Err(Error);
                }
                let mut list = Vec::with_capacity(len);
                for _ in 0..len {
                    list.push(int_array::IntArray::read(n)?.0);
                }
                Ok(List::IntArray(list))
            }
            TagType::LongArray => {
                if len * 4 > n.len() {
                    return Err(Error);
                }
                let mut list = Vec::with_capacity(len);
                for _ in 0..len {
                    list.push(long_array::LongArray::read(n)?.0);
                }
                Ok(List::LongArray(list))
            }
        }
    }
}

pub unsafe fn int_list(len: usize, slice: &[u8]) -> Vec<i32> {
    debug_assert_eq!(len * 4, slice.len());

    let mut v = Vec::with_capacity(len);
    let s = unsafe { v.spare_capacity_mut().assume_init_mut() };
    for index in 0..len {
        unsafe {
            *s.get_unchecked_mut(index) =
                i32::from_be_bytes(*slice.as_ptr().add(index * 4).cast::<[u8; 4]>());
        }
    }
    unsafe { v.set_len(len) }
    v
}
