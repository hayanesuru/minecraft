use crate::{ByteArray, Component, Ident, Rest, Utf8};
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
