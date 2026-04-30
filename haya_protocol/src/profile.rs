use crate::ResourceTexture;
use haya_collection::{List, Map};
use mser::{Either, Read, Utf8, Write};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize)]
pub struct GameProfileRef<'a> {
    pub id: Uuid,
    pub name: Utf8<'a, 16>,
    pub properties: PropertyMap<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PropertyMap<'a>(pub Map<'a, Utf8<'a, 64>, PropertyRef<'a>, 16>);

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct PropertyRef<'a> {
    pub value: Utf8<'a, 32767>,
    pub signature: Option<Utf8<'a, 1024>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ResolvableProfile<'a> {
    pub profile: Either<GameProfileRef<'a>, Partial<'a>>,
    pub skin_patch: PlayerSkinPatch<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Partial<'a> {
    pub name: Option<Utf8<'a, 16>>,
    pub id: Option<Uuid>,
    pub properties: List<'a, PropertyRef<'a>, 16>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PlayerSkinPatch<'a> {
    pub body: Option<ResourceTexture<'a>>,
    pub cape: Option<ResourceTexture<'a>>,
    pub elytra: Option<ResourceTexture<'a>>,
    pub model: Option<PlayerModelType>,
}

#[derive(Clone, Copy)]
pub enum PlayerModelType {
    Slim,
    Wide,
}

impl<'a> Read<'a> for PlayerModelType {
    fn read(buf: &mut mser::Reader<'a>) -> Result<Self, mser::Error> {
        match bool::read(buf) {
            Ok(true) => Ok(Self::Slim),
            Ok(false) => Ok(Self::Wide),
            Err(e) => Err(e),
        }
    }
}

impl Write for PlayerModelType {
    unsafe fn write(&self, w: &mut mser::Writer) {
        unsafe {
            match self {
                Self::Slim => true,
                Self::Wide => false,
            }
            .write(w);
        }
    }

    fn len_s(&self) -> usize {
        match self {
            Self::Slim => true,
            Self::Wide => false,
        }
        .len_s()
    }
}
