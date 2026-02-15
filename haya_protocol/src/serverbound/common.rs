use crate::{ClientInformation, Rest};
use haya_ident::Ident;

#[derive(Clone, Serialize, Deserialize)]
pub struct ConfigurationClientInformation<'a>(pub ClientInformation<'a>);

#[derive(Clone, Serialize, Deserialize)]
pub struct CustomPayload<'a> {
    pub id: Ident<'a>,
    pub payload: Rest<'a, 32767>,
}
