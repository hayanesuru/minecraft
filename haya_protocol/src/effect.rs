use haya_collection::Cow;
use minecraft_data::mob_effect;

#[derive(Clone, Serialize, Deserialize)]
pub struct MobEffect<'a> {
    pub effect: mob_effect,
    pub details: MobEffectDetails<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MobEffectDetails<'a> {
    #[mser(varint)]
    pub amplifier: u32,
    #[mser(varint)]
    pub duration: u32,
    pub ambient: bool,
    pub show_particles: bool,
    pub show_icon: bool,
    pub hidden_effect: Option<Cow<'a, Self>>,
}
