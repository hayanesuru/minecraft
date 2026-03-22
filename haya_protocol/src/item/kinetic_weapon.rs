use crate::Holder;
use crate::registry::SoundEventRef;
use crate::sound::SoundEvent;

#[derive(Clone, Serialize, Deserialize)]
pub struct KineticWeapon<'a> {
    #[mser(varint)]
    pub contact_cooldown_ticks: u32,
    #[mser(varint)]
    pub delay_ticks: u32,
    pub dismount_conditions: Option<Condition>,
    pub knockback_conditions: Option<Condition>,
    pub damage_conditions: Option<Condition>,
    pub forward_movement: f32,
    pub damage_multiplier: f32,
    pub sound: Option<Holder<SoundEvent<'a>, SoundEventRef>>,
    pub hit_sound: Option<Holder<SoundEvent<'a>, SoundEventRef>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Condition {
    #[mser(varint)]
    pub max_duration_ticks: u32,
    pub min_speed: f32,
    pub min_relative_speed: f32,
}
