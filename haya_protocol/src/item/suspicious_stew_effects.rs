use haya_collection::List;
use minecraft_data::mob_effect;

#[derive(Clone, Serialize, Deserialize)]
pub struct SuspiciousStewEffects<'a> {
    pub effects: List<'a, Entry>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Entry {
    pub effect: mob_effect,
    #[mser(varint)]
    pub duration: u32,
}
