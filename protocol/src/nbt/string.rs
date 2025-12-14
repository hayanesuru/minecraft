use crate::{Bytes, Error, Read, UnsafeWriter, Write};
use mser::{encode_mutf8, encode_mutf8_len, is_ascii_mutf8, is_mutf8};

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct StringTagRaw<'a>(&'a [u8]);

impl<'a> StringTagRaw<'a> {
    pub const fn new(n: &'a [u8]) -> Option<Self> {
        if is_mutf8(n) { Some(Self(n)) } else { None }
    }

    pub const fn new_unchecked(n: &'a [u8]) -> Self {
        debug_assert!(is_mutf8(n));
        Self(n)
    }

    pub const fn inner(&self) -> &'a [u8] {
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
    fn sz(&self) -> usize {
        2 + self.0.len()
    }
}

impl<'a> Read<'a> for StringTagRaw<'a> {
    #[inline]
    fn read(buf: &mut &'a [u8]) -> Result<Self, Error> {
        let len = buf.u16()?;
        let data = buf.slice(len as usize)?;
        if is_ascii_mutf8(data) {
            Ok(Self(data))
        } else {
            Err(Error)
        }
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct StringTagWriter<'a>(pub &'a str);

impl<'a> Write for StringTagWriter<'a> {
    #[inline]
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            if is_ascii_mutf8(self.0.as_bytes()) {
                StringTagRaw(self.0.as_bytes()).write(w);
            } else {
                (encode_mutf8_len(self.0) as u16).write(w);
                encode_mutf8(self.0.as_bytes(), w);
            }
        }
    }

    #[inline]
    fn sz(&self) -> usize {
        if is_ascii_mutf8(self.0.as_bytes()) {
            StringTagRaw(self.0.as_bytes()).sz()
        } else {
            encode_mutf8_len(self.0) + 2
        }
    }
}
