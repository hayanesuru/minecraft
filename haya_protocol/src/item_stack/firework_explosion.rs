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

#[derive(Clone, Copy, Serialize, Deserialize)]
#[mser(varint)]
#[repr(u8)]
pub enum Shape {
    SmallBall,
    LargeBall,
    Star,
    Creeper,
    Burst,
}

impl Shape {
    pub const fn name(self) -> &'static str {
        match self {
            Self::SmallBall => "small_ball",
            Self::LargeBall => "large_ball",
            Self::Star => "star",
            Self::Creeper => "creeper",
            Self::Burst => "burst",
        }
    }

    pub const fn translation_key(self) -> Translatable<'static> {
        Translatable(match self {
            Self::SmallBall => "item.minecraft.firework_star.shape.small_ball",
            Self::LargeBall => "item.minecraft.firework_star.shape.large_ball",
            Self::Star => "item.minecraft.firework_star.shape.star",
            Self::Creeper => "item.minecraft.firework_star.shape.creeper",
            Self::Burst => "item.minecraft.firework_star.shape.burst",
        })
    }
}
