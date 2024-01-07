use core::ptr::copy_nonoverlapping;

pub struct UnsafeWriter(pub *mut u8);

impl UnsafeWriter {
    #[inline(always)]
    pub fn write_byte(&mut self, byte: u8) {
        unsafe {
            *self.0 = byte;
            self.0 = self.0.add(1);
        }
    }

    #[inline(always)]
    pub fn write(&mut self, slice: &[u8]) {
        unsafe {
            copy_nonoverlapping(slice.as_ptr(), self.0, slice.len());
            self.0 = self.0.add(slice.len());
        }
    }

    #[inline(always)]
    pub fn ptr(&mut self) -> *mut u8 {
        self.0
    }
}

impl<T: AsRef<[u8]>> core::ops::AddAssign<T> for UnsafeWriter {
    #[inline]
    fn add_assign(&mut self, rhs: T) {
        self.write(rhs.as_ref());
    }
}

impl<T: AsRef<[u8]>> core::ops::Add<T> for UnsafeWriter {
    type Output = Self;

    #[inline]
    fn add(mut self, rhs: T) -> Self::Output {
        self.write(rhs.as_ref());
        self
    }
}
