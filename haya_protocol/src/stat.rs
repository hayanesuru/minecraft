use minecraft_data::{block, custom_stat, entity_type, item, stat_type};

#[derive(Clone, Copy, Deserialize, Serialize)]
#[mser(header = stat_type)]
pub enum Stat {
    Mined(block),
    Crafted(item),
    Used(item),
    Broken(item),
    PickedUp(item),
    Dropped(item),
    Killed(entity_type),
    KilledBy(entity_type),
    Custom(custom_stat),
}
