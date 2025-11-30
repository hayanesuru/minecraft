use alloc::alloc::{Allocator, Global};
use alloc::vec::Vec;
use mser::SmolStr;
use uuid::Uuid;

#[derive(Clone)]
pub struct Profile<A: Allocator = Global> {
    pub name: Option<SmolStr>,
    pub id: Option<Uuid>,
    pub texture: Option<SmolStr>,
    pub cape: Option<SmolStr>,
    pub model: Option<SmolStr>,
    pub profile_properties: Option<Vec<Property, A>>,
}

#[derive(Clone)]
pub struct Property {
    pub name: SmolStr,
    pub value: SmolStr,
    pub signature: Option<SmolStr>,
}
