use crate::nbt::mutf8::is_mutf8;
use crate::{Bytes, Error, Read, UnsafeWriter, Write};

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct StringTagRaw<'a>(&'a [u8]);

impl<'a> StringTagRaw<'a> {
    pub const fn new(n: &'a str) -> Self {
        debug_assert!(is_mutf8(n.as_bytes()));
        Self(n.as_bytes())
    }

    pub const fn new_unchecked(n: &'a [u8]) -> Self {
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
        if super::mutf8::is_mutf8(data) {
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
            if super::mutf8::is_mutf8(self.0.as_bytes()) {
                StringTagRaw(self.0.as_bytes()).write(w);
            } else {
                (super::mutf8::len_mutf8(self.0) as u16).write(w);
                super::mutf8::encode_mutf8(self.0.as_bytes(), w);
            }
        }
    }

    #[inline]
    fn sz(&self) -> usize {
        if super::mutf8::is_mutf8(self.0.as_bytes()) {
            StringTagRaw(self.0.as_bytes()).sz()
        } else {
            2 + super::mutf8::len_mutf8(self.0)
        }
    }
}
