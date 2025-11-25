#[derive(Clone, Serialize, Deserialize)]
pub struct StatusRequest {}

#[derive(Clone, Serialize, Deserialize)]
pub struct PingRequest {
    pub time: u64,
}
