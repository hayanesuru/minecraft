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
        if core::ptr::eq(self.ptr, self.end) {
            cold_path();
            Err(Error)
        } else {
            let b = unsafe { *self.ptr };
            self.ptr = unsafe { self.ptr.add(1) };
            Ok(b)
        }
    }

    #[inline]
    pub fn read_array<const L: usize>(&mut self) -> Result<&'a [u8; L], Error> {
        if unsafe { self.ptr.add(L) > self.end } {
            cold_path();
            Err(Error)
        } else {
            unsafe {
                let a = &*(self.ptr as *const [u8; L]);
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
            unsafe { Ok(*self.ptr) }
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
    pub fn peek_array<const L: usize>(&self) -> Result<&'a [u8; L], Error> {
        if unsafe { self.ptr.add(L) > self.end } {
            cold_path();
            Err(Error)
        } else {
            unsafe { Ok(&*(self.ptr as *const [u8; L])) }
        }
    }

    #[inline]
    pub fn peek_slice(&self, len: usize) -> Result<&'a [u8], Error> {
        if unsafe { self.ptr.add(len) > self.end } {
            cold_path();
            Err(Error)
        } else {
            unsafe { Ok(core::slice::from_raw_parts(self.ptr, len)) }
        }
    }

    #[inline]
    pub fn peek_array_unchecked<const L: usize>(&self) -> Result<&'a [u8; L], Error> {
        debug_assert!(unsafe { self.ptr.add(L) <= self.end });
        unsafe { Ok(&*(self.ptr as *const [u8; L])) }
    }

    #[inline]
    pub fn peek_slice_unchecked(&self, len: usize) -> &'a [u8] {
        debug_assert!(unsafe { self.ptr.add(len) <= self.end });
        unsafe { core::slice::from_raw_parts(self.ptr, len) }
    }

    pub fn memchr(&self, needle: u8) -> *const u8 {
        crate::memchr::memchr(needle, self.ptr, self.end)
    }

    pub fn memchr2(&self, needle1: u8, needle2: u8) -> *const u8 {
        crate::memchr::memchr2(needle1, needle2, self.ptr, self.end)
    }

    pub fn memchr3(&self, needle1: u8, needle2: u8, needle3: u8) -> *const u8 {
        crate::memchr::memchr3(needle1, needle2, needle3, self.ptr, self.end)
    }

    pub fn position(&self, f: &[u8]) -> *const u8 {
        unsafe {
            let mut ptr = self.ptr;
            while !core::ptr::eq(ptr, self.end) {
                if f.contains(&*ptr) {
                    break;
                }
                ptr = ptr.add(1);
            }
            ptr
        }
    }

    #[inline]
    pub const fn len(&self) -> usize {
        unsafe { self.end.offset_from_unsigned(self.ptr) }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        core::ptr::eq(self.ptr, self.end)
    }

    #[inline]
    pub fn end_from(&self, ptr: *const u8) -> bool {
        core::ptr::eq(ptr, self.end)
    }

    /// # Safety
    ///
    /// `origin` >= [`self.ptr`]
    ///
    /// [`self.ptr`]: Reader::ptr
    #[inline]
    pub unsafe fn offset_from(&self, origin: *const u8) -> usize {
        unsafe { origin.offset_from_unsigned(self.ptr) }
    }

    /// # Safety
    ///
    /// `origin` >= [`self.ptr`]
    ///
    /// [`self.ptr`]: Reader::ptr
    #[inline]
    pub unsafe fn read_slice_from(&mut self, origin: *const u8) -> &'a [u8] {
        unsafe {
            let len = origin.offset_from_unsigned(self.ptr);
            let s = self.peek_slice_unchecked(len);
            self.advance(len);
            s
        }
    }

    /// # Safety
    ///
    /// `origin` >= [`self.ptr`]
    ///
    /// [`self.ptr`]: Reader::ptr
    #[inline]
    pub unsafe fn peek_slice_from(&mut self, origin: *const u8) -> &'a [u8] {
        unsafe {
            let len = origin.offset_from_unsigned(self.ptr);
            self.peek_slice_unchecked(len)
        }
    }
}
