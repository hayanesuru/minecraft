#![no_std]

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec::Vec;
use mser::{Error, Read, Reader, V21, Write, Writer};

pub enum List<'a, T: 'a, const MAX: usize = { usize::MAX }> {
    Borrowed(&'a [T]),
    Owned(Vec<T>),
}

impl<'a, T, const MAX: usize> core::ops::Deref for List<'a, T, MAX> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        match *self {
            Self::Borrowed(b) => b,
            Self::Owned(ref o) => o,
        }
    }
}

impl<T: core::fmt::Debug, const MAX: usize> core::fmt::Debug for List<'_, T, MAX> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match *self {
            Self::Borrowed(b) => core::fmt::Debug::fmt(b, f),
            Self::Owned(ref o) => core::fmt::Debug::fmt(o, f),
        }
    }
}

impl<'a, T, const MAX: usize> alloc::borrow::Borrow<[T]> for List<'a, T, MAX> {
    fn borrow(&self) -> &[T] {
        match self {
            Self::Borrowed(x) => x,
            Self::Owned(x) => x.as_ref(),
        }
    }
}

impl<'a, T: Clone, const MAX: usize> Clone for List<'a, T, MAX> {
    fn clone(&self) -> Self {
        match *self {
            Self::Borrowed(b) => Self::Borrowed(b),
            Self::Owned(ref o) => Self::Owned(o.clone()),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        match (self, source) {
            (&mut Self::Owned(ref mut dest), Self::Owned(o)) => {
                alloc::borrow::ToOwned::clone_into(alloc::borrow::Borrow::<[T]>::borrow(o), dest)
            }
            (t, s) => *t = s.clone(),
        }
    }
}

impl<'a, T: 'a, const MAX: usize> List<'a, T, MAX> {
    pub fn as_slice(&self) -> &[T] {
        match self {
            Self::Borrowed(x) => x,
            Self::Owned(x) => x.as_ref(),
        }
    }
}

impl<'a, T: Write + 'a, const MAX: usize> Write for List<'a, T, MAX> {
    unsafe fn write(&self, w: &mut Writer) {
        unsafe {
            let x = self.as_slice();
            V21(x.len() as u32).write(w);
            for y in x {
                y.write(w);
            }
        }
    }

    fn len_s(&self) -> usize {
        let x = self.as_slice();
        let mut len = V21(x.len() as u32).len_s();
        for y in x {
            len += y.len_s();
        }
        len
    }
}

impl<'a, T: Read<'a>, const MAX: usize> Read<'a> for List<'a, T, MAX> {
    fn read(buf: &mut Reader<'a>) -> Result<Self, Error> {
        let len = V21::read(buf)?.0 as usize;
        if len > MAX {
            return Err(Error);
        }
        let mut vec = Vec::with_capacity(usize::min(len, 65536));
        for _ in 0..len {
            vec.push(T::read(buf)?);
        }
        Ok(Self::Owned(vec))
    }
}

#[derive(Clone, Debug)]
pub struct Map<'a, K: 'a, V: 'a, const MAX: usize = { usize::MAX }>(pub List<'a, (K, V), MAX>);

impl<'a, K: Write + 'a, V: Write + 'a, const MAX: usize> Write for Map<'a, K, V, MAX> {
    unsafe fn write(&self, w: &mut Writer) {
        unsafe {
            let x = self.0.as_slice();
            V21(x.len() as u32).write(w);
            for (k, v) in x {
                k.write(w);
                v.write(w);
            }
        }
    }

    fn len_s(&self) -> usize {
        let x = self.0.as_slice();
        let mut len = V21(x.len() as u32).len_s();
        for (k, v) in x {
            len += k.len_s();
            len += v.len_s();
        }
        len
    }
}

impl<'a, K: Read<'a>, V: Read<'a>, const MAX: usize> Read<'a> for Map<'a, K, V, MAX> {
    fn read(buf: &mut Reader<'a>) -> Result<Self, Error> {
        let len = V21::read(buf)?.0 as usize;
        if len > MAX {
            return Err(Error);
        }
        let mut vec = Vec::with_capacity(usize::min(len, 65536));
        for _ in 0..len {
            let k = K::read(buf)?;
            let v = V::read(buf)?;
            vec.push((k, v));
        }
        Ok(Self(List::Owned(vec)))
    }
}

#[derive(Clone, Debug)]
pub struct BoxCodec<T>(pub Box<T>);

impl<T: Write> Write for BoxCodec<T> {
    unsafe fn write(&self, w: &mut Writer) {
        unsafe { self.0.as_ref().write(w) }
    }

    fn len_s(&self) -> usize {
        self.0.as_ref().len_s()
    }
}

impl<'a, T: Read<'a>> Read<'a> for BoxCodec<T> {
    fn read(buf: &mut Reader<'a>) -> Result<Self, Error> {
        Ok(Self(Box::new(T::read(buf)?)))
    }
}
