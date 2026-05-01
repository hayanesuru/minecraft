use haya_math::BlockPosPacked;

#[derive(Clone, Serialize, Deserialize)]
pub struct AcceptTeleportation {
    #[mser(varint)]
    id: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BlockEntityTagQuery {
    #[mser(varint)]
    pub transaction_id: u32,
    pub pos: BlockPosPacked,
}
