use crate::chat::{Bound, MessageSignaturePacked};
use crate::command::CommandNode;
use crate::debug::{DebugSubscriptionEvent, DebugSubscriptionUpdate, RemoteDebugSampleType};
use crate::item::OptionalItemStack;
use crate::map::{MapDecoration, MapId, MapPatch};
use crate::minecart::MinecartStep;
use crate::particle::{ExplosionParticleInfo, Particle};
use crate::recipe::RecipeDisplay;
use crate::registry::{DamageTypeRef, DimensionTypeRef, SoundEventRef};
use crate::sound::SoundEvent;
use crate::stat::Stat;
use crate::trading::MerchantOffer;
use crate::{
    BitSet, Component, ContainerId, Difficulty, GameType, GameTypeOptional, GlobalPos,
    HeightmapType, Holder, InteractionHand, WeightedList,
};
use haya_collection::{List, Map};
use haya_ident::{Ident, ResourceKey};
use haya_math::{BlockPosPacked, ByteAngle, ChunkPos, LpVec3, Vec3};
use haya_nbt::Tag;
use minecraft_data::{block, block_entity_type, block_state, entity_type, menu};
use mser::{ByteArray, Utf8, V21};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize)]
pub struct BundleDelimiter {}

#[derive(Clone, Serialize, Deserialize)]
pub struct AddEntity {
    #[mser(varint)]
    pub id: u32,
    pub uuid: Uuid,
    pub r#type: entity_type,
    pub pos: Vec3,
    pub movement: LpVec3,
    pub x_rot: ByteAngle,
    pub y_rot: ByteAngle,
    pub y_head_rot: ByteAngle,
    #[mser(varint)]
    pub data: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Animate {
    #[mser(varint)]
    pub id: u32,
    pub action: u8,
}

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum AnimateAction {
    SwingMainHand = 0,
    WakeUp = 2,
    SwingOffHand = 3,
    CriticalHit = 4,
    MagicCriticalHit = 5,
}

impl AnimateAction {
    pub const fn new(action: u8) -> Option<Self> {
        match action {
            0 => Some(Self::SwingMainHand),
            2 => Some(Self::WakeUp),
            3 => Some(Self::SwingOffHand),
            4 => Some(Self::CriticalHit),
            5 => Some(Self::MagicCriticalHit),
            _ => None,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AwardStats<'a> {
    pub stats: Map<'a, Stat, StatValue>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct StatValue(#[mser(varint)] pub u32);

#[derive(Clone, Serialize, Deserialize)]
pub struct BlockChangedAck {
    #[mser(varint)]
    pub sequence: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BlockDestruction {
    #[mser(varint)]
    pub id: u32,
    pub pos: BlockPosPacked,
    pub progress: u8,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BlockEntityData {
    pub pos: BlockPosPacked,
    pub r#type: block_entity_type,
    pub tag: Tag,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BlockEvent {
    pub pos: BlockPosPacked,
    pub param_a: u8,
    pub param_b: u8,
    pub block_type: block,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BlockUpdate {
    pub pos: BlockPosPacked,
    pub state: block_state,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BossEvent {
    pub id: Uuid,
    pub operation: BossEventOperation,
}

#[derive(Clone, Serialize, Deserialize)]
#[mser(header = BossEventOperationType, camel_case)]
pub enum BossEventOperation {
    Add {
        name: Component,
        progress: f32,
        color: BossEventColor,
        overlay: BossEventOverlay,
        flags: BossEventFlags,
    },
    Remove {},
    UpdateProgress {
        progress: f32,
    },
    UpdateName {
        name: Component,
    },
    UpdateStyle {
        color: BossEventColor,
        overlay: BossEventOverlay,
    },
    UpdateProperties {
        flags: BossEventFlags,
    },
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum BossEventOperationType {
    Add,
    Remove,
    UpdateProgress,
    UpdateName,
    UpdateStyle,
    UpdateProperties,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum BossEventColor {
    Pink,
    Blue,
    Red,
    Green,
    Yellow,
    Purple,
    White,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum BossEventOverlay {
    Progress,
    Notched6,
    Notched10,
    Notched12,
    Notched20,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct BossEventFlags(pub u8);

impl BossEventFlags {
    pub const DARKEN_SCREEN: u8 = 1;
    pub const PLAY_MUSIC: u8 = 2;
    pub const CREATE_WORLD_FOG: u8 = 4;
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ChangeDifficulty {
    pub difficulty: Difficulty,
    pub locked: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ChunkBatchFinished {
    #[mser(varint)]
    pub batch_size: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ChunkBatchStart {}

#[derive(Clone, Serialize, Deserialize)]
pub struct ChunkBiomes<'a> {
    pub data: List<'a, ChunkBiomeData<'a>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ChunkBiomeData<'a> {
    pub pos: ChunkPos,
    pub data: ByteArray<'a, 2097152>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ClearTitles {
    pub reset_times: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CommandSuggestions<'a> {
    #[mser(varint)]
    pub id: u32,
    #[mser(varint)]
    pub start: u32,
    #[mser(varint)]
    pub length: u32,
    pub suggestions: List<'a, SuggestionEntry<'a>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SuggestionEntry<'a> {
    pub text: Utf8<'a>,
    pub tooltip: Option<Component>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Commands<'a> {
    pub entries: List<'a, CommandNode<'a>>,
    pub root_index: V21,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ContainerClose {
    pub container_id: ContainerId,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ContainerSetContent<'a> {
    pub container_id: ContainerId,
    #[mser(varint)]
    pub state_id: u32,
    pub items: List<'a, OptionalItemStack<'a>>,
    pub carried_item: OptionalItemStack<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ContainerSetData {
    pub container_id: ContainerId,
    pub id: u16,
    pub value: u16,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ContainerSetSlot<'a> {
    pub container_id: ContainerId,
    #[mser(varint)]
    pub state_id: u32,
    pub slot: u16,
    pub item_stack: OptionalItemStack<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Cooldown<'a> {
    pub cooldown_group: Ident<'a>,
    #[mser(varint)]
    pub duration: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CustomChatCompletions<'a> {
    pub action: CustomChatCompletionsAction,
    pub entries: List<'a, Utf8<'a>>,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum CustomChatCompletionsAction {
    Add,
    Remove,
    Set,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DamageEvent {
    #[mser(varint)]
    pub entity_id: u32,
    pub source_type: DamageTypeRef,
    pub source_cause_id: OptionalEntityId,
    pub source_direct_id: OptionalEntityId,
    pub source_position: Option<Vec3>,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct OptionalEntityId(#[mser(varint)] u32);

impl OptionalEntityId {
    pub const fn new(id: u32) -> Self {
        Self(id.wrapping_add(1))
    }

    pub const fn id(self) -> u32 {
        self.0.wrapping_sub(1)
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DebugBlockValue<'a> {
    pub block_pos: BlockPosPacked,
    pub update: DebugSubscriptionUpdate<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DebugChunkValue<'a> {
    pub chunk_pos: ChunkPos,
    pub update: DebugSubscriptionUpdate<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DebugEntityValue<'a> {
    #[mser(varint)]
    pub entity_id: u32,
    pub update: DebugSubscriptionUpdate<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DebugEvent<'a> {
    pub event: DebugSubscriptionEvent<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DebugSample<'a> {
    pub sample: List<'a, u64>,
    pub debug_sample_type: RemoteDebugSampleType,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DeleteChat<'a> {
    pub message_signature: MessageSignaturePacked<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DisguisedChat<'a> {
    pub message: Component,
    pub chat_type: Bound<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct EntityEvent {
    pub entity_id: u32,
    pub event_id: u8,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct EntityPositionSync {
    #[mser(varint)]
    pub id: u32,
    pub values: PositionMoveRotation,
    pub on_ground: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PositionMoveRotation {
    pub position: Vec3,
    pub delta_movement: Vec3,
    pub y_rot: f32,
    pub x_rot: f32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Explode<'a> {
    pub center: Vec3,
    pub radius: f32,
    pub block_count: u32,
    pub player_knockback: Option<Vec3>,
    pub explosion_particle: Particle<'a>,
    pub explosion_sound: Holder<SoundEvent<'a>, SoundEventRef>,
    pub block_particles: WeightedList<'a, ExplosionParticleInfo<'a>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ForgetLevelChunk {
    pub pos: ChunkPos,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct GameEvent {
    pub event: GameEventType,
    pub param: f32,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
pub enum GameEventType {
    NoRespawnBlockAvailable,
    StartRaining,
    StopRaining,
    ChangeGameMode,
    WinGame,
    DemoEvent,
    PlayArrowHitSound,
    RainLevelChange,
    ThunderLevelChange,
    PufferFishSting,
    GuardianElderEffect,
    ImmediateRespawn,
    LimitedCrafting,
    LevelChunksLoadStart,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct GameTestHighlightPos {
    pub absolute_pos: BlockPosPacked,
    pub relative_pos: BlockPosPacked,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MountScreenOpen {
    pub container_id: ContainerId,
    #[mser(varint)]
    pub inventory_columns: u32,
    pub entity_id: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct HurtAnimation {
    #[mser(varint)]
    pub id: u32,
    pub yaw: f32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InitializeBorder {
    pub new_center_x: f64,
    pub new_center_z: f64,
    pub old_size: f64,
    pub new_size: f64,
    #[mser(varint)]
    pub lerp_time: u64,
    #[mser(varint)]
    pub new_absolute_max_size: u32,
    #[mser(varint)]
    pub warning_blocks: u32,
    #[mser(varint)]
    pub warning_time: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LevelChunkWithLight<'a> {
    pub pos: ChunkPos,
    pub chunk_data: ChunkData<'a>,
    pub light_data: LightData<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ChunkData<'a> {
    pub heightmaps: Map<'a, HeightmapType, List<'a, u64>>,
    pub data: ByteArray<'a>,
    pub block_entities_data: List<'a, BlockEntityInfo>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BlockEntityInfo {
    pub packed_xz: u8,
    pub y: i16,
    pub ty: block_entity_type,
    pub tag: Tag,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LightData<'a> {
    pub sky_y_mask: BitSet<'a>,
    pub block_y_mask: BitSet<'a>,
    pub empty_sky_y_mask: BitSet<'a>,
    pub empty_block_y_mask: BitSet<'a>,
    pub sky_updates: List<'a, ByteArray<'a, 2048>>,
    pub block_updates: List<'a, ByteArray<'a, 2048>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LevelEvent {
    pub ty: u32,
    pub pos: BlockPosPacked,
    pub data: u32,
    pub global_event: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LevelParticles<'a> {
    pub override_limiter: bool,
    pub always_show: bool,
    pub pos: Vec3,
    pub x_dist: f32,
    pub y_dist: f32,
    pub z_dist: f32,
    pub max_speed: f32,
    pub count: u32,
    pub particle: Particle<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LightUpdate<'a> {
    #[mser(varint)]
    pub x: i32,
    #[mser(varint)]
    pub z: i32,
    pub light_data: LightData<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Login<'a> {
    #[mser(varint)]
    pub player_id: u32,
    pub hardcore: bool,
    pub levels: List<'a, ResourceKey<'a>>,
    #[mser(varint)]
    pub max_players: u32,
    #[mser(varint)]
    pub chunk_radius: u32,
    #[mser(varint)]
    pub simulation_distance: u32,
    pub reduced_debug_info: bool,
    pub show_death_screen: bool,
    pub do_limited_crafting: bool,
    pub common_player_spawn_info: CommonPlayerSpawnInfo<'a>,
    pub enforces_secure_chat: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CommonPlayerSpawnInfo<'a> {
    pub dimension_type: DimensionTypeRef,
    pub dimension: ResourceKey<'a>,
    pub seed: u64,
    pub game_type: GameType,
    pub previous_game_type: GameTypeOptional,
    pub is_debug: bool,
    pub is_flat: bool,
    pub last_death_location: Option<GlobalPos<'a>>,
    #[mser(varint)]
    pub portal_cooldown: u32,
    #[mser(varint)]
    pub sea_level: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MapItemData<'a> {
    pub map_id: MapId,
    pub scale: u8,
    pub locked: bool,
    pub decorations: Option<List<'a, MapDecoration>>,
    pub color_patch: MapPatch<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MerchantOffers<'a> {
    pub container_id: ContainerId,
    pub offers: List<'a, MerchantOffer<'a>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MoveEntityPos {
    #[mser(varint)]
    pub entity_id: u32,
    pub xa: i16,
    pub ya: i16,
    pub za: i16,
    pub on_ground: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MoveEntityPosRot {
    #[mser(varint)]
    pub entity_id: u32,
    pub xa: i16,
    pub ya: i16,
    pub za: i16,
    pub y_rot: ByteAngle,
    pub x_rot: ByteAngle,
    pub on_ground: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MoveMinecartAlongTrack<'a> {
    #[mser(varint)]
    pub entity_id: u32,
    pub lerp_steps: List<'a, MinecartStep>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MoveEntityRot {
    #[mser(varint)]
    pub entity_id: u32,
    pub y_rot: ByteAngle,
    pub x_rot: ByteAngle,
    pub on_ground: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MoveVehicle {
    pub position: Vec3,
    pub y_rot: f32,
    pub x_rot: f32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct OpenBook {
    pub hand: InteractionHand,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct OpenScreen {
    pub container_id: ContainerId,
    pub ty: menu,
    pub title: Component,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct OpenSignEditor {
    pub pos: BlockPosPacked,
    pub is_front_text: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PlaceGhostRecipe<'a> {
    pub container_id: ContainerId,
    pub recipe_display: RecipeDisplay<'a>,
}
