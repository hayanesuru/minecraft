use haya_collection::Cow;
use minecraft_data::mob_effect;

#[derive(Clone, Serialize, Deserialize)]
pub struct MobEffect<'a> {
    effect: mob_effect,
    details: MobEffectDetails<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MobEffectDetails<'a> {
    #[mser(varint)]
    amplifier: u32,
    #[mser(varint)]
    duration: u32,
    ambient: bool,
    show_particles: bool,
    show_icon: bool,
    hidden_effect: Option<Cow<'a, Self>>,
}
