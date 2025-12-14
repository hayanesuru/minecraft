use crate::{Ident, Rest};

#[derive(Clone, Serialize, Deserialize)]
pub struct CustomPayload<'a> {
    pub id: Ident<'a>,
    pub payload: Rest<'a, 1048576>,
}
