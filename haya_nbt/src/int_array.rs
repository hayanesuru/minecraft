use alloc::vec::Vec;
use mser::{Error, Read};

#[derive(Clone)]
pub(crate) struct IntArray(pub Vec<i32>);

impl<'a> Read<'a> for IntArray {
    /// Deserializes an `IntArray` from the start of a byte slice, advancing the slice past the consumed bytes.
    ///
    /// Reads a 4-byte unsigned big-endian length, then reads that many 4-byte big-endian signed integers
    /// into the returned `IntArray`. If the buffer does not contain enough bytes for the declared length,
    /// an `mser::Error` is returned and the input slice is not advanced.
    ///
    /// # Examples
    ///
    /// ```
    /// // bytes: length = 2, values = [1, 2]
    /// let mut buf: &[u8] = &[0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0, 2];
    /// let mut s = buf;
    /// let arr = crate::int_array::IntArray::read(&mut s).unwrap();
    /// assert_eq!(arr.0, vec![1, 2]);
    /// // s has been advanced past the consumed bytes
    /// assert_eq!(s.len(), 0);
    /// ```
    fn read(buf: &mut &'a [u8]) -> Result<Self, mser::Error> {
        let len = u32::read(buf)? as usize;
        let data = match buf.split_at_checked(len * 4) {
            Some((x, y)) => {
                *buf = y;
                x
            }
            None => return Err(Error),
        };
        let mut vec = Vec::with_capacity(len);
        unsafe { copy_swap(len, data.as_ptr(), vec.as_mut_ptr()) }
        unsafe { vec.set_len(len) }
        Ok(Self(vec))
    }
}

unsafe fn copy_swap(len: usize, mut src: *const u8, mut dst: *mut i32) {
    unsafe {
        for _ in 0..len {
            *dst = i32::from_be_bytes(*src.cast::<[u8; 4]>());
            src = src.add(4);
            dst = dst.add(1);
        }
    }
}