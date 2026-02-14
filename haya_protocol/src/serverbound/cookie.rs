use haya_ident::Ident;
use mser::ByteArray;

#[derive(Clone, Serialize, Deserialize)]
pub struct CookieResponse<'a> {
    pub key: Ident<'a>,
    pub payload: Option<ByteArray<'a, 5120>>,
}
