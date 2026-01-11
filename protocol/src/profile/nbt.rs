use super::*;
use crate::nbt::{Kv, MapCodec, TagType};
use mser::{Error, UnsafeWriter, Write};

impl MapCodec for GameProfile {
    fn read_kv(buf: &mut &[u8]) -> Result<Self, Error> {
        Err(Error)
    }

    unsafe fn write_kv(&self, w: &mut UnsafeWriter) {
        unsafe {
            Kv(NAME, &self.name).write(w);
            Kv(ID, self.id).write(w);
            if !self.properties.is_empty() {
                Kv(PROPERTIES, &*self.properties).write(w);
            }
            TagType::End.write(w);
        }
    }

    fn len_kv(&self) -> usize {
        let mut w = 0;
        w += Kv(NAME, &self.name).len_s();
        w += Kv(ID, self.id).len_s();
        if !self.properties.is_empty() {
            w += Kv(PROPERTIES, &*self.properties).len_s();
        }
        w + TagType::End.len_s()
    }
}

impl MapCodec for ResolvableProfile {
    fn read_kv(buf: &mut &[u8]) -> Result<Self, Error> {
        Err(Error)
    }

    unsafe fn write_kv(&self, w: &mut UnsafeWriter) {
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

    fn len_kv(&self) -> usize {
        let mut w = 0;
        if let Some(ref name) = self.name {
            w += Kv(NAME, name).len_s();
        }
        if let Some(id) = self.id {
            w += Kv(ID, id).len_s();
        }
        if !self.properties.is_empty() {
            w += Kv(PROPERTIES, &*self.properties).len_s();
        }
        w + TagType::End.len_s()
    }
}
