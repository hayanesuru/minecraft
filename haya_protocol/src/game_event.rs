use crate::EntityId;
use haya_math::BlockPosPacked;
use minecraft_data::position_source_type;

#[derive(Clone, Copy, Serialize, Deserialize)]
#[mser(header = position_source_type)]
pub enum PositionSource {
    Block(BlockPositionSource),
    Entity(EntityPositionSource),
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct BlockPositionSource {
    pub pos: BlockPosPacked,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct EntityPositionSource {
    pub entity_id: EntityId,
    pub y_offset: f32,
}
