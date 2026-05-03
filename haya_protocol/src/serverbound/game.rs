use crate::chat::{
    LastSeenMessagesUpdate, MessageSignature, MessageSignaturePacked, RemoteChatSession,
};
use crate::command::ArgumentSignatures;
use crate::{Difficulty, GameType, MilliSeconds};
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

#[derive(Clone, Serialize, Deserialize)]
pub struct ChatCommandSigned<'a> {
    pub command: Utf8<'a>,
    pub time_stamp: MilliSeconds,
    pub salt: u64,
    pub argument_signatures: ArgumentSignatures<'a>,
    pub last_seen_messages: LastSeenMessagesUpdate<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Chat<'a> {
    pub message: Utf8<'a, 256>,
    pub time_stamp: MilliSeconds,
    pub salt: u64,
    pub signature: Option<MessageSignature<'a>>,
    pub last_seen_messages: LastSeenMessagesUpdate<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ChatSessionUpdate<'a> {
    pub chat_session: RemoteChatSession<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ChunkBatchReceived {
    pub desired_chunks_per_tick: f32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ClientCommand {
    pub action: ClientCommandAction,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum ClientCommandAction {
    PerformRespawn,
    RequestStats,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ClientTickEnd {}

#[derive(Clone, Serialize, Deserialize)]
pub struct ClientInformation<'a> {
    pub information: crate::ClientInformation<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CommandSuggestion<'a> {
    #[mser(varint)]
    pub id: u32,
    pub command: Utf8<'a, 32500>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ConfigurationAcknowledged {}
