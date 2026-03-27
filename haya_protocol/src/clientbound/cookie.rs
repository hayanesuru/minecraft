use haya_ident::Ident;

#[derive(Clone, Serialize, Deserialize)]
pub struct LoginCookieRequest<'a>(pub CookieRequest<'a>);

#[derive(Clone, Serialize, Deserialize)]
pub struct ConfigurationCookieRequest<'a>(pub CookieRequest<'a>);

#[derive(Clone, Serialize, Deserialize)]
pub struct GameCookieRequest<'a>(pub CookieRequest<'a>);

#[derive(Clone, Serialize, Deserialize)]
pub struct CookieRequest<'a> {
    pub key: Ident<'a>,
}
