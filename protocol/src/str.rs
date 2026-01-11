use alloc::boxed::Box;
use alloc::vec::Vec;
use core::ops::{Deref, DerefMut};

#[repr(transparent)]
#[derive(Clone)]
pub struct BoxStr(Box<[u8]>);

impl BoxStr {
    pub fn new(bytes: Box<[u8]>) -> Option<Self> {
        match core::str::from_utf8(&bytes) {
            Ok(_) => Some(Self(bytes)),
            Err(_) => None,
        }
    }

    /// # Safety
    ///
    /// `bytes` must be valid UTF-8.
    pub unsafe fn new_unchecked(bytes: Box<[u8]>) -> Self {
        Self(bytes)
    }

    pub fn as_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.0) }
    }

    pub fn as_str_mut(&mut self) -> &mut str {
        unsafe { core::str::from_utf8_unchecked_mut(&mut self.0) }
    }

    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl BoxStr {
    pub fn empty() -> Self {
        unsafe { Self::new_unchecked(Vec::<u8>::new().into_boxed_slice()) }
    }
}

impl Deref for BoxStr {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        unsafe { core::str::from_utf8_unchecked(&self.0) }
    }
}

impl DerefMut for BoxStr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { core::str::from_utf8_unchecked_mut(&mut self.0) }
    }
}

impl AsRef<str> for BoxStr {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsMut<str> for BoxStr {
    fn as_mut(&mut self) -> &mut str {
        self.as_str_mut()
    }
}
