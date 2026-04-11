#![no_std]

mod memchr;
mod read;
mod reader;
mod varint;
mod write;
mod writer;

use core::marker::PhantomData;

#[derive(Debug)]
pub struct Reader<'a> {
    pub(crate) ptr: *const u8,
    pub(crate) end: *const u8,
    pub(crate) marker: PhantomData<&'a [u8]>,
}

#[derive(Debug)]
pub struct Writer(pub(crate) *mut u8);

pub trait Write {
    /// # Safety
    ///
    /// Must write [`len_s`] bytes exactly.
    ///
    /// [`len_s`]: Write::len_s
    unsafe fn write(&self, w: &mut Writer);

    fn len_s(&self) -> usize;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Error;

pub trait Read<'a>: Sized {
    fn read(buf: &mut Reader<'a>) -> Result<Self, Error>;
}

pub const V21MAX: usize = 0x1FFFFF;
pub const V7MAX: usize = 0x7F;

#[repr(transparent)]
#[must_use]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct V21(pub u32);

#[repr(transparent)]
#[must_use]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct V32(pub u32);

#[repr(transparent)]
#[must_use]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct V64(pub u64);

/// # Safety
///
/// `ptr` must be valid for writes of [`len_s`] bytes.
///
/// [`len_s`]: Write::len_s
#[inline]
pub unsafe fn write_unchecked(ptr: *mut u8, x: &(impl Write + ?Sized)) {
    unsafe {
        let mut w = Writer(ptr);
        Write::write(x, &mut w);
        debug_assert_eq!(w.0, ptr.add(x.len_s()))
    }
}

#[cold]
pub const fn cold_path() {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ByteArray<'a, const MAX: usize = { usize::MAX }>(pub &'a [u8]);

impl<'a, const MAX: usize> Write for ByteArray<'a, MAX> {
    unsafe fn write(&self, w: &mut Writer) {
        unsafe {
            V21(self.0.len() as u32).write(w);
            w.write(self.0);
        }
    }

    fn len_s(&self) -> usize {
        V21(self.0.len() as u32).len_s() + self.0.len()
    }
}

impl<'a, const MAX: usize> Read<'a> for ByteArray<'a, MAX> {
    fn read(buf: &mut Reader<'a>) -> Result<Self, Error> {
        let len = V21::read(buf)?.0 as usize;
        if len > MAX {
            return Err(Error);
        }
        Ok(Self(buf.read_slice(len)?))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Utf8<'a, const MAX: usize = 32767>(pub &'a str);

impl<'a, const MAX: usize> Utf8<'a, MAX> {
    const MAX_BYTES: usize = MAX.checked_mul(3).unwrap();
}

impl<'a, const MAX: usize> Write for Utf8<'a, MAX> {
    #[inline]
    unsafe fn write(&self, w: &mut Writer) {
        unsafe {
            V21(self.0.len() as u32).write(w);
            w.write(self.0.as_bytes());
        }
    }

    #[inline]
    fn len_s(&self) -> usize {
        V21(self.0.len() as u32).len_s() + self.0.len()
    }
}

impl<'a, const MAX: usize> Read<'a> for Utf8<'a, MAX> {
    #[inline]
    fn read(buf: &mut Reader<'a>) -> Result<Self, Error> {
        let len = V21::read(buf)?.0 as usize;
        if len > Self::MAX_BYTES {
            return Err(Error);
        }
        let bytes = buf.read_slice(len)?;
        let s = match core::str::from_utf8(bytes) {
            Ok(x) => x,
            Err(_) => return Err(Error),
        };
        if s.chars().map(|x| x.len_utf16()).sum::<usize>() <= MAX {
            Ok(Utf8(s))
        } else {
            Err(Error)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<'a, L: Read<'a>, R: Read<'a>> Read<'a> for Either<L, R> {
    fn read(buf: &mut Reader<'a>) -> Result<Self, Error> {
        if bool::read(buf)? {
            Ok(Self::Left(L::read(buf)?))
        } else {
            Ok(Self::Right(R::read(buf)?))
        }
    }
}

impl<L: Write, R: Write> Write for Either<L, R> {
    unsafe fn write(&self, w: &mut Writer) {
        unsafe {
            match self {
                Self::Left(l) => {
                    true.write(w);
                    l.write(w);
                }
                Self::Right(r) => {
                    false.write(w);
                    r.write(w);
                }
            }
        }
    }

    fn len_s(&self) -> usize {
        match self {
            Self::Left(l) => true.len_s() + l.len_s(),
            Self::Right(r) => false.len_s() + r.len_s(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Rest<'a, const MAX: usize = { usize::MAX }>(pub &'a [u8]);

impl<'a, const MAX: usize> Read<'a> for Rest<'a, MAX> {
    fn read(buf: &mut Reader<'a>) -> Result<Self, Error> {
        let len = buf.len();
        if len > MAX {
            return Err(Error);
        }
        Ok(Self(buf.read_slice(len)?))
    }
}

impl<'a, const MAX: usize> Write for Rest<'a, MAX> {
    unsafe fn write(&self, w: &mut Writer) {
        unsafe { w.write(self.0) }
    }

    fn len_s(&self) -> usize {
        self.0.len()
    }
}
