use crate::nbt::Tag;
use crate::{
    ByteArray, Component, Ident, List, Rest, ServerLinkUntrustedEntry, TagNetworkEntry, Utf8,
};
use mser::V32;
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize)]
pub struct CustomPayload<'a> {
    pub id: Ident<'a>,
    pub payload: Rest<'a, 1048576>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Disconnect {
    pub reason: Component,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct KeepAlive {
    pub id: u64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Ping {
    pub id: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ResetChat {}

#[derive(Clone, Serialize, Deserialize)]
pub struct ResourcePackPop {
    pub id: Option<Uuid>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ResourcePackPush<'a> {
    pub id: Uuid,
    pub url: Utf8<'a>,
    pub hash: Utf8<'a, 40>,
    pub required: bool,
    pub prompt: Option<Component>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct StoreCookie<'a> {
    pub key: Ident<'a>,
    pub payload: ByteArray<'a, 5120>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Transfer<'a> {
    pub host: Utf8<'a>,
    pub port: V32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct UpdateTags<'a> {
    pub tags: List<'a, TagNetworkEntry<'a>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CustomReportDetails<'a> {
    pub details: List<'a, CustomReportDetailsEntry<'a>, 32>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CustomReportDetailsEntry<'a> {
    pub key: Utf8<'a, 128>,
    pub value: Utf8<'a, 4096>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ServerLinks<'a> {
    pub links: List<'a, ServerLinkUntrustedEntry<'a>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ClearDialog {}

#[derive(Clone, Serialize, Deserialize)]
pub struct ShowDialog {
    dialog: Tag,
}
