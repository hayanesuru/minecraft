use alloc::vec::Vec;
use mser::{Error, Read, Reader};

pub(crate) const fn u8_to_i8_slice(x: &[u8]) -> &[i8] {
    unsafe { core::slice::from_raw_parts(x.as_ptr().cast::<i8>(), x.len()) }
}

pub(crate) const fn i8_to_u8_slice(x: &[i8]) -> &[u8] {
    unsafe { core::slice::from_raw_parts(x.as_ptr().cast::<u8>(), x.len()) }
}

pub(crate) struct ByteArray(pub Vec<i8>);

impl<'a> Read<'a> for ByteArray {
    fn read(buf: &mut Reader<'a>) -> Result<Self, Error> {
        let len = u32::read(buf)? as usize;
        Ok(Self(Vec::from(u8_to_i8_slice(buf.read_slice(len)?))))
    }
}
