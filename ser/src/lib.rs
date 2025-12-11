#![no_std]
#![allow(internal_features)]
#![cfg_attr(nightly, feature(core_intrinsics))]

mod bytes;
mod float;
mod hex;
mod integer;
mod json;
mod read;
mod varint;
mod write;
mod writer;

pub use self::bytes::Bytes;
pub use self::float::parse_float;
pub use self::hex::{hex_to_u8, parse_hex, u8_to_hex};
pub use self::integer::parse_int;
pub use self::json::JsonStr;
pub use self::varint::{V21, V21MAX, V32, V64, V7MAX};
pub use self::write::{Write2, Write3};
pub use self::writer::UnsafeWriter;

/// # Safety
///
/// `sz` must be the size of `write` to be written.
pub trait Write {
    /// # Safety
    ///
    /// [`sz`] must be the size of `write` to be written.
    ///
    /// [`sz`]: Write::sz
    unsafe fn write(&self, w: &mut UnsafeWriter);

    fn sz(&self) -> usize;
}

#[derive(Clone, Debug)]
pub struct Error;

pub trait Read<'a>: Sized
where
    Self: 'a,
{
    fn read(buf: &mut &'a [u8]) -> Result<Self, Error>;
}

/// # Safety
///
/// `ptr` must be valid for writes of `x.sz()` bytes.
#[inline]
pub unsafe fn write_unchecked(ptr: *mut u8, x: &(impl Write + ?Sized)) {
    unsafe {
        let mut w = UnsafeWriter(core::ptr::NonNull::new_unchecked(ptr));
        Write::write(x, &mut w);
        debug_assert_eq!(w.0, core::ptr::NonNull::new_unchecked(ptr.add(x.sz())))
    }
}

#[inline(always)]
#[cfg(not(nightly))]
pub const fn unlikely(b: bool) -> bool {
    #[allow(clippy::needless_bool)]
    if (1i32).checked_div(if b { 0 } else { 1 }).is_none() {
        true
    } else {
        false
    }
}

#[inline(always)]
#[cfg(not(nightly))]
pub const fn likely(b: bool) -> bool {
    #[allow(clippy::needless_bool)]
    if (1i32).checked_div(if b { 1 } else { 0 }).is_some() {
        true
    } else {
        false
    }
}

#[inline(always)]
#[cfg(nightly)]
pub const fn unlikely(b: bool) -> bool {
    ::core::intrinsics::unlikely(b)
}

#[inline(always)]
#[cfg(nightly)]
pub const fn likely(b: bool) -> bool {
    ::core::intrinsics::likely(b)
}
