#![no_std]

mod hex;
mod json;
mod read;
mod varint;
mod write;
mod writer;

pub use self::hex::{hex_to_u8, u8_to_hex};
pub use self::json::json_char_width_escaped;
pub use self::varint::{V7MAX, V21, V21MAX, V32, V64};
pub use self::writer::UnsafeWriter;

pub trait Write {
    /// # Safety
    ///
    /// Must write [`len_s`] bytes exactly.
    ///
    /// [`len_s`]: Write::len_s
    unsafe fn write(&self, w: &mut UnsafeWriter);

    fn len_s(&self) -> usize;
}

#[derive(Clone, Debug, Default)]
pub struct Error;

pub trait Read<'a>: Sized {
    fn read(buf: &mut &'a [u8]) -> Result<Self, Error>;
}

/// # Safety
///
/// `ptr` must be valid for writes of [`len_s`] bytes.
///
/// [`len_s`]: Write::len_s
#[inline]
pub unsafe fn write_unchecked(ptr: *mut u8, x: &(impl Write + ?Sized)) {
    unsafe {
        let mut w = UnsafeWriter(ptr);
        Write::write(x, &mut w);
        debug_assert_eq!(w.0, ptr.add(x.len_s()))
    }
}

#[cold]
pub const fn cold_path() {}

#[derive(Clone, Copy, Debug)]
pub struct ByteArray<'a, const MAX: usize = { usize::MAX }>(pub &'a [u8]);

impl<'a, const MAX: usize> Write for ByteArray<'a, MAX> {
    unsafe fn write(&self, w: &mut UnsafeWriter) {
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
    fn read(buf: &mut &'a [u8]) -> Result<Self, Error> {
        let len = V21::read(buf)?.0 as usize;
        if len > MAX {
            return Err(Error);
        }
        match buf.split_at_checked(len) {
            Some((x, y)) => {
                *buf = y;
                Ok(Self(x))
            }
            None => Err(Error),
        }
    }
}
