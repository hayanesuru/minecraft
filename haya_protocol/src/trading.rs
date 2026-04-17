use crate::item::{DataComponentExactPredicate, ItemStack};
use minecraft_data::item;

#[derive(Clone, Serialize, Deserialize)]
pub struct MerchantOffer<'a> {
    pub base_cost_a: ItemCost<'a>,
    pub result: ItemStack<'a>,
    pub cost_b: Option<ItemCost<'a>>,
    pub out_of_stock: bool,
    pub uses: u32,
    pub max_uses: u32,
    pub xp: u32,
    pub special_price_diff: u32,
    pub price_multiplier: f32,
    pub demand: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ItemCost<'a> {
    pub item: item,
    #[mser(varint)]
    pub count: u32,
    pub components: DataComponentExactPredicate<'a>,
}
