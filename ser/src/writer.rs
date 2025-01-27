use core::ptr::NonNull;

pub struct UnsafeWriter(pub(crate) NonNull<u8>);

impl UnsafeWriter {
    /// # Safety
    #[inline(always)]
    pub unsafe fn write_byte(&mut self, byte: u8) {
        unsafe {
            self.0.write(byte);
            self.0 = self.0.add(1);
        }
    }

    /// # Safety
    #[inline(always)]
    pub unsafe fn write(&mut self, slice: &[u8]) {
        unsafe {
            core::ptr::copy_nonoverlapping(slice.as_ptr(), self.0.as_ptr(), slice.len());
            self.0 = self.0.add(slice.len());
        }
    }

    /// # Safety
    #[inline(always)]
    pub unsafe fn ptr(&mut self) -> NonNull<u8> {
        self.0
    }
}
