use mser::V32;
use uuid::Uuid;

use crate::{ByteArray, Rest, Utf8};

#[derive(Clone, Serialize, Deserialize)]
pub struct Hello<'a> {
    pub name: Utf8<'a, 16>,
    pub profile_id: Uuid,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Key<'a> {
    pub key_bytes: ByteArray<'a>,
    pub encrypted_challenge: ByteArray<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CustomQueryAnswer<'a> {
    pub transaction_id: V32,
    pub payload: Option<Rest<'a, 1048576>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LoginAcknowledged {}
