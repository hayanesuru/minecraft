#[derive(Clone, Serialize, Deserialize)]
pub struct PingRequest {
    pub time: u64,
}
