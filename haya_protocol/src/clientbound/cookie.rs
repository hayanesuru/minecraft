use crate::Ident;

#[derive(Clone, Serialize, Deserialize)]
pub struct LoginCookieRequest<'a> {
    pub key: Ident<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ConfigCookieRequest<'a>(pub LoginCookieRequest<'a>);
