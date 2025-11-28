#![no_std]
#![allow(internal_features)]
#![cfg_attr(nightly, feature(core_intrinsics))]
#![cfg_attr(feature = "allocator_api", feature(allocator_api))]

#[cfg(feature = "allocator_api")]
use alloc::alloc::Allocator;
use alloc::borrow::{Cow, ToOwned};
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
#[cfg(all(feature = "allocator-api2", not(feature = "allocator_api")))]
use allocator_api2::alloc::Allocator;

extern crate alloc;

mod bytes;
mod float;
mod hex;
mod integer;
mod json;
mod read;
mod str;
mod varint;
mod write;
mod writer;

#[cfg(feature = "nbt")]
pub mod nbt;

pub use self::bytes::Bytes;
pub use self::float::parse_float;
pub use self::hex::{hex_to_u8, parse_hex, u8_to_hex};
pub use self::integer::parse_int;
pub use self::json::JsonStr;
pub use self::str::{SmolStr, SmolStrBuilder, StrExt, ToSmolStr};
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

#[must_use]
pub fn boxed(x: &(impl Write + ?Sized)) -> Box<[u8]> {
    unsafe {
        let len = x.sz();
        let mut vec = Vec::<u8>::with_capacity(len);
        write_unchecked(vec.as_mut_ptr(), x);
        vec.set_len(len);
        vec.into_boxed_slice()
    }
}

#[cfg(all(feature = "allocator-api2", not(feature = "allocator_api")))]
pub fn boxed_in<A: Allocator>(
    x: &(impl Write + ?Sized),
    alloc: A,
) -> allocator_api2::boxed::Box<[u8], A> {
    unsafe {
        let len = x.sz();
        let mut vec = allocator_api2::vec::Vec::with_capacity_in(len, alloc);
        write_unchecked(vec.as_mut_ptr(), x);
        vec.set_len(len);
        vec.into_boxed_slice()
    }
}

#[cfg(feature = "allocator_api")]
pub fn boxed_in<A: Allocator>(x: &(impl Write + ?Sized), alloc: A) -> Box<[u8], A> {
    unsafe {
        let len = x.sz();
        let mut vec = Vec::with_capacity_in(len, alloc);
        write_unchecked(vec.as_mut_ptr(), x);
        vec.set_len(len);
        vec.into_boxed_slice()
    }
}

#[cfg(not(any(feature = "allocator-api2", feature = "allocator_api")))]
pub fn write(vec: &mut Vec<u8>, x: &(impl Write + ?Sized)) {
    unsafe {
        let len = x.sz();
        vec.reserve(len);
        write_unchecked(vec.as_mut_ptr().add(vec.len()), x);
        vec.set_len(len + vec.len());
    }
}

#[cfg(all(feature = "allocator-api2", not(feature = "allocator_api")))]
pub fn write<A: Allocator>(vec: &mut allocator_api2::vec::Vec<u8, A>, x: &(impl Write + ?Sized)) {
    unsafe {
        let len = x.sz();
        vec.reserve(len);
        write_unchecked(vec.as_mut_ptr().add(vec.len()), x);
        vec.set_len(len + vec.len());
    }
}

#[cfg(feature = "allocator_api")]
pub fn write<A: Allocator>(vec: &mut Vec<u8, A>, x: &(impl Write + ?Sized)) {
    unsafe {
        let len = x.sz();
        vec.reserve(len);
        write_unchecked(vec.as_mut_ptr().add(vec.len()), x);
        vec.set_len(len + vec.len());
    }
}

#[cfg(not(any(feature = "allocator-api2", feature = "allocator_api")))]
pub fn write_exact(vec: &mut Vec<u8>, x: &(impl Write + ?Sized)) {
    unsafe {
        let len = x.sz();
        vec.reserve_exact(len);
        write_unchecked(vec.as_mut_ptr().add(vec.len()), x);
        vec.set_len(len + vec.len());
    }
}

#[cfg(all(feature = "allocator-api2", not(feature = "allocator_api")))]
pub fn write_exact<A: Allocator>(
    vec: &mut allocator_api2::vec::Vec<u8, A>,
    x: &(impl Write + ?Sized),
) {
    unsafe {
        let len = x.sz();
        vec.reserve_exact(len);
        write_unchecked(vec.as_mut_ptr().add(vec.len()), x);
        vec.set_len(len + vec.len());
    }
}

#[cfg(feature = "allocator_api")]
pub fn write_exact<A: Allocator>(vec: &mut Vec<u8, A>, x: &(impl Write + ?Sized)) {
    unsafe {
        let len = x.sz();
        vec.reserve_exact(len);
        write_unchecked(vec.as_mut_ptr().add(vec.len()), x);
        vec.set_len(len + vec.len());
    }
}

pub fn json_str_escape(buf: &mut String, b: &[u8]) {
    unsafe {
        let e = JsonStr(b);
        let wlen = e.sz();
        buf.reserve(wlen);
        e.write(&mut UnsafeWriter(core::ptr::NonNull::new_unchecked(
            buf.as_mut_ptr().add(buf.len()),
        )));
        let len = buf.len() + wlen;
        buf.as_mut_vec().set_len(len);
    }
}

impl<T: ?Sized + Write + ToOwned> Write for Cow<'_, T> {
    #[inline]
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            <Self as AsRef<T>>::as_ref(self).write(w);
        }
    }

    #[inline]
    fn sz(&self) -> usize {
        <Self as AsRef<T>>::as_ref(self).sz()
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

#[test]
fn test_write() {
    let mut vec = Vec::<u8>::new();
    write(&mut vec, &1u32);
    assert_eq!(&vec[..], &[0u8, 0, 0, 1]);
}
