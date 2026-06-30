use haya_collection::List;
use haya_ident::Ident;
use mser::{Either, Read, Utf8, Write};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize)]
pub struct GameProfileRef<'a> {
    pub id: Uuid,
    pub name: Utf8<'a, 16>,
    pub properties: PropertyMapRef<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PropertyMapRef<'a>(pub List<'a, PropertyRef<'a>, 16>);

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct PropertyRef<'a> {
    pub name: Utf8<'a, 64>,
    pub value: Utf8<'a, 32767>,
    pub signature: Option<Utf8<'a, 1024>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ResolvableProfileRef<'a> {
    pub profile: Either<GameProfileRef<'a>, PartialRef<'a>>,
    pub skin_patch: PlayerSkinPatchRef<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PartialRef<'a> {
    pub name: Option<Utf8<'a, 16>>,
    pub id: Option<Uuid>,
    pub properties: List<'a, PropertyRef<'a>, 16>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PlayerSkinPatchRef<'a> {
    pub body: Option<Ident<'a>>,
    pub cape: Option<Ident<'a>>,
    pub elytra: Option<Ident<'a>>,
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
