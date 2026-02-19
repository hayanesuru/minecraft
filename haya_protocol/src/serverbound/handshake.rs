use crate::{ClientIntent, Utf8};

#[derive(Clone, Serialize, Deserialize)]
pub struct Intention<'a> {
    pub protocol_version: Utf8<'a>,
    pub host_name: Utf8<'a>, // bungeecord
    pub port: u16,
    pub intention: ClientIntent,
}
