use crate::{Error, Read, UnsafeWriter, Write};
use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::vec::Vec;
use haya_mutf8::{
    Mutf8, as_mutf8_ascii, decode_mutf8, decode_mutf8_len, encode_mutf8, encode_mutf8_len,
};

#[derive(Clone, Copy)]
#[repr(transparent)]
#[must_use]
pub struct StringTagRaw<'a>(&'a str);

impl<'a> StringTagRaw<'a> {
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

impl<'a> Write for StringTagRaw<'a> {
    #[inline]
    unsafe fn write(&self, w: &mut UnsafeWriter) {
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

impl<'a> Read<'a> for StringTagRaw<'a> {
    #[inline]
    fn read(buf: &mut &'a [u8]) -> Result<Self, Error> {
        let len = u16::read(buf)?;
        let data = match buf.split_at_checked(len as usize) {
            Some((x, y)) => {
                *buf = y;
                x
            }
            None => return Err(Error),
        };
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
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            if let Some(x) = StringTagRaw::new(self.0.as_bytes()) {
                x.write(w);
            } else {
                (encode_mutf8_len(self.0) as u16).write(w);
                encode_mutf8(self.0, w);
            }
        }
    }

    #[inline]
    fn len_s(&self) -> usize {
        if let Some(x) = StringTagRaw::new(self.0.as_bytes()) {
            x.len_s()
        } else {
            encode_mutf8_len(self.0) + 2
        }
    }
}

impl<'a> Read<'a> for RefStringTag<'a> {
    fn read(buf: &mut &'a [u8]) -> Result<Self, Error> {
        Ok(Self(StringTagRaw::read(buf)?.0))
    }
}

#[derive(Clone)]
#[repr(transparent)]
pub struct StringTag(pub Box<str>);

impl Read<'_> for StringTag {
    #[inline]
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
            Ok(Self(x.to_owned().into_boxed_str()))
        } else {
            let len = decode_mutf8_len(data)?;
            let mut x = Vec::with_capacity(len);
            unsafe {
                mser::write_unchecked(
                    x.as_mut_ptr(),
                    &(DecodeMutf8(Mutf8::new_unchecked(data), len)),
                );
                x.set_len(len);
                Ok(Self(
                    alloc::string::String::from_utf8_unchecked(x).into_boxed_str(),
                ))
            }
        }
    }
}

pub(crate) struct DecodeMutf8<'a>(pub Mutf8<'a>, pub usize);

impl Write for DecodeMutf8<'_> {
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe { decode_mutf8(self.0, w).unwrap_unchecked() }
    }

    fn len_s(&self) -> usize {
        self.1
    }
}

impl Write for StringTag {
    #[inline]
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe { RefStringTag(&self.0).write(w) }
    }

    #[inline]
    fn len_s(&self) -> usize {
        RefStringTag(&self.0).len_s()
    }
}
