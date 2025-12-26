use super::*;
use crate::nbt::{Kv, TagType};
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
    pub unsafe fn write_ty(&self, w: &mut UnsafeWriter) {
        unsafe {
            if let Some(ref name) = self.name {
                Kv(NAME, name).write(w);
            }
            if let Some(id) = self.id {
                Kv(ID, id).write(w);
            }
            if !self.properties.is_empty() {
                Kv(PROPERTIES, &*self.properties).write(w);
            }
            TagType::End.write(w);
        }
    }

    pub fn ty_sz(&self) -> usize {
        let mut w = 0;
        if let Some(ref name) = self.name {
            w += Kv(NAME, name).sz();
        }
        if let Some(id) = self.id {
            w += Kv(ID, id).sz();
        }
        if !self.properties.is_empty() {
            w += Kv(PROPERTIES, &*self.properties).sz();
        }

        w += TagType::End.sz();
        w
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

    pub fn ty_sz(&self) -> usize {
        0
    }
}

impl<'a> GameProfile {
    pub fn read_ty(buf: &mut &'a [u8]) -> Result<Self, Error> {
        Err(Error)
    }
}
