use crate::string::DecodeMutf8;
use crate::{RefStringTag, Tag, TagType};
use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use haya_mutf8::{Mutf8, as_mutf8_ascii, decode_mutf8_len};
use haya_str::HayaStr;
use mser::{Error, Read, UnsafeWriter, Write};

#[derive(Clone)]
#[repr(transparent)]
pub struct Compound(Vec<(Name, Tag)>);

#[derive(Clone)]
pub enum Name {
    Thin(HayaStr),
    Heap(Box<str>),
}

enum CowVec {
    Thin(HayaStr),
    Heap(Vec<u8>),
}

impl Read<'_> for Name {
    fn read(buf: &mut &'_ [u8]) -> Result<Self, Error> {
        let len = u16::read(buf)? as usize;
        let data = match buf.split_at_checked(len) {
            Some((x, y)) => {
                *buf = y;
                x
            }
            None => return Err(Error),
        };
        if let Some(x) = as_mutf8_ascii(data) {
            Ok(Self::new(x))
        } else {
            let len = decode_mutf8_len(data)?;
            let mut ptr = if len <= haya_str::MAX {
                CowVec::Thin(HayaStr::new())
            } else {
                CowVec::Heap(Vec::with_capacity(len))
            };
            unsafe {
                mser::write_unchecked(
                    match &mut ptr {
                        CowVec::Thin(s) => s.as_mut_ptr(),
                        CowVec::Heap(x) => x.as_mut_ptr(),
                    },
                    &(DecodeMutf8(Mutf8::new_unchecked(data), len)),
                );
                match ptr {
                    CowVec::Thin(mut s) => {
                        s.set_len(len);
                        Ok(Self::Thin(s))
                    }
                    CowVec::Heap(mut x) => {
                        x.set_len(len);
                        Ok(Self::Heap(String::from_utf8_unchecked(x).into_boxed_str()))
                    }
                }
            }
        }
    }
}

impl AsRef<str> for Name {
    fn as_ref(&self) -> &str {
        match self {
            Self::Thin(x) => x,
            Self::Heap(x) => x,
        }
    }
}

impl core::ops::Deref for Name {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl Name {
    pub fn new(s: &str) -> Self {
        match HayaStr::copy_from(s) {
            Ok(x) => Self::Thin(x),
            Err(_) => Self::Heap(s.to_owned().into_boxed_str()),
        }
    }
}

impl AsRef<[(Name, Tag)]> for Compound {
    #[inline]
    fn as_ref(&self) -> &[(Name, Tag)] {
        self.0.as_slice()
    }
}

impl AsMut<[(Name, Tag)]> for Compound {
    #[inline]
    fn as_mut(&mut self) -> &mut [(Name, Tag)] {
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

impl<K: Into<Name>, V> FromIterator<(K, V)> for Compound
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
    pub fn iter(&self) -> core::slice::Iter<'_, (Name, Tag)> {
        self.0.iter()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> core::slice::IterMut<'_, (Name, Tag)> {
        self.0.iter_mut()
    }

    #[inline]
    pub fn sort(&mut self) {
        self.0.sort_unstable_by(|x, y| x.0.cmp(&y.0));
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
        (&**x, y)
    }

    #[inline]
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    #[inline]
    pub fn push(&mut self, k: Name, v: Tag) {
        self.0.push((k, v));
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
    pub fn into_inner(self) -> Vec<(Name, Tag)> {
        self.0
    }
}

impl Read<'_> for Compound {
    #[inline]
    fn read(buf: &mut &[u8]) -> Result<Self, Error> {
        match TagType::Compound.tag(buf) {
            Ok(Tag::Compound(x)) => Ok(x),
            Ok(_) => Err(Error),
            Err(e) => Err(e),
        }
    }
}

impl From<Vec<(Name, Tag)>> for Compound {
    #[inline]
    fn from(value: Vec<(Name, Tag)>) -> Self {
        Self(value)
    }
}
