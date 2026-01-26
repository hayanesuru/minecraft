use crate::nbt::Compound;
use minecraft_data::item;

#[derive(Clone)]
pub struct ItemStack {
    pub id: item,
    pub count: u32,
    pub components: Compound,
}
