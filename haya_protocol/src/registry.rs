use crate::Holder;
use crate::block::BannerPattern;
use crate::entity::PaintingVariant;
use crate::item::{Instrument, JukeboxSong};
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

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct InstrumentRef(#[mser(varint)] pub u32);

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct SoundEventRef(pub sound_event);

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct JukeboxSongRef(#[mser(varint)] pub u32);

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct BannerPatternRef(#[mser(varint)] pub u32);

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct VillagerTypeRef(#[mser(varint)] pub u32);

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct WolfVariantRef(#[mser(varint)] pub u32);

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct WolfSoundVariantRef(#[mser(varint)] pub u32);

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct PigVariantRef(#[mser(varint)] pub u32);

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct CowVariantRef(#[mser(varint)] pub u32);

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct ChickenVariantRef(#[mser(varint)] pub u32);

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct ZombieNautilusVariantRef(#[mser(varint)] pub u32);

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct FrogVariantRef(#[mser(varint)] pub u32);

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct PaintingVariantRef(#[mser(varint)] pub u32);

impl<'a> Read<'a> for Holder<SoundEvent<'a>, SoundEventRef> {
    fn read(buf: &mut Reader<'a>) -> Result<Self, Error> {
        let id = V32::read(buf)?.0;
        if id == 0 {
            Ok(Self::Direct(SoundEvent::read(buf)?))
        } else {
            match match TryFrom::try_from(id - 1) {
                Ok(x) => sound_event::new(x),
                Err(_) => None,
            } {
                Some(x) => Ok(Self::Reference(SoundEventRef(x))),
                None => Err(Error),
            }
        }
    }
}

impl<'a> Write for Holder<SoundEvent<'a>, SoundEventRef> {
    unsafe fn write(&self, w: &mut Writer) {
        unsafe {
            match self {
                Self::Reference(id) => {
                    V32((id.0.id() as u32) + 1).write(w);
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
            Self::Reference(id) => V32((id.0.id() as u32) + 1).len_s(),
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

impl<'a> Read<'a> for Holder<Instrument<'a>, InstrumentRef> {
    fn read(buf: &mut Reader<'a>) -> Result<Self, Error> {
        let id = V32::read(buf)?.0;
        if id == 0 {
            Ok(Self::Direct(Instrument::read(buf)?))
        } else {
            let x = id - 1;
            Ok(Self::Reference(InstrumentRef(x)))
        }
    }
}

impl<'a> Write for Holder<Instrument<'a>, InstrumentRef> {
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

impl<'a> Read<'a> for Holder<JukeboxSong<'a>, JukeboxSongRef> {
    fn read(buf: &mut Reader<'a>) -> Result<Self, Error> {
        let id = V32::read(buf)?.0;
        if id == 0 {
            Ok(Self::Direct(JukeboxSong::read(buf)?))
        } else {
            let x = id - 1;
            Ok(Self::Reference(JukeboxSongRef(x)))
        }
    }
}

impl<'a> Write for Holder<JukeboxSong<'a>, JukeboxSongRef> {
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

impl<'a> Read<'a> for Holder<BannerPattern<'a>, BannerPatternRef> {
    fn read(buf: &mut Reader<'a>) -> Result<Self, Error> {
        let id = V32::read(buf)?.0;
        if id == 0 {
            Ok(Self::Direct(BannerPattern::read(buf)?))
        } else {
            let x = id - 1;
            Ok(Self::Reference(BannerPatternRef(x)))
        }
    }
}

impl<'a> Write for Holder<BannerPattern<'a>, BannerPatternRef> {
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

impl<'a> Read<'a> for Holder<PaintingVariant<'a>, PaintingVariantRef> {
    fn read(buf: &mut Reader<'a>) -> Result<Self, Error> {
        let id = V32::read(buf)?.0;
        if id == 0 {
            Ok(Self::Direct(PaintingVariant::read(buf)?))
        } else {
            let x = id - 1;
            Ok(Self::Reference(PaintingVariantRef(x)))
        }
    }
}

impl<'a> Write for Holder<PaintingVariant<'a>, PaintingVariantRef> {
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
