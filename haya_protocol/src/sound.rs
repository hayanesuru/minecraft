use haya_ident::Ident;

#[derive(Clone, Serialize, Deserialize)]
pub struct SoundEvent<'a> {
    pub location: Ident<'a>,
    pub fixed_range: Option<f32>,
}

#[derive(Clone, Copy, Serialize, Deserialize, Name)]
#[repr(u8)]
#[mser(varint, camel_case)]
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
