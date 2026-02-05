use crate::{Component, Ident, Rest};

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
