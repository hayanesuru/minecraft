use haya_ident::Ident;

#[derive(Clone, Serialize, Deserialize)]
pub struct SoundEvent<'a> {
    pub location: Ident<'a>,
    pub fixed_range: Option<f32>,
}
