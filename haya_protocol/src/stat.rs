use minecraft_data::stat_type;

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Stat {
    pub category: stat_type,
    #[mser(varint)]
    pub registry: u32,
}
