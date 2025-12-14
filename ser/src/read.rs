use crate::{Bytes, Error, Read};

impl<'a> Read<'a> for u8 {
    #[inline]
    fn read(buf: &mut &[u8]) -> Result<Self, Error> {
        buf.u8()
    }
}

impl<'a> Read<'a> for i8 {
    #[inline]
    fn read(buf: &mut &[u8]) -> Result<Self, Error> {
        buf.i8()
    }
}

impl<'a> Read<'a> for u16 {
    #[inline]
    fn read(buf: &mut &[u8]) -> Result<Self, Error> {
        buf.u16()
    }
}

impl<'a> Read<'a> for i16 {
    #[inline]
    fn read(buf: &mut &[u8]) -> Result<Self, Error> {
        buf.i16()
    }
}

impl<'a> Read<'a> for u32 {
    #[inline]
    fn read(buf: &mut &[u8]) -> Result<Self, Error> {
        buf.u32()
    }
}

impl<'a> Read<'a> for i32 {
    #[inline]
    fn read(buf: &mut &[u8]) -> Result<Self, Error> {
        buf.i32()
    }
}

impl<'a> Read<'a> for u64 {
    #[inline]
    fn read(buf: &mut &[u8]) -> Result<Self, Error> {
        buf.u64()
    }
}

impl<'a> Read<'a> for i64 {
    #[inline]
    fn read(buf: &mut &[u8]) -> Result<Self, Error> {
        buf.i64()
    }
}

impl<'a> Read<'a> for f32 {
    #[inline]
    fn read(buf: &mut &[u8]) -> Result<Self, Error> {
        buf.f32()
    }
}

impl<'a> Read<'a> for f64 {
    #[inline]
    fn read(buf: &mut &[u8]) -> Result<Self, Error> {
        buf.f64()
    }
}

impl<'a> Read<'a> for bool {
    #[inline]
    fn read(buf: &mut &[u8]) -> Result<Self, Error> {
        Ok(buf.u8()? == 1)
    }
}

impl<'a> Read<'a> for uuid::Uuid {
    #[inline]
    fn read(buf: &mut &[u8]) -> Result<Self, Error> {
        Ok(Self::from_bytes(*buf.array()?))
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
