use haya_collection::BoxCodec;
use minecraft_data::mob_effect;

#[derive(Clone, Serialize, Deserialize)]
pub struct MobEffect {
    effect: mob_effect,
    details: MobEffectDetails,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MobEffectDetails {
    #[mser(varint)]
    amplifier: u32,
    #[mser(varint)]
    duration: u32,
    ambient: bool,
    show_particles: bool,
    show_icon: bool,
    hidden_effect: Option<BoxCodec<Self>>,
}
