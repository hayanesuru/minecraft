use crate::{Error, Inner, RawStringTag, Read, RefStringTag, StringTag, Write, Writer};
use alloc::vec::Vec;
use haya_mutf8::{as_mutf8_ascii, decode_mutf8, decode_mutf8_len, encode_mutf8, encode_mutf8_len};
use haya_str::HayaStr;
use mser::Reader;

impl<'a> RawStringTag<'a> {
    pub const fn new(n: &'a [u8]) -> Option<Self> {
        if let Some(s) = as_mutf8_ascii(n) {
            Some(Self(s))
        } else {
            None
        }
    }

    pub const fn new_unchecked(n: &'a str) -> Self {
        debug_assert!(as_mutf8_ascii(n.as_bytes()).is_some());
        Self(n)
    }

    pub const fn inner(&self) -> &'a str {
        self.0
    }
}

impl<'a> Write for RawStringTag<'a> {
    #[inline]
    unsafe fn write(&self, w: &mut Writer) {
        unsafe {
            (self.0.len() as u16).write(w);
            w.write(self.0.as_ref());
        }
    }

    #[inline]
    fn len_s(&self) -> usize {
        2 + self.0.len()
    }
}

impl<'a> Read<'a> for RawStringTag<'a> {
    #[inline]
    fn read(buf: &mut Reader<'a>) -> Result<Self, Error> {
        let len = u16::read(buf)?;
        let data = buf.read_slice(len as usize)?;
        if let Some(x) = as_mutf8_ascii(data) {
            Ok(Self(x))
        } else {
            Err(Error)
        }
    }
}

impl<'a> Write for RefStringTag<'a> {
    #[inline]
    unsafe fn write(&self, w: &mut Writer) {
        unsafe {
            if let Some(x) = RawStringTag::new(self.0.as_bytes()) {
                x.write(w);
            } else {
                (encode_mutf8_len(self.0) as u16).write(w);
                encode_mutf8(self.0, w);
            }
        }
    }

    #[inline]
    fn len_s(&self) -> usize {
        if let Some(x) = RawStringTag::new(self.0.as_bytes()) {
            x.len_s()
        } else {
            encode_mutf8_len(self.0) + 2
        }
    }
}

impl<'a> Read<'a> for RefStringTag<'a> {
    fn read(buf: &mut Reader<'a>) -> Result<Self, Error> {
        Ok(Self(RawStringTag::read(buf)?.0))
    }
}

struct DecodeMutf8<'a>(&'a [u8], usize);

impl Write for DecodeMutf8<'_> {
    unsafe fn write(&self, w: &mut Writer) {
        unsafe { decode_mutf8(self.0, w).unwrap_unchecked() }
    }

    fn len_s(&self) -> usize {
        self.1
    }
}

impl Write for StringTag {
    #[inline]
    unsafe fn write(&self, w: &mut Writer) {
        unsafe {
            match &self.0 {
                Inner::Thin(direct) => RawStringTag::new_unchecked(direct).write(w),
                Inner::Heap(heap) => match RawStringTag::new(heap.as_bytes()) {
                    Some(x) => x.write(w),
                    None => {
                        (encode_mutf8_len(self) as u16).write(w);
                        encode_mutf8(self, w);
                    }
                },
            }
        }
    }

    #[inline]
    fn len_s(&self) -> usize {
        match &self.0 {
            Inner::Thin(direct) => RawStringTag::new_unchecked(direct).len_s(),
            Inner::Heap(heap) => match RawStringTag::new(heap.as_bytes()) {
                Some(x) => x.len_s(),
                None => encode_mutf8_len(self) + 2,
            },
        }
    }
}

impl<'a> Read<'a> for StringTag {
    fn read(buf: &mut Reader<'a>) -> Result<Self, Error> {
        let len = u16::read(buf)? as usize;
        let data = buf.read_slice(len)?;
        if let Some(s) = as_mutf8_ascii(data) {
            match HayaStr::copy_from(s) {
                Ok(x) => Ok(Self(Inner::Thin(x))),
                Err(_) => Ok(Self(Inner::Heap(
                    alloc::string::String::from(s).into_boxed_str(),
                ))),
            }
        } else {
            let len = decode_mutf8_len(data)?;
            let mut vec = Vec::with_capacity(len);
            unsafe {
                mser::write_unchecked(vec.as_mut_ptr(), &(DecodeMutf8(data, len)));
                vec.set_len(len);
                Ok(Self(crate::Inner::Heap(
                    alloc::string::String::from_utf8_unchecked(vec).into_boxed_str(),
                )))
            }
        }
    }
}

impl AsRef<str> for StringTag {
    fn as_ref(&self) -> &str {
        match &self.0 {
            Inner::Thin(x) => x,
            Inner::Heap(x) => x,
        }
    }
}

impl core::ops::Deref for StringTag {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl Default for StringTag {
    fn default() -> Self {
        Self::new()
    }
}

impl StringTag {
    pub const fn new() -> Self {
        Self(Inner::Thin(HayaStr::new()))
    }

    /// # Safety
    ///
    /// all byte values must be ASCII and non-zero
    pub const unsafe fn from_ascii_nunzero_unchecked(s: &str) -> Option<Self> {
        match HayaStr::copy_from(s) {
            Ok(x) => Some(Self(Inner::Thin(x))),
            Err(_) => None,
        }
    }

    pub const fn from_ascii_nonzero(s: &[u8]) -> Option<Self> {
        match as_mutf8_ascii(s) {
            Some(ascii) => unsafe { Self::from_ascii_nunzero_unchecked(ascii) },
            None => None,
        }
    }

    pub fn from_utf8(s: &str) -> Self {
        match Self::from_ascii_nonzero(s.as_bytes()) {
            Some(x) => x,
            None => Self(Inner::Heap(alloc::string::String::from(s).into_boxed_str())),
        }
    }

    pub fn from_owned(s: alloc::boxed::Box<str>) -> Self {
        match Self::from_ascii_nonzero(s.as_bytes()) {
            Some(x) => x,
            None => Self(Inner::Heap(s)),
        }
    }

    pub fn into_owned(self) -> alloc::boxed::Box<str> {
        match self.0 {
            Inner::Thin(s) => alloc::string::String::from(&*s).into_boxed_str(),
            Inner::Heap(s) => s,
        }
    }
}
