use crate::{ByteArray, Identifier};

#[derive(Clone, Serialize, Deserialize)]
pub struct CookieResponse<'a> {
    pub key: Identifier<'a>,
    pub payload: Option<ByteArray<'a, 5120>>,
}
