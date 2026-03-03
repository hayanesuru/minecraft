use crate::{Error, Reader, cold_path};
use core::marker::PhantomData;

impl<'a> Reader<'a> {
    #[inline]
    pub const fn new(buf: &'a [u8]) -> Self {
        Self {
            ptr: buf.as_ptr(),
            end: unsafe { buf.as_ptr().add(buf.len()) },
            marker: PhantomData,
        }
    }

    #[inline]
    pub fn read_byte(&mut self) -> Result<u8, Error> {
        if self.ptr == self.end {
            cold_path();
            Err(Error)
        } else {
            let b = unsafe { *self.ptr };
            self.ptr = unsafe { self.ptr.add(1) };
            Ok(b)
        }
    }

    #[inline]
    pub fn read_array<const L: usize>(&mut self) -> Result<[u8; L], Error> {
        if unsafe { self.ptr.add(L) > self.end } {
            cold_path();
            Err(Error)
        } else {
            unsafe {
                let a = *(self.ptr as *const [u8; L]);
                self.ptr = self.ptr.add(L);
                Ok(a)
            }
        }
    }

    #[inline]
    pub fn read_slice(&mut self, len: usize) -> Result<&'a [u8], Error> {
        if unsafe { self.ptr.add(len) > self.end } {
            cold_path();
            Err(Error)
        } else {
            unsafe {
                let a = core::slice::from_raw_parts(self.ptr, len);
                self.ptr = self.ptr.add(len);
                Ok(a)
            }
        }
    }

    #[inline]
    pub fn peek(&self) -> Result<u8, Error> {
        if self.ptr == self.end {
            cold_path();
            Err(Error)
        } else {
            unsafe {
                let b = *self.ptr;
                Ok(b)
            }
        }
    }

    /// # Safety
    ///
    /// `len` must not out of bounds.
    #[inline]
    pub unsafe fn advance(&mut self, len: usize) {
        self.ptr = unsafe { self.ptr.add(len) };
    }

    #[inline]
    pub fn peek_array<const L: usize>(&self) -> Result<[u8; L], Error> {
        if unsafe { self.ptr.add(L) > self.end } {
            cold_path();
            Err(Error)
        } else {
            unsafe {
                let a = *(self.ptr as *const [u8; L]);
                Ok(a)
            }
        }
    }

    #[inline]
    pub fn peek_slice(&self, len: usize) -> Result<&'a [u8], Error> {
        if unsafe { self.ptr.add(len) > self.end } {
            cold_path();
            Err(Error)
        } else {
            unsafe {
                let a = core::slice::from_raw_parts(self.ptr, len);
                Ok(a)
            }
        }
    }

    #[inline]
    pub const fn len(&self) -> usize {
        unsafe { self.end.offset_from_unsigned(self.ptr) }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.ptr == self.end
    }
}
