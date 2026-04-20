#[derive(Clone, Serialize, Deserialize)]
pub struct StatusPongResponse(pub PongResponse);

#[derive(Clone, Serialize, Deserialize)]
pub struct GamePongResponse(pub PongResponse);

#[derive(Clone, Serialize, Deserialize)]
pub struct PongResponse {
    pub time: u64,
}
