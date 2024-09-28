#![no_std]

use core::ops::{Add, AddAssign};

extern crate alloc;

mod bytes;
mod float;
mod hex;
mod integer;
mod json;
mod varint;
mod write;
mod writer;

pub mod nbt;

pub use self::bytes::Bytes;
pub use self::float::parse_float;
pub use self::hex::{hex_to_u8, parse_hex, u8_to_hex};
pub use self::integer::parse_int;
pub use self::json::{json_str_escape, JsonStr};
pub use self::varint::{V21, V21MAX, V32, V64, V7MAX};
pub use self::write::{Write2, Write3};
pub use self::writer::UnsafeWriter;

#[allow(clippy::len_without_is_empty)]
pub trait Write {
    fn write(&self, w: &mut UnsafeWriter);

    fn sz(&self) -> usize;
}

pub trait Read: Sized {
    fn read(buf: &mut &[u8]) -> Option<Self>;
}

impl<T: Write + ?Sized> AddAssign<&T> for UnsafeWriter {
    #[inline]
    fn add_assign(&mut self, rhs: &T) {
        rhs.write(self);
    }
}

impl<T: Write + ?Sized> Add<&T> for UnsafeWriter {
    type Output = Self;

    #[inline]
    fn add(mut self, rhs: &T) -> Self::Output {
        rhs.write(&mut self);
        self
    }
}

#[must_use]
pub fn boxed(x: &impl Write) -> alloc::boxed::Box<[u8]> {
    let len = x.sz();
    let mut vec = alloc::vec::Vec::<u8>::with_capacity(len);
    let mut w = UnsafeWriter(vec.as_mut_ptr());
    Write::write(x, &mut w);
    unsafe { debug_assert_eq!(w.0, vec.as_mut_ptr().add(len)) };
    unsafe { vec.set_len(len) };

    vec.into_boxed_slice()
}

pub fn write(vec: &mut alloc::vec::Vec<u8>, x: &impl Write) {
    let len = x.sz();
    vec.reserve(len);
    let mut w = unsafe { UnsafeWriter(vec.as_mut_ptr().add(vec.len())) };
    Write::write(x, &mut w);
    unsafe { debug_assert_eq!(w.0, vec.as_mut_ptr().add(vec.len()).add(len)) };
    unsafe { vec.set_len(len + vec.len()) };
}

pub fn write_exact(vec: &mut alloc::vec::Vec<u8>, x: &impl Write) {
    let len = x.sz();
    vec.reserve_exact(len);
    let mut w = unsafe { UnsafeWriter(vec.as_mut_ptr().add(vec.len())) };
    Write::write(x, &mut w);
    unsafe { debug_assert_eq!(w.0, vec.as_mut_ptr().add(vec.len()).add(len)) };
    unsafe { vec.set_len(len + vec.len()) };
}
