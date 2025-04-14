use crate::{Bytes, Read, UnsafeWriter, Write};
use core::ptr::NonNull;

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct MUTF8Tag(pub NonNull<[u8]>);

impl MUTF8Tag {
    /// # Safety
    ///
    /// The bytes passed in must be valid UTF-8.
    #[inline]
    pub const unsafe fn new_unchecked(n: &[u8]) -> Self {
        Self(NonNull::new_unchecked(n as *const [u8] as *mut [u8]))
    }
}

unsafe impl Write for MUTF8Tag {
    #[inline]
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        (self.0.len() as u16).write(w);
        w.write(unsafe { self.0.as_ref() });
    }

    #[inline]
    unsafe fn sz(&self) -> usize {
        2 + self.0.len()
    }
}

impl Read for MUTF8Tag {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        let len = buf.u16()?;
        let data = buf.slice(len as usize)?;
        Some(unsafe { Self::new_unchecked(data) })
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct UTF8Tag(pub NonNull<[u8]>);

impl UTF8Tag {
    /// # Safety
    ///
    /// The bytes passed in must be valid UTF-8.
    #[inline]
    pub const unsafe fn new_unchecked(n: &[u8]) -> Self {
        Self(NonNull::new_unchecked(n as *const _ as _))
    }
}

unsafe impl Write for UTF8Tag {
    #[inline]
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        let x = unsafe { self.0.as_ref() };
        if super::mutf8::is_mutf8(x) {
            MUTF8Tag(self.0).write(w);
        } else {
            unsafe {
                (super::mutf8::len_mutf8(x) as u16).write(w);
                super::mutf8::encode_mutf8(x, w);
            }
        }
    }

    #[inline]
    unsafe fn sz(&self) -> usize {
        unsafe { 2 + super::mutf8::len_mutf8(self.0.as_ref()) }
    }
}
