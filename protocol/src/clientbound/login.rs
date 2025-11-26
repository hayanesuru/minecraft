use crate::{ByteArray, GameProfile, Utf8};

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
    pub game_profile: GameProfile<'a>,
}
