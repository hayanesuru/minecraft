use crate::Holder;
use crate::sound::SoundEvent;
use minecraft_data::sound_event;

#[derive(Clone, Serialize, Deserialize)]
pub struct Consumable<'a> {
    pub consume_seconds: f32,
    pub animation: ItemUseAnimation,
    pub sound: Holder<SoundEvent<'a>, sound_event>,
}

#[derive(Clone, Copy, Serialize, Deserialize, Name)]
#[repr(u8)]
#[mser(varint)]
pub enum ItemUseAnimation {
    None,
    Eat,
    Drink,
    Block,
    Bow,
    Trident,
    Crossbow,
    Spyglass,
    TootHorn,
    Brush,
    Bundle,
    Spear,
}
