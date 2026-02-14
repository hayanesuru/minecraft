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

/// Computes a 128-bit hash of a byte slice using the provided seed.
///
/// The result is a deterministic 128-bit value derived from `n` and `seed`.
///
/// # Examples
///
/// ```
/// let a = hash128(b"hello", 0);
/// let b = hash128(b"hello", 1);
/// assert_ne!(a, b);
/// ```
///
/// # Returns
///
/// A `[u64; 2]` where element `0` is the high 64 bits and element `1` is the low 64 bits of the 128-bit hash.
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

#[derive(Clone, Copy, Debug)]
pub struct ByteArray<'a, const MAX: usize = { usize::MAX }>(pub &'a [u8]);

impl<'a, const MAX: usize> Write for ByteArray<'a, MAX> {
    /// Writes the byte slice as a V21 length-prefixed payload into the provided `UnsafeWriter`.
    ///
    /// # Safety
    ///
    /// The caller must ensure `w` points to a writable region large enough to accept exactly
    /// `self.len_s()` bytes; violating this may cause undefined behavior.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let payload = b"hello";
    /// let ba = haya_ser::ByteArray::<{ usize::MAX }>(payload.as_ref());
    ///
    /// // prepare a buffer large enough for the length prefix + payload
    /// let mut buf = [0u8; 16];
    /// let mut uw = haya_ser::UnsafeWriter::new(&mut buf);
    ///
    /// unsafe {
    ///     ba.write(&mut uw);
    /// }
    /// // after writing, the first bytes of `buf` contain the V21 length prefix followed by "hello"
    /// ```
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            V21(self.0.len() as u32).write(w);
            w.write(self.0);
        }
    }

    /// Compute the number of bytes that will be written for this `ByteArray`.
    ///
    /// The result is the size of the V21-encoded length prefix followed by the payload length.
    ///
    /// # Examples
    ///
    /// ```
    /// let ba = ByteArray(b"abc");
    /// let expected = V21(ba.0.len() as u32).len_s() + ba.0.len();
    /// assert_eq!(ba.len_s(), expected);
    /// ```
    fn len_s(&self) -> usize {
        V21(self.0.len() as u32).len_s() + self.0.len()
    }
}

impl<'a, const MAX: usize> Read<'a> for ByteArray<'a, MAX> {
    /// Reads a length-prefixed byte slice from `buf` and returns a `ByteArray` referencing that slice.
    ///
    /// The function first reads a V21 length prefix, verifies the length does not exceed `MAX`,
    /// then attempts to take that many bytes from `buf`. On success `buf` is advanced past the
    /// consumed bytes and a `ByteArray` containing the slice is returned.
    ///
    /// # Errors
    ///
    /// Returns `Err(Error)` if the decoded length is greater than `MAX` or if `buf` does not contain
    /// enough bytes for the requested length.
    ///
    /// # Examples
    ///
    /// ```
    /// // Buffer layout: [len = 1 (V21), payload = b'a']
    /// let mut buf: &[u8] = &[0x01, b'a'];
    /// let ba = ByteArray::<10>::read(&mut buf).unwrap();
    /// assert_eq!(ba.0, b"a");
    /// assert_eq!(buf, &[] as &[u8]); // remaining buffer is empty
    /// ```
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