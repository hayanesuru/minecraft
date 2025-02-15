use crate::{Bytes, Read};

impl Read for u8 {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        buf.u8()
    }
}

impl Read for i8 {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        buf.i8()
    }
}

impl Read for u16 {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        buf.u16()
    }
}

impl Read for i16 {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        buf.i16()
    }
}

impl Read for u32 {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        buf.u32()
    }
}

impl Read for i32 {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        buf.i32()
    }
}

impl Read for u64 {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        buf.u64()
    }
}

impl Read for i64 {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        buf.i64()
    }
}

impl Read for f32 {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        buf.f32()
    }
}

impl Read for f64 {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        buf.f64()
    }
}

impl Read for bool {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        Some(buf.u8()? == 1)
    }
}

impl Read for uuid::Uuid {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        Some(Self::from_bytes(*buf.array()?))
    }
}
