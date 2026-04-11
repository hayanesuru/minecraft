use crate::{Error, Read, Reader};

impl<'a> Read<'a> for u8 {
    #[inline]
    fn read(buf: &mut Reader) -> Result<Self, Error> {
        buf.read_byte()
    }
}

impl<'a> Read<'a> for i8 {
    #[inline]
    fn read(buf: &mut Reader) -> Result<Self, Error> {
        Ok(buf.read_byte()? as i8)
    }
}

impl<'a> Read<'a> for u16 {
    #[inline]
    fn read(buf: &mut Reader) -> Result<Self, Error> {
        Ok(Self::from_be_bytes(*buf.read_array::<2>()?))
    }
}

impl<'a> Read<'a> for i16 {
    #[inline]
    fn read(buf: &mut Reader) -> Result<Self, Error> {
        Ok(Self::from_be_bytes(*buf.read_array::<2>()?))
    }
}

impl<'a> Read<'a> for u32 {
    #[inline]
    fn read(buf: &mut Reader) -> Result<Self, Error> {
        Ok(Self::from_be_bytes(*buf.read_array::<4>()?))
    }
}

impl<'a> Read<'a> for i32 {
    #[inline]
    fn read(buf: &mut Reader) -> Result<Self, Error> {
        Ok(Self::from_be_bytes(*buf.read_array::<4>()?))
    }
}

impl<'a> Read<'a> for u64 {
    #[inline]
    fn read(buf: &mut Reader) -> Result<Self, Error> {
        Ok(Self::from_be_bytes(*buf.read_array::<8>()?))
    }
}

impl<'a> Read<'a> for i64 {
    #[inline]
    fn read(buf: &mut Reader) -> Result<Self, Error> {
        Ok(Self::from_be_bytes(*buf.read_array::<8>()?))
    }
}

impl<'a> Read<'a> for u128 {
    #[inline]
    fn read(buf: &mut Reader) -> Result<Self, Error> {
        Ok(Self::from_be_bytes(*buf.read_array::<16>()?))
    }
}

impl<'a> Read<'a> for i128 {
    #[inline]
    fn read(buf: &mut Reader) -> Result<Self, Error> {
        Ok(Self::from_be_bytes(*buf.read_array::<16>()?))
    }
}

impl<'a> Read<'a> for f32 {
    #[inline]
    fn read(buf: &mut Reader) -> Result<Self, Error> {
        Ok(Self::from_be_bytes(*buf.read_array::<4>()?))
    }
}

impl<'a> Read<'a> for f64 {
    #[inline]
    fn read(buf: &mut Reader) -> Result<Self, Error> {
        Ok(Self::from_be_bytes(*buf.read_array::<8>()?))
    }
}

impl<'a> Read<'a> for bool {
    #[inline]
    fn read(buf: &mut Reader) -> Result<Self, Error> {
        Ok(buf.read_byte()? == 1)
    }
}

impl<'a> Read<'a> for uuid::Uuid {
    #[inline]
    fn read(buf: &mut Reader) -> Result<Self, Error> {
        Ok(Self::from_u128(u128::read(buf)?))
    }
}

impl<'a, const N: usize> Read<'a> for [u8; N] {
    #[inline]
    fn read(buf: &mut Reader<'a>) -> Result<Self, Error> {
        match buf.read_array::<N>() {
            Ok(&x) => Ok(x),
            Err(e) => Err(e),
        }
    }
}

impl<'a, T: Read<'a>> Read<'a> for Option<T> {
    #[inline]
    fn read(buf: &mut Reader<'a>) -> Result<Self, Error> {
        if bool::read(buf)? {
            Ok(Some(T::read(buf)?))
        } else {
            Ok(None)
        }
    }
}
