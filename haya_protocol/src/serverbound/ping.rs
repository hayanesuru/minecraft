#[derive(Clone, Serialize, Deserialize)]
pub struct StatusPingRequest(pub PingRequest);

#[derive(Clone, Serialize, Deserialize)]
pub struct GamePingRequest(pub PingRequest);

#[derive(Clone, Serialize, Deserialize)]
pub struct PingRequest {
    pub time: u64,
}
