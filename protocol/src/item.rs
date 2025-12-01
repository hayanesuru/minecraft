use crate::nbt::Compound;
use crate::Identifier;
use alloc::alloc::{Allocator, Global};

#[derive(Clone)]
pub struct ItemStack<A: Allocator = Global> {
    pub id: Identifier<A>,
    pub count: u32,
    pub components: Compound<A>,
}
