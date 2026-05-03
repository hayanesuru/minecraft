use crate::registry::EnchntmentRef;
use haya_collection::Map;

#[derive(Clone, Serialize, Deserialize)]
pub struct ItemEnchantments<'a>(pub Map<'a, EnchntmentRef, Level>);

#[derive(Clone, Serialize, Deserialize)]
pub struct Level(#[mser(varint, filter = validate_enchantment_level)] pub i32);

fn validate_enchantment_level(level: &i32) -> bool {
    (0..=255).contains(level)
}
