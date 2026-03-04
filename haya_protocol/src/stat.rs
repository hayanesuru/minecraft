use crate::Registry;
use minecraft_data::stat_type;

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Stat {
    pub category: stat_type,
    pub registry: Registry,
}
