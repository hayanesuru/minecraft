use alloc::vec::Vec;
use mser::{Error, Read};

#[derive(Clone)]
pub(crate) struct LongArray(pub Vec<i64>);

impl<'a> Read<'a> for LongArray {
    fn read(buf: &mut &'a [u8]) -> Result<Self, mser::Error> {
        let len = u32::read(buf)? as usize;
        let data = match buf.split_at_checked(len.checked_mul(8).ok_or(Error)?) {
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
