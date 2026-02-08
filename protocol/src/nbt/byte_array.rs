use alloc::vec::Vec;
use mser::{Error, Read};

pub const fn u8_to_i8_slice(x: &[u8]) -> &[i8] {
    unsafe { core::slice::from_raw_parts(x.as_ptr().cast::<i8>(), x.len()) }
}

pub const fn i8_to_u8_slice(x: &[i8]) -> &[u8] {
    unsafe { core::slice::from_raw_parts(x.as_ptr().cast::<u8>(), x.len()) }
}

pub struct ByteArray(pub Vec<i8>);

impl<'a> Read<'a> for ByteArray {
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
