pub const fn u8_to_i8_slice(x: &[u8]) -> &[i8] {
    unsafe { core::slice::from_raw_parts(x.as_ptr().cast::<i8>(), x.len()) }
}

pub const fn i8_to_u8_slice(x: &[i8]) -> &[u8] {
    unsafe { core::slice::from_raw_parts(x.as_ptr().cast::<u8>(), x.len()) }
}
