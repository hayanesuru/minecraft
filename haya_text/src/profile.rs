use alloc::boxed::Box;
use alloc::vec::Vec;
use uuid::Uuid;

#[derive(Clone)]
pub struct ResolvableProfile {
    pub name: Option<Box<str>>,
    pub id: Option<Uuid>,
    pub properties: PropertyMap,
    pub patch: PlayerSkin,
}

#[derive(Clone)]
pub struct GameProfile {
    pub name: Box<str>,
    pub id: Uuid,
    pub properties: PropertyMap,
    pub patch: PlayerSkin,
}

#[derive(Clone)]
pub struct PropertyMap(pub Vec<Property>);

#[derive(Clone)]
pub struct Property {
    pub name: Box<str>,
    pub value: Box<str>,
    pub signature: Option<Box<str>>,
}

#[derive(Clone)]
pub struct PlayerSkin {
    pub texture: Option<Box<str>>,
    pub cape: Option<Box<str>>,
    pub elytra: Option<Box<str>>,
    pub model: Option<PlayerModelType>,
}

#[derive(Clone, Copy)]
pub enum PlayerModelType {
    Slim,
    Wide,
}
