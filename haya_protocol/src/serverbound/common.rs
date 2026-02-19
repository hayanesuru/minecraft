use crate::{ClientInformation, Rest};
use haya_ident::Ident;
use haya_nbt::Tag;
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize)]
pub struct ConfigurationClientInformation<'a>(pub ClientInformation<'a>);

#[derive(Clone, Serialize, Deserialize)]
pub struct CustomPayload<'a> {
    pub id: Ident<'a>,
    pub payload: Rest<'a, 32767>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct KeepAlive {
    pub id: u64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Pong {
    pub id: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ResourcePack {
    pub id: Uuid,
    pub action: ResourcePackAction,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
pub enum ResourcePackAction {
    SuccessfullyLoaded,
    Declined,
    FailedDownload,
    Accepted,
    Downloaded,
    InvalidUrl,
    FailedReload,
    Discarded,
}

impl ResourcePackAction {
    pub const fn is_terminal(&self) -> bool {
        !matches!(self, Self::Accepted | Self::Downloaded)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CustomClickAction<'a> {
    pub id: Ident<'a>,
    pub payload: Option<Tag>,
}
