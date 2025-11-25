use crate::{unlikely, Error, Read as _, V32, V64};
use core::slice::from_raw_parts;

pub trait Bytes<'a>: 'a {
    fn u8(&mut self) -> Result<u8, Error>;
    fn i8(&mut self) -> Result<i8, Error>;
    fn u16(&mut self) -> Result<u16, Error>;
    fn i16(&mut self) -> Result<i16, Error>;
    fn u32(&mut self) -> Result<u32, Error>;
    fn i32(&mut self) -> Result<i32, Error>;
    fn u64(&mut self) -> Result<u64, Error>;
    fn i64(&mut self) -> Result<i64, Error>;
    fn f32(&mut self) -> Result<f32, Error>;
    fn f64(&mut self) -> Result<f64, Error>;
    fn v32(&mut self) -> Result<u32, Error>;
    fn v64(&mut self) -> Result<u64, Error>;
    fn array<const L: usize>(&mut self) -> Result<&'a [u8; L], Error>;
    fn slice(&mut self, mid: usize) -> Result<&'a [u8], Error>;
    fn peek<const L: usize>(&self) -> Result<&'a [u8; L], Error>;
    fn at(&self, index: usize) -> Result<u8, Error>;
    fn peek1(&self) -> Result<u8, Error>;
}

#[allow(clippy::manual_map)]
impl<'a> Bytes<'a> for &'a [u8] {
    #[inline]
    fn u8(&mut self) -> Result<u8, Error> {
        if let [a, b @ ..] = self {
            *self = b;
            Ok(*a)
        } else {
            Err(Error)
        }
    }

    #[inline]
    fn i8(&mut self) -> Result<i8, Error> {
        if let [a, b @ ..] = self {
            *self = b;
            Ok(*a as i8)
        } else {
            Err(Error)
        }
    }

    #[inline]
    fn u16(&mut self) -> Result<u16, Error> {
        match self.array() {
            Ok(&x) => Ok(u16::from_be_bytes(x)),
            Err(e) => Err(e),
        }
    }

    #[inline]
    fn i16(&mut self) -> Result<i16, Error> {
        match self.array() {
            Ok(&x) => Ok(i16::from_be_bytes(x)),
            Err(e) => Err(e),
        }
    }

    #[inline]
    fn u32(&mut self) -> Result<u32, Error> {
        match self.array() {
            Ok(&x) => Ok(u32::from_be_bytes(x)),
            Err(e) => Err(e),
        }
    }

    #[inline]
    fn i32(&mut self) -> Result<i32, Error> {
        match self.array() {
            Ok(&x) => Ok(i32::from_be_bytes(x)),
            Err(e) => Err(e),
        }
    }

    #[inline]
    fn u64(&mut self) -> Result<u64, Error> {
        match self.array() {
            Ok(&x) => Ok(u64::from_be_bytes(x)),
            Err(e) => Err(e),
        }
    }

    #[inline]
    fn i64(&mut self) -> Result<i64, Error> {
        match self.array() {
            Ok(&x) => Ok(i64::from_be_bytes(x)),
            Err(e) => Err(e),
        }
    }

    #[inline]
    fn f32(&mut self) -> Result<f32, Error> {
        match self.array() {
            Ok(&x) => Ok(f32::from_be_bytes(x)),
            Err(e) => Err(e),
        }
    }

    #[inline]
    fn f64(&mut self) -> Result<f64, Error> {
        match self.array() {
            Ok(&x) => Ok(f64::from_be_bytes(x)),
            Err(e) => Err(e),
        }
    }

    #[inline]
    fn v32(&mut self) -> Result<u32, Error> {
        match V32::read(self) {
            Ok(x) => Ok(x.0),
            Err(e) => Err(e),
        }
    }

    #[inline]
    fn v64(&mut self) -> Result<u64, Error> {
        match V64::read(self) {
            Ok(x) => Ok(x.0),
            Err(e) => Err(e),
        }
    }

    #[inline]
    fn array<const L: usize>(&mut self) -> Result<&'a [u8; L], Error> {
        if unlikely(L > self.len()) {
            return Err(Error);
        }
        let len = self.len();
        let ptr = self.as_ptr();
        unsafe {
            let (a, b) = (from_raw_parts(ptr, L), from_raw_parts(ptr.add(L), len - L));
            *self = b;
            Ok(&*a.as_ptr().cast())
        }
    }

    #[inline]
    fn slice(&mut self, mid: usize) -> Result<&'a [u8], Error> {
        if unlikely(mid > self.len()) {
            return Err(Error);
        }
        let len = self.len();
        let ptr = self.as_ptr();
        unsafe {
            let (a, b) = (
                from_raw_parts(ptr, mid),
                from_raw_parts(ptr.add(mid), len - mid),
            );
            *self = b;
            Ok(a)
        }
    }

    #[inline]
    fn peek<const L: usize>(&self) -> Result<&'a [u8; L], Error> {
        if unlikely(L > self.len()) {
            return Err(Error);
        }
        unsafe { Ok(&*from_raw_parts(self.as_ptr(), L).as_ptr().cast()) }
    }

    #[inline]
    fn at(&self, index: usize) -> Result<u8, Error> {
        match self.get(index) {
            Some(x) => Ok(*x),
            None => Err(Error),
        }
    }

    #[inline]
    fn peek1(&self) -> Result<u8, Error> {
        if let [a, ..] = self {
            Ok(*a)
        } else {
            Err(Error)
        }
    }
}
