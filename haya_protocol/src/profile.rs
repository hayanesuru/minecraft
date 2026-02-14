//mod nbt;

use crate::{List, Utf8};
use alloc::boxed::Box;
use alloc::vec::Vec;
use uuid::Uuid;

// const NAME: &[u8] = b"name";
// const ID: &[u8] = b"id";
// const PROPERTIES: &[u8] = b"properties";
// const PROPERTY_NAME: &[u8] = b"name";
// const PROPERTY_VALUE: &[u8] = b"value";
// const PROPERTY_SIGNATURE: &[u8] = b"signature";

#[derive(Clone, Serialize, Deserialize)]
pub struct GameProfileRef<'a> {
    pub id: Uuid,
    pub name: Utf8<'a, 16>,
    pub peoperties: List<'a, PropertyRef<'a>, 16>,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct PropertyRef<'a> {
    pub name: Utf8<'a, 64>,
    pub value: Utf8<'a, 32767>,
    pub signature: Option<Utf8<'a, 1024>>,
}

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

impl PropertyMap {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
