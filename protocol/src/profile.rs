mod nbt;

use crate::str::BoxStr;
use alloc::vec::Vec;
use uuid::Uuid;

const NAME: &[u8] = b"name";
const ID: &[u8] = b"id";
const PROPERTIES: &[u8] = b"properties";

#[derive(Clone)]
pub struct ResolvableProfile {
    pub name: Option<BoxStr>,
    pub id: Option<Uuid>,
    pub properties: PropertyMap,
    pub patch: ProfilePatch,
}

#[derive(Clone)]
pub struct GameProfile {
    pub name: BoxStr,
    pub id: Uuid,
    pub properties: PropertyMap,
    pub patch: ProfilePatch,
}

#[derive(Clone)]
pub struct PropertyMap(pub Vec<Property>);

#[derive(Clone)]
pub struct Property {
    pub name: BoxStr,
    pub value: BoxStr,
    pub signature: Option<BoxStr>,
}

#[derive(Clone)]
pub struct ProfilePatch {
    pub texture: Option<BoxStr>,
    pub cape: Option<BoxStr>,
    pub elytra: Option<BoxStr>,
    pub model: Option<BoxStr>,
}
