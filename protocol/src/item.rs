use crate::nbt::Compound;
use alloc::alloc::{Allocator, Global};
use minecraft_data::item;

#[derive(Clone)]
pub struct ItemStack<A: Allocator = Global> {
    pub id: item,
    pub count: u32,
    pub components: Compound<A>,
}
