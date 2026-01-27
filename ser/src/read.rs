use crate::{Error, Read, unlikely};

impl<'a> Read<'a> for u8 {
    #[inline]
    fn read(buf: &mut &[u8]) -> Result<Self, Error> {
        if let [a, b @ ..] = buf {
            *buf = b;
            Ok(*a)
        } else {
            Err(Error)
        }
    }
}

impl<'a> Read<'a> for i8 {
    #[inline]
    fn read(buf: &mut &[u8]) -> Result<Self, Error> {
        if let [a, b @ ..] = &mut *buf {
            *buf = b;
            Ok(*a as i8)
        } else {
            Err(Error)
        }
    }
}

impl<'a> Read<'a> for u16 {
    #[inline]
    fn read(buf: &mut &[u8]) -> Result<Self, Error> {
        if let [a, b, c @ ..] = buf {
            *buf = c;
            Ok(u16::from_be_bytes([*a, *b]))
        } else {
            Err(Error)
        }
    }
}

impl<'a> Read<'a> for i16 {
    #[inline]
    fn read(buf: &mut &[u8]) -> Result<Self, Error> {
        if let [a, b, c @ ..] = buf {
            *buf = c;
            Ok(i16::from_be_bytes([*a, *b]))
        } else {
            Err(Error)
        }
    }
}

impl<'a> Read<'a> for u32 {
    #[inline]
    fn read(buf: &mut &[u8]) -> Result<Self, Error> {
        if let [a, b, c, d, e @ ..] = buf {
            *buf = e;
            Ok(u32::from_be_bytes([*a, *b, *c, *d]))
        } else {
            Err(Error)
        }
    }
}

impl<'a> Read<'a> for i32 {
    #[inline]
    fn read(buf: &mut &[u8]) -> Result<Self, Error> {
        if let [a, b, c, d, e @ ..] = buf {
            *buf = e;
            Ok(i32::from_be_bytes([*a, *b, *c, *d]))
        } else {
            Err(Error)
        }
    }
}

impl<'a> Read<'a> for u64 {
    #[inline]
    fn read(buf: &mut &[u8]) -> Result<Self, Error> {
        if let [a, b, c, d, e, f, g, h, i @ ..] = buf {
            *buf = i;
            Ok(u64::from_be_bytes([*a, *b, *c, *d, *e, *f, *g, *h]))
        } else {
            Err(Error)
        }
    }
}

impl<'a> Read<'a> for i64 {
    #[inline]
    fn read(buf: &mut &[u8]) -> Result<Self, Error> {
        if let [a, b, c, d, e, f, g, h, i @ ..] = buf {
            *buf = i;
            Ok(i64::from_be_bytes([*a, *b, *c, *d, *e, *f, *g, *h]))
        } else {
            Err(Error)
        }
    }
}

impl<'a> Read<'a> for f32 {
    #[inline]
    fn read(buf: &mut &[u8]) -> Result<Self, Error> {
        if let [a, b, c, d, e @ ..] = buf {
            *buf = e;
            Ok(f32::from_be_bytes([*a, *b, *c, *d]))
        } else {
            Err(Error)
        }
    }
}

impl<'a> Read<'a> for f64 {
    #[inline]
    fn read(buf: &mut &[u8]) -> Result<Self, Error> {
        if let [a, b, c, d, e, f, g, h, i @ ..] = buf {
            *buf = i;
            Ok(f64::from_be_bytes([*a, *b, *c, *d, *e, *f, *g, *h]))
        } else {
            Err(Error)
        }
    }
}

impl<'a> Read<'a> for bool {
    #[inline]
    fn read(buf: &mut &[u8]) -> Result<Self, Error> {
        if let [a, b @ ..] = buf {
            *buf = b;
            Ok(*a == 1)
        } else {
            Err(Error)
        }
    }
}

impl<'a> Read<'a> for uuid::Uuid {
    #[inline]
    fn read(buf: &mut &[u8]) -> Result<Self, Error> {
        if unlikely(16 > buf.len()) {
            return Err(Error);
        }
        let len = buf.len();
        let ptr = buf.as_ptr();
        unsafe {
            let (a, b) = (
                core::slice::from_raw_parts(ptr, 16),
                core::slice::from_raw_parts(ptr.add(16), len - 16),
            );
            *buf = b;
            Ok(Self::from_bytes(*a.as_ptr().cast()))
        }
    }
}

impl<'a, T: Read<'a>> Read<'a> for Option<T> {
    #[inline]
    fn read(buf: &mut &'a [u8]) -> Result<Self, Error> {
        if bool::read(buf)? {
            Ok(Some(T::read(buf)?))
        } else {
            Ok(None)
        }
    }
}
