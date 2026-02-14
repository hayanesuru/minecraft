use super::*;
use crate::nbt::{End, Kv, MapCodec, MapReader, TagType};
use mser::{Error, UnsafeWriter, Write};

impl MapCodec for GameProfile {
    fn read_kv(buf: &mut &[u8]) -> Result<Self, Error> {
        let ResolvableProfile {
            name,
            id,
            properties,
            patch,
        } = ResolvableProfile::read_kv(buf)?;
        if let Some(name) = name
            && let Some(id) = id
        {
            Ok(Self {
                name,
                id,
                properties,
                patch,
            })
        } else {
            Err(Error)
        }
    }

    unsafe fn write_kv(&self, w: &mut UnsafeWriter) {
        unsafe {
            Kv(NAME, &self.name).write(w);
            Kv(ID, self.id).write(w);
            if !self.properties.is_empty() {
                Kv(PROPERTIES, &self.properties).write(w);
            }
            let PlayerSkin {
                texture,
                cape,
                elytra,
                model,
            } = &self.patch;
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
        let PlayerSkin {
            texture,
            cape,
            elytra,
            model,
        } = &self.patch;
        w + End.len_s()
    }
}

impl MapReader for ResolvableProfile {
    fn visit(&mut self, ty: TagType, k: &[u8], buf: &mut &[u8]) -> Result<(), Error> {
        match k {
            NAME => {
                self.name = Some(ty.string(buf)?);
            }
            ID => {
                if let [a, b, c, d] = ty.int_list(buf)?[..] {
                    let h = (a as u32 as u64) << 32 | (b as u32 as u64);
                    let l = (c as u32 as u64) << 32 | (d as u32 as u64);
                    self.id = Some(Uuid::from_u64_pair(h, l));
                }
            }
            PROPERTIES => {
                self.properties = PropertyMap::read_kv(buf)?;
            }
            _ => return Err(Error),
        }
        Ok(())
    }

    fn end(self) -> Result<Self, Error> {
        Ok(self)
    }
}

impl MapCodec for ResolvableProfile {
    fn read_kv(buf: &mut &[u8]) -> Result<Self, Error> {
        Self {
            name: None,
            id: None,
            properties: PropertyMap(Vec::new()),
            patch: PlayerSkin {
                texture: None,
                cape: None,
                elytra: None,
                model: None,
            },
        }
        .read_map(buf)
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
            let PlayerSkin {
                texture,
                cape,
                elytra,
                model,
            } = &self.patch;
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
        let PlayerSkin {
            texture,
            cape,
            elytra,
            model,
        } = &self.patch;
        w + End.len_s()
    }
}

impl MapCodec for PropertyMap {
    fn read_kv(buf: &mut &[u8]) -> Result<Self, mser::Error> {
        todo!()
    }

    unsafe fn write_kv(&self, w: &mut mser::UnsafeWriter) {
        unsafe {
            for p in &self.0 {
                Kv(PROPERTY_NAME, &p.name).write(w);
                Kv(PROPERTY_VALUE, &p.value).write(w);
                if let Some(ref signature) = p.signature {
                    Kv(PROPERTY_SIGNATURE, signature).write(w);
                }
            }
            End.write(w);
        }
    }

    fn len_kv(&self) -> usize {
        let mut w = 0;
        for p in &self.0 {
            w += Kv(PROPERTY_NAME, &p.name).len_s();
            w += Kv(PROPERTY_VALUE, &p.value).len_s();
            if let Some(ref signature) = p.signature {
                w += Kv(PROPERTY_SIGNATURE, signature).len_s();
            }
        }
        w + End.len_s()
    }
}
