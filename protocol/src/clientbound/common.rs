use crate::{Identifier, Rest};

#[derive(Clone, Serialize, Deserialize)]
pub struct CustomPayload<'a> {
    pub id: Identifier<'a>,
    pub payload: Rest<'a, 1048576>,
}
