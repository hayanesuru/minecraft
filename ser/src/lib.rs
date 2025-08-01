#![no_std]
#![allow(internal_features)]
#![cfg_attr(nightly, feature(core_intrinsics))]

extern crate alloc;

mod bytes;
mod float;
mod hex;
mod integer;
mod json;
mod read;
mod varint;
mod write;
mod writer;

#[cfg(feature = "nbt")]
pub mod nbt;

pub use self::bytes::Bytes;
pub use self::float::parse_float;
pub use self::hex::{hex_to_u8, parse_hex, u8_to_hex};
pub use self::integer::parse_int;
pub use self::json::{json_str_escape, JsonStr};
pub use self::varint::{V21, V21MAX, V32, V64, V7MAX};
pub use self::write::{Write2, Write3};
pub use self::writer::UnsafeWriter;

/// # Safety
///
/// `sz` must be the size of `write` to be written.
pub unsafe trait Write {
    /// # Safety
    ///
    /// `sz` must be the size of `write` to be written.
    unsafe fn write(&self, w: &mut UnsafeWriter);

    /// # Safety
    ///
    /// `sz` must be the size of `write` to be written.
    unsafe fn sz(&self) -> usize;
}

pub trait Read: Sized {
    fn read(buf: &mut &[u8]) -> Option<Self>;
}

#[must_use]
pub fn boxed(x: &(impl Write + ?Sized)) -> alloc::boxed::Box<[u8]> {
    unsafe {
        let len = x.sz();
        let mut vec = alloc::vec::Vec::<u8>::with_capacity(len);
        write_unchecked(vec.as_mut_ptr(), x);
        vec.set_len(len);
        vec.into_boxed_slice()
    }
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

#[inline]
pub fn write(vec: &mut alloc::vec::Vec<u8>, x: &(impl Write + ?Sized)) {
    unsafe {
        let len = x.sz();
        vec.reserve(len);
        write_unchecked(vec.as_mut_ptr().add(vec.len()), x);
        vec.set_len(len + vec.len());
    }
}

#[inline]
pub fn write_exact(vec: &mut alloc::vec::Vec<u8>, x: &(impl Write + ?Sized)) {
    unsafe {
        let len = x.sz();
        vec.reserve_exact(len);
        write_unchecked(vec.as_mut_ptr().add(vec.len()), x);
        vec.set_len(len + vec.len());
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
