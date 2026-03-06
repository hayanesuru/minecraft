use minecraft_data::stat_type;
use mser::V32;

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Stat {
    pub category: stat_type,
    pub registry: V32,
}
