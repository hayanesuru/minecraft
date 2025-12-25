use super::*;
use mser::{Error, Read, UnsafeWriter, Write};

impl<A: Allocator> Write for GameProfile<A> {
    unsafe fn write(&self, w: &mut UnsafeWriter) {}

    fn sz(&self) -> usize {
        0
    }
}

impl<'a> Read<'a> for GameProfile {
    fn read(buf: &mut &'a [u8]) -> Result<Self, Error> {
        Err(Error)
    }
}

impl<A: Allocator> Write for ResolvableProfile<A> {
    unsafe fn write(&self, w: &mut UnsafeWriter) {}

    fn sz(&self) -> usize {
        0
    }
}

impl<'a> Read<'a> for ResolvableProfile {
    fn read(buf: &mut &'a [u8]) -> Result<Self, Error> {
        Err(Error)
    }
}

impl<A: Allocator> ResolvableProfile<A> {
    /// # Safety
    pub unsafe fn write_ty(&self, w: &mut UnsafeWriter) {}

    pub fn write_ty_sz(&self) -> usize {
        0
    }
}

impl<'a> ResolvableProfile {
    pub fn read_ty(buf: &mut &'a [u8]) -> Result<Self, Error> {
        Err(Error)
    }
}

impl<A: Allocator> GameProfile<A> {
    /// # Safety
    pub unsafe fn write_ty(&self, w: &mut UnsafeWriter) {}

    pub fn write_ty_sz(&self) -> usize {
        0
    }
}

impl<'a> GameProfile {
    pub fn read_ty(buf: &mut &'a [u8]) -> Result<Self, Error> {
        Err(Error)
    }
}
