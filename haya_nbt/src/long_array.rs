use alloc::vec::Vec;
use mser::{Error, Read};

#[derive(Clone)]
pub(crate) struct LongArray(pub Vec<i64>);

impl<'a> Read<'a> for LongArray {
    /// Reads a `LongArray` from the start of `buf`, consuming the length field and the specified number of big-endian 64-bit integers.
    ///
    /// The function first reads a 32-bit unsigned length, then reads `length` consecutive i64 values encoded as big-endian bytes, and advances `buf` past the consumed bytes. Returns an error if the buffer does not contain enough bytes for the declared length.
    ///
    /// # Examples
    ///
    /// ```
    /// use haya_nbt::long_array::LongArray;
    /// use mser::Read;
    ///
    /// let mut bytes: &[u8] = &[0,0,0,2,                 // length = 2
    ///                          0,0,0,0,0,0,0,1,       // 1i64
    ///                          0,0,0,0,0,0,0,2];      // 2i64
    /// let mut slice = bytes;
    /// let arr = LongArray::read(&mut slice).unwrap();
    /// assert_eq!(arr.0, vec![1i64, 2i64]);
    /// assert!(slice.is_empty());
    /// ```
    fn read(buf: &mut &'a [u8]) -> Result<Self, mser::Error> {
        let len = u32::read(buf)? as usize;
        let data = match buf.split_at_checked(len * 8) {
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

unsafe fn copy_swap(len: usize, mut src: *const u8, mut dst: *mut i64) {
    unsafe {
        for _ in 0..len {
            *dst = i64::from_be_bytes(*src.cast::<[u8; 8]>());
            src = src.add(8);
            dst = dst.add(1);
        }
    }
}