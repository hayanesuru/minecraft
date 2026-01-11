use super::*;
use crate::nbt::{End, Kv, MapCodec};
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
                Kv(PROPERTIES, &self.properties).write(w);
            }
            End.write(w);
        }
    }

    fn len_kv(&self) -> usize {
        let mut w = 0;
        w += Kv(NAME, &self.name).len_s();
        w += Kv(ID, self.id).len_s();
        if !self.properties.is_empty() {
            w += Kv(PROPERTIES, &self.properties).len_s();
        }
        w + End.len_s()
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
                Kv(PROPERTIES, &self.properties).write(w);
            }
            End.write(w);
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
            w += Kv(PROPERTIES, &self.properties).len_s();
        }
        w + End.len_s()
    }
}

impl PropertyMap {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
impl MapCodec for PropertyMap {
    fn read_kv(buf: &mut &[u8]) -> Result<Self, mser::Error> {
        todo!()
    }

    unsafe fn write_kv(&self, w: &mut mser::UnsafeWriter) {
        unsafe {
            for p in &self.0 {
                Kv(b"name", &p.name).write(w);
                Kv(b"value", &p.value).write(w);
                if let Some(ref signature) = p.signature {
                    Kv(b"signature", signature).write(w);
                }
            }
            End.write(w);
        }
    }

    fn len_kv(&self) -> usize {
        let mut w = 0;
        for p in &self.0 {
            w += Kv(b"name", &p.name).len_s();
            w += Kv(b"value", &p.value).len_s();
            if let Some(ref signature) = p.signature {
                w += Kv(b"signature", signature).len_s();
            }
        }
        w + End.len_s()
    }
}
