use crate::str::SmolStr;
use alloc::alloc::{Allocator, Global};
use alloc::vec::Vec;
use uuid::Uuid;

#[derive(Clone)]
pub struct Profile<A: Allocator = Global> {
    pub name: Option<SmolStr<A>>,
    pub id: Option<Uuid>,
    pub texture: Option<SmolStr<A>>,
    pub cape: Option<SmolStr<A>>,
    pub model: Option<SmolStr<A>>,
    pub profile_properties: Option<Vec<Property<A>, A>>,
}

#[derive(Clone)]
pub struct Property<A: Allocator = Global> {
    pub name: SmolStr<A>,
    pub value: SmolStr<A>,
    pub signature: Option<SmolStr<A>>,
}
