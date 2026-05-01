#[derive(Clone, Serialize, Deserialize)]
pub struct AcceptTeleportation {
    #[mser(varint)]
    id: u32,
}
