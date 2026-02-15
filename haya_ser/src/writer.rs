pub struct UnsafeWriter(pub(crate) *mut u8);

impl UnsafeWriter {
    /// # Safety
    ///
    /// must be vaild for write
    #[inline(always)]
    pub unsafe fn write_byte(&mut self, byte: u8) {
        unsafe {
            self.0.write(byte);
            self.0 = self.0.add(1);
        }
    }

    /// # Safety
    ///
    /// must be vaild for write
    #[inline(always)]
    pub unsafe fn write(&mut self, slice: &[u8]) {
        unsafe {
            core::ptr::copy_nonoverlapping(slice.as_ptr(), self.0, slice.len());
            self.0 = self.0.add(slice.len());
        }
    }

    #[inline(always)]
    pub fn ptr(&mut self) -> *mut u8 {
        self.0
    }
}
