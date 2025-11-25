use crate::{Bytes, Error, Read, UnsafeWriter, Write};

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct MUTF8Tag<'a>(pub &'a str);

impl<'a> Write for MUTF8Tag<'a> {
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

impl<'a> Read<'a> for MUTF8Tag<'a> {
    #[inline]
    fn read(buf: &mut &'a [u8]) -> Result<Self, Error> {
        let len = buf.u16()?;
        let data = buf.slice(len as usize)?;
        let data = match core::str::from_utf8(data) {
            Ok(x) => x,
            Err(_) => return Err(Error),
        };
        if super::mutf8::is_mutf8(data.as_bytes()) {
            Ok(Self(data))
        } else {
            Err(Error)
        }
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct UTF8Tag<'a>(pub &'a str);

impl<'a> Write for UTF8Tag<'a> {
    #[inline]
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            if super::mutf8::is_mutf8(self.0.as_bytes()) {
                MUTF8Tag(self.0).write(w);
            } else {
                (super::mutf8::len_mutf8(self.0) as u16).write(w);
                super::mutf8::encode_mutf8(self.0.as_bytes(), w);
            }
        }
    }

    #[inline]
    fn sz(&self) -> usize {
        2 + super::mutf8::len_mutf8(self.0)
    }
}
