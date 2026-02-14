#![no_std]

mod hex;
mod json;
mod mutf8;
mod read;
mod varint;
mod write;
mod writer;

pub use self::hex::{hex_to_u8, parse_hex, u8_to_hex};
pub use self::json::json_char_width_escaped;
pub use self::mutf8::{
    decode_mutf8, decode_mutf8_len, encode_mutf8, encode_mutf8_len, is_ascii_mutf8, is_mutf8,
};
pub use self::varint::{V7MAX, V21, V21MAX, V32, V64};
pub use self::write::{Write2, Write3};
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

#[derive(Clone, Debug)]
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
        let mut w = UnsafeWriter(core::ptr::NonNull::new_unchecked(ptr));
        Write::write(x, &mut w);
        debug_assert_eq!(w.0, core::ptr::NonNull::new_unchecked(ptr.add(x.len_s())))
    }
}

#[cold]
pub const fn cold_path() {}

pub const fn hash128(n: &[u8], seed: u64) -> [u64; 2] {
    const M: u64 = 0xc6a4a7935bd1e995;
    const N: u128 = 0xdbe6d5d5fe4cce213198a2e03707344u128;
    let mut h: u64 = seed ^ ((n.len() as u64).wrapping_mul(M));
    let mut i = 0;
    while i + 8 <= n.len() {
        h ^= u64::from_le_bytes(unsafe { *(n.as_ptr().add(i) as *const [u8; 8]) }).wrapping_mul(M);
        i += 8;
    }
    while i < n.len() {
        h ^= (unsafe { *n.as_ptr().add(i) } as u64) << ((i & 7) * 8);
        i += 1;
    }
    let h = (h as u128).wrapping_mul(N);
    let h = h ^ (h >> 64);
    let h = h.wrapping_mul(N);
    [(h >> 64) as u64, h as u64]
}
