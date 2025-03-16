use crate::{Read as _, V32, V64};
use core::slice::from_raw_parts;

pub trait Bytes {
    fn u8(&mut self) -> Option<u8>;
    fn i8(&mut self) -> Option<i8>;
    fn u16(&mut self) -> Option<u16>;
    fn i16(&mut self) -> Option<i16>;
    fn u32(&mut self) -> Option<u32>;
    fn i32(&mut self) -> Option<i32>;
    fn u64(&mut self) -> Option<u64>;
    fn i64(&mut self) -> Option<i64>;
    fn f32(&mut self) -> Option<f32>;
    fn f64(&mut self) -> Option<f64>;
    fn v32(&mut self) -> Option<u32>;
    fn v64(&mut self) -> Option<u64>;
    fn array<'a, const L: usize>(&mut self) -> Option<&'a [u8; L]>;
    fn slice<'a>(&mut self, len: usize) -> Option<&'a [u8]>;
    fn peek<'a, const L: usize>(&self) -> Option<&'a [u8; L]>;
}

#[allow(clippy::manual_map)]
impl Bytes for &[u8] {
    #[inline]
    fn u8(&mut self) -> Option<u8> {
        if let [a, b @ ..] = self {
            *self = b;
            Some(*a)
        } else {
            None
        }
    }

    #[inline]
    fn i8(&mut self) -> Option<i8> {
        if let [a, b @ ..] = self {
            *self = b;
            Some(*a as i8)
        } else {
            None
        }
    }

    #[inline]
    fn u16(&mut self) -> Option<u16> {
        match self.array() {
            Some(&x) => Some(u16::from_be_bytes(x)),
            None => None,
        }
    }

    #[inline]
    fn i16(&mut self) -> Option<i16> {
        match self.array() {
            Some(&x) => Some(i16::from_be_bytes(x)),
            None => None,
        }
    }

    #[inline]
    fn u32(&mut self) -> Option<u32> {
        match self.array() {
            Some(&x) => Some(u32::from_be_bytes(x)),
            None => None,
        }
    }

    #[inline]
    fn i32(&mut self) -> Option<i32> {
        match self.array() {
            Some(&x) => Some(i32::from_be_bytes(x)),
            None => None,
        }
    }

    #[inline]
    fn u64(&mut self) -> Option<u64> {
        match self.array() {
            Some(&x) => Some(u64::from_be_bytes(x)),
            None => None,
        }
    }

    #[inline]
    fn i64(&mut self) -> Option<i64> {
        match self.array() {
            Some(&x) => Some(i64::from_be_bytes(x)),
            None => None,
        }
    }

    #[inline]
    fn f32(&mut self) -> Option<f32> {
        match self.array() {
            Some(&x) => Some(f32::from_be_bytes(x)),
            None => None,
        }
    }

    #[inline]
    fn f64(&mut self) -> Option<f64> {
        match self.array() {
            Some(&x) => Some(f64::from_be_bytes(x)),
            None => None,
        }
    }

    #[inline]
    fn v32(&mut self) -> Option<u32> {
        match V32::read(self) {
            Some(x) => Some(x.0),
            None => None,
        }
    }

    #[inline]
    fn v64(&mut self) -> Option<u64> {
        match V64::read(self) {
            Some(x) => Some(x.0),
            None => None,
        }
    }

    #[inline]
    fn array<'a, const L: usize>(&mut self) -> Option<&'a [u8; L]> {
        if L > self.len() {
            return None;
        }
        let len = self.len();
        let ptr = self.as_ptr();
        unsafe {
            let (a, b) = (from_raw_parts(ptr, L), from_raw_parts(ptr.add(L), len - L));
            *self = b;
            Some(&*a.as_ptr().cast())
        }
    }

    #[inline]
    fn slice<'a>(&mut self, mid: usize) -> Option<&'a [u8]> {
        if mid > self.len() {
            return None;
        }
        let len = self.len();
        let ptr = self.as_ptr();
        unsafe {
            let (a, b) = (
                from_raw_parts(ptr, mid),
                from_raw_parts(ptr.add(mid), len - mid),
            );
            *self = b;
            Some(a)
        }
    }

    #[inline]
    fn peek<'a, const L: usize>(&self) -> Option<&'a [u8; L]> {
        if L > self.len() {
            return None;
        }
        unsafe { Some(&*from_raw_parts(self.as_ptr(), L).as_ptr().cast()) }
    }
}
