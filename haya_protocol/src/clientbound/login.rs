use crate::profile::GameProfileRef;
use crate::{Rest, Utf8};
use haya_ident::Ident;
use mser::{ByteArray, V32};

#[derive(Clone, Serialize, Deserialize)]
pub struct LoginDisconnect<'a> {
    pub status: Utf8<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Hello<'a> {
    pub server_id: Utf8<'a, 20>,
    pub public_key: ByteArray<'a>,
    pub challenge: ByteArray<'a>,
    pub should_authenticate: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LoginFinished<'a> {
    pub game_profile: GameProfileRef<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LoginCompression {
    pub compression_threshold: V32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CustomQuery<'a> {
    pub transaction_id: V32,
    pub id: Ident<'a>,
    pub payload: Rest<'a, 1048576>,
}
