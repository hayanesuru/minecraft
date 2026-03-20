use crate::Holder;
use crate::sound::SoundEvent;
use crate::trim::{TrimMaterial, TrimPattern};
use minecraft_data::sound_event;
use mser::{Error, Read, Reader, V32, Write, Writer};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct DamageTypeRef(#[mser(varint)] pub u32);

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct TrimMaterialRef(#[mser(varint)] pub u32);

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct EnchntmentRef(#[mser(varint)] pub u32);

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct TrimPatternRef(#[mser(varint)] pub u32);

impl<'a> Read<'a> for Holder<SoundEvent<'a>, sound_event> {
    fn read(buf: &mut Reader<'a>) -> Result<Self, Error> {
        let id = V32::read(buf)?.0;
        if id == 0 {
            Ok(Self::Direct(SoundEvent::read(buf)?))
        } else {
            match match TryFrom::try_from(id - 1) {
                Ok(x) => sound_event::new(x),
                Err(_) => None,
            } {
                Some(x) => Ok(Self::Reference(x)),
                None => Err(Error),
            }
        }
    }
}

impl<'a> Write for Holder<SoundEvent<'a>, sound_event> {
    unsafe fn write(&self, w: &mut Writer) {
        unsafe {
            match self {
                Self::Reference(id) => {
                    V32((id.id() as u32) + 1).write(w);
                }
                Self::Direct(direct) => {
                    V32(0).write(w);
                    direct.write(w);
                }
            }
        }
    }

    fn len_s(&self) -> usize {
        match self {
            Self::Reference(id) => V32((id.id() as u32) + 1).len_s(),
            Self::Direct(direct) => {
                let mut len = V32(0).len_s();
                len += direct.len_s();
                len
            }
        }
    }
}

impl<'a> Read<'a> for Holder<TrimMaterial<'a>, TrimMaterialRef> {
    fn read(buf: &mut Reader<'a>) -> Result<Self, Error> {
        let id = V32::read(buf)?.0;
        if id == 0 {
            Ok(Self::Direct(TrimMaterial::read(buf)?))
        } else {
            let x = id - 1;
            Ok(Self::Reference(TrimMaterialRef(x)))
        }
    }
}

impl<'a> Write for Holder<TrimMaterial<'a>, TrimMaterialRef> {
    unsafe fn write(&self, w: &mut Writer) {
        unsafe {
            match self {
                Self::Reference(id) => {
                    V32(id.0 + 1).write(w);
                }
                Self::Direct(direct) => {
                    V32(0).write(w);
                    direct.write(w);
                }
            }
        }
    }

    fn len_s(&self) -> usize {
        match self {
            Self::Reference(id) => V32(id.0 + 1).len_s(),
            Self::Direct(direct) => {
                let mut len = V32(0).len_s();
                len += direct.len_s();
                len
            }
        }
    }
}

impl<'a> Read<'a> for Holder<TrimPattern<'a>, TrimPatternRef> {
    fn read(buf: &mut Reader<'a>) -> Result<Self, Error> {
        let id = V32::read(buf)?.0;
        if id == 0 {
            Ok(Self::Direct(TrimPattern::read(buf)?))
        } else {
            let x = id - 1;
            Ok(Self::Reference(TrimPatternRef(x)))
        }
    }
}

impl<'a> Write for Holder<TrimPattern<'a>, TrimPatternRef> {
    unsafe fn write(&self, w: &mut Writer) {
        unsafe {
            match self {
                Self::Reference(id) => {
                    V32(id.0 + 1).write(w);
                }
                Self::Direct(direct) => {
                    V32(0).write(w);
                    direct.write(w);
                }
            }
        }
    }

    fn len_s(&self) -> usize {
        match self {
            Self::Reference(id) => V32(id.0 + 1).len_s(),
            Self::Direct(direct) => {
                let mut len = V32(0).len_s();
                len += direct.len_s();
                len
            }
        }
    }
}
