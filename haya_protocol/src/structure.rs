use haya_math::BlockPosPacked;

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct BoundingBoxPacked {
    pub min: BlockPosPacked,
    pub max: BlockPosPacked,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct PiecePacked {
    pub bounding_box: BoundingBoxPacked,
    pub is_start: bool,
}
