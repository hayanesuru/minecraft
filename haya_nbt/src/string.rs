use crate::{Error, Name, Read, Write, Writer};
use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::vec::Vec;
use haya_mutf8::{as_mutf8_ascii, decode_mutf8, decode_mutf8_len, encode_mutf8, encode_mutf8_len};
use mser::Reader;

#[derive(Clone, Copy)]
#[repr(transparent)]
#[must_use]
pub struct RawStringTag<'a>(&'a str);

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

#[derive(Clone, Copy)]
#[repr(transparent)]
#[must_use]
pub struct RefStringTag<'a>(pub &'a str);

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

#[derive(Clone)]
#[repr(transparent)]
pub struct StringTag(pub Box<str>);

impl<'a> Read<'a> for StringTag {
    #[inline]
    fn read(buf: &mut Reader<'a>) -> Result<Self, Error> {
        let len = u16::read(buf)? as usize;
        let data = buf.read_slice(len)?;
        if let Some(x) = as_mutf8_ascii(data) {
            Ok(Self(x.to_owned().into_boxed_str()))
        } else {
            let len = decode_mutf8_len(data)?;
            let mut x = Vec::with_capacity(len);
            unsafe {
                mser::write_unchecked(x.as_mut_ptr(), &(DecodeMutf8(data, len)));
                x.set_len(len);
                Ok(Self(
                    alloc::string::String::from_utf8_unchecked(x).into_boxed_str(),
                ))
            }
        }
    }
}

pub(crate) struct DecodeMutf8<'a>(pub &'a [u8], pub usize);

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
        unsafe { RefStringTag(&self.0).write(w) }
    }

    #[inline]
    fn len_s(&self) -> usize {
        RefStringTag(&self.0).len_s()
    }
}

impl Write for Name {
    #[inline]
    unsafe fn write(&self, w: &mut Writer) {
        unsafe {
            if let Some(x) = RawStringTag::new(self.as_bytes()) {
                x.write(w);
            } else {
                (encode_mutf8_len(self) as u16).write(w);
                encode_mutf8(self, w);
            }
        }
    }

    #[inline]
    fn len_s(&self) -> usize {
        if let Some(x) = RawStringTag::new(self.as_bytes()) {
            x.len_s()
        } else {
            encode_mutf8_len(self) + 2
        }
    }
}
