use crate::str::BoxStr;
use alloc::alloc::{Allocator, Global};
use alloc::vec::Vec;
use uuid::Uuid;

#[derive(Clone)]
pub struct Profile<A: Allocator = Global> {
    pub name: Option<BoxStr<A>>,
    pub id: Option<Uuid>,
    pub texture: Option<BoxStr<A>>,
    pub cape: Option<BoxStr<A>>,
    pub model: Option<BoxStr<A>>,
    pub profile_properties: Option<Vec<Property<A>, A>>,
}

#[derive(Clone)]
pub struct Property<A: Allocator = Global> {
    pub name: BoxStr<A>,
    pub value: BoxStr<A>,
    pub signature: Option<BoxStr<A>>,
}
