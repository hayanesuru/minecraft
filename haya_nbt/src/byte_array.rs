use alloc::vec::Vec;
use mser::{Error, Read};

pub(crate) const fn u8_to_i8_slice(x: &[u8]) -> &[i8] {
    unsafe { core::slice::from_raw_parts(x.as_ptr().cast::<i8>(), x.len()) }
}

/// Reinterprets a slice of `i8` as a slice of `u8` without copying.
///
/// The returned slice shares the same memory as the input and has the same length.
///
/// # Examples
///
/// ```
/// let signed: &[i8] = &[-1, 0, 1];
/// let unsigned: &[u8] = i8_to_u8_slice(signed);
/// assert_eq!(unsigned, &[255u8, 0u8, 1u8]);
/// ```
pub(crate) const fn i8_to_u8_slice(x: &[i8]) -> &[u8] {
    unsafe { core::slice::from_raw_parts(x.as_ptr().cast::<u8>(), x.len()) }
}

pub(crate) struct ByteArray(pub Vec<i8>);

impl<'a> Read<'a> for ByteArray {
    /// Reads a length-prefixed sequence of bytes from `buf` and returns it as a `ByteArray`.
    ///
    /// The method consumes the length prefix from `buf`, advances `buf` past the read bytes, and converts
    /// the read byte slice into a `Vec<i8>` stored in `ByteArray`.
    ///
    /// # Errors
    ///
    /// Returns an `mser::Error` if the length prefix cannot be read or if `buf` does not contain the
    /// declared number of bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use haya_nbt::byte_array::ByteArray;
    /// use mser::Read;
    ///
    /// let mut buf: &[u8] = &[0, 0, 0, 3, 0xFF, 0x01, 0x02]; // length = 3, bytes = [0xFF, 0x01, 0x02]
    /// let ba = ByteArray::read(&mut buf).unwrap();
    /// assert_eq!(ba.0, vec![-1, 1, 2]); // 0xFF reinterpreted as -1 i8
    /// assert_eq!(buf.len(), 0); // buffer advanced past the read data
    /// ```
    fn read(buf: &mut &'a [u8]) -> Result<Self, mser::Error> {
        match buf.split_at_checked(u32::read(buf)? as usize) {
            Some((x, y)) => {
                *buf = y;
                Ok(Self(Vec::from(u8_to_i8_slice(x))))
            }
            None => Err(Error),
        }
    }
}