use alloc::alloc::{Allocator, Global};
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::ops::{Deref, DerefMut};

#[repr(transparent)]
#[derive(Clone)]
pub struct BoxStr<A: Allocator = Global>(Box<[u8], A>);

impl<A: Allocator> BoxStr<A> {
    pub fn new(bytes: Box<[u8], A>) -> Option<Self> {
        match core::str::from_utf8(&bytes) {
            Ok(_) => Some(Self(bytes)),
            Err(_) => None,
        }
    }

    /// # Safety
    ///
    /// `bytes` must be valid UTF-8.
    pub unsafe fn new_unchecked(bytes: Box<[u8], A>) -> Self {
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

impl<A: Allocator> Deref for BoxStr<A> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        unsafe { core::str::from_utf8_unchecked(&self.0) }
    }
}

impl<A: Allocator> DerefMut for BoxStr<A> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { core::str::from_utf8_unchecked_mut(&mut self.0) }
    }
}

impl<A: Allocator + Default> Default for BoxStr<A> {
    fn default() -> Self {
        unsafe { Self::new_unchecked(Vec::<u8, A>::new_in(A::default()).into_boxed_slice()) }
    }
}

impl<A: Allocator> AsRef<str> for BoxStr<A> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<A: Allocator> AsMut<str> for BoxStr<A> {
    fn as_mut(&mut self) -> &mut str {
        self.as_str_mut()
    }
}
