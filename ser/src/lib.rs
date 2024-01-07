#![feature(ptr_sub_ptr)]
#![no_std]

use core::ops::{Add, AddAssign};

extern crate alloc;

mod bytes;
mod float;
mod hex;
mod integer;
mod json;
mod mutf8;
mod snbt;
mod varint;
mod write;
mod writer;

pub mod text;
pub mod nbt;

pub use self::bytes::Bytes;
pub use self::float::parse_float;
pub use self::hex::{hex_to_u8, parse_hex, u8_to_hex};
pub use self::integer::parse_int;
pub use self::json::{json_str_escape, JsonStr};
pub use self::snbt::Snbt;
pub use self::varint::{V21, V21MAX, V32, V64, V7MAX};
pub use self::writer::UnsafeWriter;

#[allow(clippy::len_without_is_empty)]
pub trait Write {
    fn write(&self, w: &mut UnsafeWriter);

    fn len(&self) -> usize;
}

pub trait Read: Sized {
    fn read(buf: &mut &[u8]) -> Option<Self>;
}

impl<T: Write> AddAssign<T> for UnsafeWriter {
    #[inline]
    fn add_assign(&mut self, rhs: T) {
        rhs.write(self);
    }
}

impl<T: Write> Add<T> for UnsafeWriter {
    type Output = Self;

    #[inline]
    fn add(mut self, rhs: T) -> Self::Output {
        rhs.write(&mut self);
        self
    }
}
