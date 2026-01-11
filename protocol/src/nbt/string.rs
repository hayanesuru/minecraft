use crate::{Bytes, Error, Ident, Read, UnsafeWriter, Write};
use mser::{encode_mutf8, encode_mutf8_len, is_ascii_mutf8, is_mutf8};

#[derive(Clone, Copy)]
#[repr(transparent)]
#[must_use]
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
    fn len_s(&self) -> usize {
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
#[must_use]
pub struct RefStringTag<'a>(pub &'a str);

impl<'a> Write for RefStringTag<'a> {
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
    fn len_s(&self) -> usize {
        if is_ascii_mutf8(self.0.as_bytes()) {
            StringTagRaw(self.0.as_bytes()).len_s()
        } else {
            encode_mutf8_len(self.0) + 2
        }
    }
}

impl<'a> Read<'a> for RefStringTag<'a> {
    fn read(buf: &mut &'a [u8]) -> Result<Self, Error> {
        unsafe {
            Ok(Self(core::str::from_utf8_unchecked(
                StringTagRaw::read(buf)?.0,
            )))
        }
    }
}
#[derive(Clone)]
pub struct IdentifierTag<'a>(pub Ident<'a>);

impl Write for IdentifierTag<'_> {
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            let l = self.0.namespace.len() + 1 + self.0.path.len();
            (l as u16).write(w);
            w.write(self.0.namespace.as_bytes());
            w.write_byte(b':');
            w.write(self.0.path.as_bytes());
        }
    }

    fn len_s(&self) -> usize {
        2 + self.0.namespace.len() + 1 + self.0.path.len()
    }
}

impl<'a> Read<'a> for IdentifierTag<'a> {
    fn read(buf: &mut &'a [u8]) -> Result<Self, Error> {
        let s = unsafe { core::str::from_utf8_unchecked(StringTagRaw::read(buf)?.inner()) };
        match Ident::parse(s) {
            Some(ident) => Ok(Self(ident)),
            None => Err(Error),
        }
    }
}
