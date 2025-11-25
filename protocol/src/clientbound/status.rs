use crate::Utf8;

#[derive(Clone, Serialize, Deserialize)]
pub struct StatusResponse<'a> {
    pub status: Utf8<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PongResponse {
    pub time: u64,
}
