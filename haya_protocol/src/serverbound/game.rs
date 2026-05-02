use crate::{Difficulty, GameType};
use haya_math::BlockPosPacked;
use mser::Utf8;

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

#[derive(Clone, Serialize, Deserialize)]
pub struct BundleItemSelected {
    #[mser(varint)]
    pub slot_id: u32,
    #[mser(varint, filter = valicate_bundle_item_selected)]
    pub selected_item_index: u32,
}

fn valicate_bundle_item_selected(x: &u32) -> bool {
    let x = *x as i32;
    x >= 0 || x == -1
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ChangeDifficulty {
    pub difficulty: Difficulty,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ChangeGameMode {
    pub mode: GameType,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ChatAck {
    pub offset: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ChatCommand<'a> {
    pub command: Utf8<'a>,
}
