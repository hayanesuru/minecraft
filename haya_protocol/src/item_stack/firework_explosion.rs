use crate::Translatable;
use haya_collection::List;

#[derive(Clone, Serialize, Deserialize)]
pub struct FireworkExplosion<'a> {
    pub shape: Shape,
    pub colors: List<'a, u32>,
    pub fade_colors: List<'a, u32>,
    pub has_trail: bool,
    pub has_twinkle: bool,
}

#[derive(Clone, Copy, Serialize, Deserialize, Name)]
#[repr(u8)]
#[mser(varint, camel_case)]
pub enum Shape {
    SmallBall,
    LargeBall,
    Star,
    Creeper,
    Burst,
}

impl Shape {
    pub const fn translation_key(self) -> Translatable<'static> {
        Translatable("item.minecraft.firework_star.shape.", self.name())
    }
}
