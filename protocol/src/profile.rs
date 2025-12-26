mod binrary;

use crate::str::BoxStr;
use alloc::alloc::{Allocator, Global};
use alloc::vec::Vec;
use uuid::Uuid;

const NAME: &[u8] = b"name";
const ID: &[u8] = b"ID";

#[derive(Clone)]
pub struct ResolvableProfile<A: Allocator = Global> {
    pub name: Option<BoxStr<A>>,
    pub id: Option<Uuid>,
    pub properties: Vec<Property<A>, A>,
    pub patch: ProfilePatch<A>,
}

#[derive(Clone)]
pub struct GameProfile<A: Allocator = Global> {
    pub name: BoxStr<A>,
    pub id: Uuid,
    pub properties: Vec<Property<A>, A>,
    pub patch: ProfilePatch<A>,
}

#[derive(Clone)]
pub struct Property<A: Allocator = Global> {
    pub name: BoxStr<A>,
    pub value: BoxStr<A>,
    pub signature: Option<BoxStr<A>>,
}

#[derive(Clone)]
pub struct ProfilePatch<A: Allocator = Global> {
    pub texture: Option<BoxStr<A>>,
    pub cape: Option<BoxStr<A>>,
    pub elytra: Option<BoxStr<A>>,
    pub model: Option<BoxStr<A>>,
}
