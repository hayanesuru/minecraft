use crate::boxed_slice_as_array_unchecked;

#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct NibbleArray {
    pub data: Box<[u8; 2048]>,
}

impl Default for NibbleArray {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl NibbleArray {
    #[inline]
    pub fn new() -> Self {
        let data = vec![0_u8; 2048].into_boxed_slice();
        let data = unsafe { boxed_slice_as_array_unchecked(data) };
        Self { data }
    }

    #[inline]
    pub fn new_with_f() -> Self {
        let data = vec![0xff_u8; 2048].into_boxed_slice();
        let data = unsafe { boxed_slice_as_array_unchecked(data) };
        Self { data }
    }

    #[inline]
    pub const fn get_index(x: usize, y: usize, z: usize) -> usize {
        (y << 8) | (z << 4) | x
    }

    #[inline]
    pub const fn as_bytes(&self) -> &[u8; 2048] {
        &self.data
    }

    #[inline]
    pub fn get(&self, idx: usize) -> u8 {
        let y = unsafe { *self.data.get_unchecked(idx >> 1) };
        if 0 == idx & 1 {
            y & 0xF
        } else {
            y >> 4
        }
    }

    #[inline]
    pub fn set(&mut self, index: usize, val: u8) {
        let x = unsafe { self.data.get_unchecked_mut(index >> 1) };
        if index & 1 == 0 {
            *x = (*x & 0xF0) | val;
        } else {
            *x = (*x & 0x0F) | (val << 4);
        }
    }
}
