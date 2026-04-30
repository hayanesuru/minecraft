use haya_ident::Ident;

#[derive(Clone, Serialize, Deserialize)]
pub struct SoundEvent<'a> {
    pub location: Ident<'a>,
    pub fixed_range: Option<f32>,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum SoundSource {
    Master,
    Music,
    Record,
    Weather,
    Block,
    Hostile,
    Neutral,
    Player,
    Ambient,
    Voice,
    Ui,
}

impl SoundSource {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Master => "master",
            Self::Music => "music",
            Self::Record => "record",
            Self::Weather => "weather",
            Self::Block => "block",
            Self::Hostile => "hostile",
            Self::Neutral => "neutral",
            Self::Player => "player",
            Self::Ambient => "ambient",
            Self::Voice => "voice",
            Self::Ui => "ui",
        }
    }
}
