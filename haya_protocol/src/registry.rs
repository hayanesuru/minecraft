use crate::block::BannerPattern;
use crate::chat::ChatType;
use crate::entity::PaintingVariant;
use crate::item_stack::{Instrument, JukeboxSong};
use crate::sound::SoundEvent;
use crate::trim::{TrimMaterial, TrimPattern};
use crate::{DialogRaw, Holder};
use minecraft_data::sound_event;
use mser::{Error, Read, Reader, V32, Write, Writer};

macro_rules! decl {
    ($($i:ident),*) => {
        $(
            #[derive(Clone, Copy, Serialize, Deserialize)]
            pub struct $i(#[mser(varint)] pub u32);
        )*
    };
}

decl![
    DamageTypeRef,
    TrimMaterialRef,
    EnchntmentRef,
    TrimPatternRef,
    InstrumentRef,
    JukeboxSongRef,
    BannerPatternRef,
    VillagerTypeRef,
    WolfVariantRef,
    WolfSoundVariantRef,
    PigVariantRef,
    PigSoundVariantRef,
    CowVariantRef,
    CowSoundVariantRef,
    ChickenVariantRef,
    ChickenSoundVariantRef,
    ZombieNautilusVariantRef,
    FrogVariantRef,
    PaintingVariantRef,
    CatVariantRef,
    CatSoundVariantRef,
    ChatTypeRef,
    DimensionTypeRef,
    DialogRef,
    WorldClockRef
];

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

impl<'a> Read<'a> for Holder<ChatType<'a>, ChatTypeRef> {
    fn read(buf: &mut Reader<'a>) -> Result<Self, Error> {
        let id = V32::read(buf)?.0;
        if id == 0 {
            Ok(Self::Direct(ChatType::read(buf)?))
        } else {
            let x = id - 1;
            Ok(Self::Reference(ChatTypeRef(x)))
        }
    }
}

impl<'a> Write for Holder<ChatType<'a>, ChatTypeRef> {
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

impl<'a> Read<'a> for Holder<DialogRaw, DialogRef> {
    fn read(buf: &mut Reader<'a>) -> Result<Self, Error> {
        let id = V32::read(buf)?.0;
        if id == 0 {
            Ok(Self::Direct(DialogRaw::read(buf)?))
        } else {
            let x = id - 1;
            Ok(Self::Reference(DialogRef(x)))
        }
    }
}

impl Write for Holder<DialogRaw, DialogRef> {
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
