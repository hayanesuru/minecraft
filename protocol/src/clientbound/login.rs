use alloc::alloc::{Allocator, Global};
use mser::V32;

use crate::{ByteArray, GameProfile, Rest, Utf8};

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
pub struct LoginFinished<'a, A: Allocator = Global> {
    pub game_profile: GameProfile<'a, A>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LoginCompression {
    pub compression_threshold: V32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CustomQuery<'a> {
    pub transaction_id: V32,
    pub id: Utf8<'a>,
    pub payload: Rest<'a, 1048576>,
}
