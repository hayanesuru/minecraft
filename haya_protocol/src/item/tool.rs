use crate::HolderSet;
use haya_collection::List;
use minecraft_data::block;

#[derive(Clone, Serialize, Deserialize)]
pub struct Rule<'a> {
    pub blocks: HolderSet<'a, block>,
    pub speed: Option<f32>,
    pub correct_for_drops: Option<bool>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Tool<'a> {
    pub rules: List<'a, Rule<'a>>,
    pub default_mining_speed: f32,
    #[mser(varint)]
    pub damage_per_block: u32,
    pub can_destroy_blocks_in_creative: bool,
}
