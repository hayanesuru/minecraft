use alloc::vec::Vec;
use mser::{Error, Read};

#[derive(Clone)]
pub(crate) struct IntArray(pub Vec<i32>);

impl<'a> Read<'a> for IntArray {
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
