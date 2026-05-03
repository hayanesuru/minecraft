use crate::chat::{LastSeenMessagesUpdate, MessageSignature, RemoteChatSession};
use crate::command::ArgumentSignatures;
use crate::crafting::RecipeDisplayId;
use crate::inventory::{ContainerId, InteractionHand, RecipeBookType};
use crate::{ClickType, Difficulty, GameType, HashedStack, Input, MilliSeconds};
use haya_collection::{List, Map};
use haya_ident::Ident;
use haya_math::{BlockPosPacked, ByteDirection, FVec3, Vec3};
use minecraft_data::debug_subscription;
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

#[derive(Clone, Serialize, Deserialize)]
pub struct ContainerButtonClick {
    pub container_id: ContainerId,
    #[mser(varint)]
    pub button_id: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ContainerClick<'a> {
    pub container_id: ContainerId,
    #[mser(varint)]
    pub state_id: u32,
    pub slot_num: u16,
    pub button_num: u8,
    pub click_type: ClickType,
    pub changed_slots: Map<'a, u16, Option<HashedStack<'a>>, 128>,
    pub carried_item: Option<HashedStack<'a>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ContainerClose {
    pub container_id: ContainerId,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ContainerSlotStateChanged {
    #[mser(varint)]
    pub slot_id: u32,
    pub container_id: ContainerId,
    pub new_state: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DebugSubscriptionRequest<'a> {
    pub subscriptions: List<'a, debug_subscription>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct EditBook<'a> {
    #[mser(varint)]
    pub slot: u32,
    pub pages: List<'a, Utf8<'a, 1024>, 100>,
    pub title: Option<Utf8<'a, 32>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct EntityTagQuery {
    #[mser(varint)]
    pub transaction_id: u32,
    #[mser(varint)]
    pub entity_id: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Interact {
    #[mser(varint)]
    pub entity_id: u32,
    pub action: InteractAction,
    pub using_secondary_action: bool,
}

#[derive(Clone, Serialize, Deserialize)]
#[mser(header = InteractActionType, camel_case)]
pub enum InteractAction {
    Interact {
        hand: InteractionHand,
    },
    Attack,
    InteractAt {
        location: FVec3,
        hand: InteractionHand,
    },
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum InteractActionType {
    Interact,
    Attack,
    InteractAt,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct JigsawGenerate {
    pub pos: BlockPosPacked,
    #[mser(varint)]
    pub levels: u32,
    pub keep_jigsaws: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LockDifficulty {
    pub locked: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MovePlayerPos {
    pub pos: Vec3,
    pub flags: MovePlayerFlags,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct MovePlayerFlags(pub u8);

impl MovePlayerFlags {
    pub const ON_GROUND: u8 = 1;
    pub const HORIZONTAL_COLLISION: u8 = 2;
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MovePlayerPosRot {
    pub pos: Vec3,
    pub y_rot: f32,
    pub x_rot: f32,
    pub flags: MovePlayerFlags,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MovePlayerRot {
    pub y_rot: f32,
    pub x_rot: f32,
    pub flags: MovePlayerFlags,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MovePlayerStatusOnly {
    pub flags: MovePlayerFlags,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MoveVehicle {
    pub position: Vec3,
    pub y_rot: f32,
    pub x_rot: f32,
    pub on_ground: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PaddleBoat {
    pub left: bool,
    pub right: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PickItemFromBlock {
    pub pos: BlockPosPacked,
    pub include_data: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PickItemFromEntity {
    #[mser(varint)]
    pub id: u32,
    pub include_data: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PlaceRecipe {
    pub container_id: ContainerId,
    pub recipe: RecipeDisplayId,
    pub use_max_items: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PlayerAbilities {
    pub flags: PlayerAbilitiesFlags,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct PlayerAbilitiesFlags(pub u8);

impl PlayerAbilitiesFlags {
    pub const FLYING: u8 = 2;

    pub const fn flying(self) -> bool {
        self.0 & Self::FLYING != 0
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PlayerAction {
    pub action: PlayerActionType,
    pub pos: BlockPosPacked,
    pub direction: ByteDirection,
    #[mser(varint)]
    pub sequence: u32,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum PlayerActionType {
    StartDestroyBlock,
    AbortDestroyBlock,
    StopDestroyBlock,
    DropAllItems,
    DropItem,
    ReleaseUseItem,
    SwapItemWithOffhand,
    Stab,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PlayerCommand {
    #[mser(varint)]
    pub id: u32,
    pub action: PlayerCommandAction,
    #[mser(varint)]
    pub data: u32,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum PlayerCommandAction {
    StopSleeping,
    StartSprinting,
    StopSprinting,
    StartRidingJump,
    StopRidingJump,
    OpenInventory,
    StartFallFlying,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PlayerInput {
    pub input: Input,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PlayerLoaded {}

#[derive(Clone, Serialize, Deserialize)]
pub struct RecipeBookChangeSettings {
    pub book_type: RecipeBookType,
    pub is_open: bool,
    pub is_filtering: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RecipeBookSeenRecipe {
    pub recipe: RecipeDisplayId,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RenameItem<'a> {
    pub name: Utf8<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SeenAdvancements<'a> {
    pub action: SeenAdvancementsAction<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
#[mser(header = SeenAdvancementsType, camel_case)]
pub enum SeenAdvancementsAction<'a> {
    OpenedTab { tab: Ident<'a> },
    ClosedScreen,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum SeenAdvancementsType {
    OpenedTab,
    ClosedScreen,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SelectTrade {
    #[mser(varint)]
    pub item: u32,
}
