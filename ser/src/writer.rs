use core::ptr::NonNull;

pub struct UnsafeWriter(pub(crate) NonNull<u8>);

impl UnsafeWriter {
    /// # Safety
    pub const unsafe fn new(ptr: *mut u8) -> Self {
        Self(unsafe { NonNull::new_unchecked(ptr) })
    }

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

    /// # Safety
    #[inline(always)]
    pub unsafe fn offset(&mut self, ptr: NonNull<u8>) -> usize {
        unsafe { self.0.as_ptr().offset_from_unsigned(ptr.as_ptr()) }
    }
}
