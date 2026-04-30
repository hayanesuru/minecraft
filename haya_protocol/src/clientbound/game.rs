use crate::chat::{
    Bound, FilterMask, MessageSignature, MessageSignaturePacked, RemoteChatSession,
    SignedMessageBodyPacked,
};
use crate::command::CommandNode;
use crate::debug::{DebugSubscriptionEvent, DebugSubscriptionUpdate, RemoteDebugSampleType};
use crate::entity_data::EntityDataSerializer;
use crate::item::OptionalItemStack;
use crate::map::{MapDecoration, MapId, MapPatch};
use crate::minecart::MinecartStep;
use crate::particle::{ExplosionParticleInfo, Particle};
use crate::profile::PropertyMap;
use crate::recipe::{RecipeDisplay, RecipeDisplayEntry, RecipeDisplayId};
use crate::registry::{DamageTypeRef, DimensionTypeRef};
use crate::score::{DisplaySlot, ObjectiveCriteriaRenderType, TeamCollisionRule, TeamVisibility};
use crate::sound::{SoundEvent, SoundSource};
use crate::stat::Stat;
use crate::trading::MerchantOffer;
use crate::{
    BitSet, ChatFormatting, Component, ContainerId, Difficulty, EntityAnchor, EquipmentSlot,
    GameType, GlobalPos, HeightmapType, Holder, InteractionHand, OptionalGameType, RespawnData,
    V32List, WeightedList,
};
use alloc::vec::Vec;
use haya_collection::{List, Map, capacity_fix};
use haya_ident::{Ident, ResourceKey};
use haya_math::{BlockPosPacked, ByteAngle, ChunkPos, ChunkSectionPosPacked, LpVec3, Vec3};
use haya_nbt::Tag;
use minecraft_data::{
    block, block_entity_type, block_state, entity_type, menu, mob_effect, number_format_type,
    sound_event,
};
use mser::{ByteArray, Error, Read, Reader, Utf8, V21, V32, Write, Writer};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize)]
pub struct BundleDelimiter {}

#[derive(Clone, Serialize, Deserialize)]
pub struct AddEntity {
    #[mser(varint)]
    pub id: u32,
    pub uuid: Uuid,
    pub ty: entity_type,
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
    pub id: AnimateAction,
    pub action: u8,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum AnimateAction {
    SwingMainHand,
    Unused,
    WakeUp,
    SwingOffHand,
    CriticalHit,
    MagicCriticalHit,
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
    pub ty: block_entity_type,
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
    pub explosion_sound: Holder<SoundEvent<'a>, sound_event>,
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
    pub previous_game_type: OptionalGameType,
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

#[derive(Clone, Serialize, Deserialize)]
pub struct PlayerAbilities {
    pub flags: PlayerAbilitiesFlags,
    pub flying_speed: f32,
    pub walking_speed: f32,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct PlayerAbilitiesFlags(pub u8);

impl PlayerAbilitiesFlags {
    pub const INVULNERABLE: u8 = 1;
    pub const FLYING: u8 = 2;
    pub const CAN_FLY: u8 = 4;
    pub const INSTABUILD: u8 = 8;
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PlayerChat<'a> {
    #[mser(varint)]
    pub global_index: u32,
    pub sender: Uuid,
    #[mser(varint)]
    pub index: u32,
    pub signature: Option<MessageSignature<'a>>,
    pub body: SignedMessageBodyPacked<'a>,
    pub unsigned_content: Option<Component>,
    pub filter_mask: FilterMask<'a>,
    pub chat_type: Bound<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PlayerCombatEnd {
    #[mser(varint)]
    pub duration: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PlayerCombatEnter {}

#[derive(Clone, Serialize, Deserialize)]
pub struct PlayerCombatKill {
    #[mser(varint)]
    pub player_id: u32,
    pub message: Component,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PlayerInfoRemove<'a> {
    pub profile_ids: List<'a, Uuid>,
}

pub struct PlayerInfoUpdate<'a> {
    pub actions: PlayerInfoUpdateActions,
    pub entries: List<'a, PlayerInfoUpdateEntry<'a>>,
}

impl<'a> Write for PlayerInfoUpdate<'a> {
    unsafe fn write(&self, w: &mut Writer) {
        unsafe {
            self.actions.write(w);
            for entry in self.entries.iter() {
                entry.write(w, self.actions);
            }
        }
    }

    fn len_s(&self) -> usize {
        let mut l = self.actions.len_s();
        for entry in self.entries.iter() {
            l += entry.len_s(self.actions);
        }
        l
    }
}

impl<'a> Read<'a> for PlayerInfoUpdate<'a> {
    fn read(buf: &mut Reader<'a>) -> Result<Self, Error> {
        let actions = PlayerInfoUpdateActions::read(buf)?;
        let len = V21::read(buf)?.0 as usize;
        let mut vec = Vec::with_capacity(capacity_fix(len));
        for _ in 0..len {
            vec.push(PlayerInfoUpdateEntry::read(buf, actions)?);
        }
        Ok(Self {
            actions,
            entries: List::Owned(vec),
        })
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct PlayerInfoUpdateActions(pub u8);

impl PlayerInfoUpdateActions {
    pub const ADD_PLAYER: u8 = 1;
    pub const INITIALIZE_CHAT: u8 = 2;
    pub const UPDATE_GAME_MODE: u8 = 4;
    pub const UPDATE_LISTED: u8 = 8;
    pub const UPDATE_LATENCY: u8 = 16;
    pub const UPDATE_DISPLAY_NAME: u8 = 32;
    pub const UPDATE_LIST_ORDER: u8 = 64;
    pub const UPDATE_HAT: u8 = 128;

    pub const fn add_player(self) -> bool {
        self.0 & Self::ADD_PLAYER != 0
    }

    pub const fn initialize_chat(self) -> bool {
        self.0 & Self::INITIALIZE_CHAT != 0
    }

    pub const fn update_game_mode(self) -> bool {
        self.0 & Self::UPDATE_GAME_MODE != 0
    }

    pub const fn update_listed(self) -> bool {
        self.0 & Self::UPDATE_LISTED != 0
    }

    pub const fn update_latency(self) -> bool {
        self.0 & Self::UPDATE_LATENCY != 0
    }

    pub const fn update_display_name(self) -> bool {
        self.0 & Self::UPDATE_DISPLAY_NAME != 0
    }

    pub const fn update_list_order(self) -> bool {
        self.0 & Self::UPDATE_LIST_ORDER != 0
    }

    pub const fn update_hat(self) -> bool {
        self.0 & Self::UPDATE_HAT != 0
    }
}

#[derive(Clone)]
pub struct PlayerInfoUpdateEntry<'a> {
    pub profile_id: Uuid,
    pub name: Utf8<'a, 16>,
    pub properties: PropertyMap<'a>,
    pub chat_session: Option<RemoteChatSession<'a>>,
    pub game_mode: GameType,
    pub listed: bool,
    pub latency: u32,
    pub display_name: Option<Component>,
    pub list_order: u32,
    pub show_hat: bool,
}

impl<'a> PlayerInfoUpdateEntry<'a> {
    /// # Safety
    ///
    /// [`mser::Write::write`]
    pub unsafe fn write(&self, w: &mut Writer, actions: PlayerInfoUpdateActions) {
        unsafe {
            self.profile_id.write(w);
            if actions.add_player() {
                self.name.write(w);
                self.properties.write(w);
            }
            if actions.initialize_chat() {
                self.chat_session.write(w);
            }
            if actions.update_game_mode() {
                self.game_mode.write(w);
            }
            if actions.update_listed() {
                self.listed.write(w);
            }
            if actions.update_latency() {
                V32(self.latency).write(w);
            }
            if actions.update_display_name() {
                self.display_name.write(w);
            }
            if actions.update_list_order() {
                V32(self.list_order).write(w);
            }
            if actions.update_hat() {
                self.show_hat.write(w);
            }
        }
    }

    pub fn len_s(&self, actions: PlayerInfoUpdateActions) -> usize {
        let mut len = self.profile_id.len_s();
        if actions.add_player() {
            len += self.name.len_s();
            len += self.properties.len_s();
        }
        if actions.initialize_chat() {
            len += self.chat_session.len_s();
        }
        if actions.update_game_mode() {
            len += self.game_mode.len_s();
        }
        if actions.update_listed() {
            len += self.listed.len_s();
        }
        if actions.update_latency() {
            len += V32(self.latency).len_s();
        }
        if actions.update_display_name() {
            len += self.display_name.len_s();
        }
        if actions.update_list_order() {
            len += V32(self.list_order).len_s();
        }
        if actions.update_hat() {
            len += self.show_hat.len_s();
        }
        len
    }

    pub fn read(buf: &mut Reader<'a>, actions: PlayerInfoUpdateActions) -> Result<Self, Error> {
        Ok(Self {
            profile_id: Uuid::read(buf)?,
            name: if actions.add_player() {
                Utf8::read(buf)?
            } else {
                Utf8("")
            },
            properties: if actions.add_player() {
                PropertyMap::read(buf)?
            } else {
                PropertyMap(Map(List::Borrowed(&[])))
            },
            chat_session: if actions.initialize_chat() {
                Read::read(buf)?
            } else {
                None
            },
            game_mode: if actions.update_game_mode() {
                GameType::read(buf)?
            } else {
                GameType::Survival
            },
            listed: if actions.update_listed() {
                bool::read(buf)?
            } else {
                false
            },
            latency: if actions.update_latency() {
                V32::read(buf)?.0
            } else {
                0
            },
            display_name: if actions.update_display_name() {
                Read::read(buf)?
            } else {
                None
            },
            list_order: if actions.update_list_order() {
                V32::read(buf)?.0
            } else {
                0
            },
            show_hat: if actions.update_hat() {
                bool::read(buf)?
            } else {
                false
            },
        })
    }
}
#[derive(Clone, Serialize, Deserialize)]
pub struct PlayerLookAt {
    pub from_anchor: EntityAnchor,
    pub pos: Vec3,
    pub at_entity: Option<PlayerLookAtEntity>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PlayerLookAtEntity {
    #[mser(varint)]
    pub entity: u32,
    pub to_anchor: EntityAnchor,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PlayerPosition {
    #[mser(varint)]
    pub id: u32,
    pub change: PositionMoveRotation,
    pub relatives: Relatives,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Relatives(pub u32);

impl Relatives {
    pub const X: u32 = 1;
    pub const Y: u32 = 2;
    pub const Z: u32 = 4;
    pub const Y_ROT: u32 = 8;
    pub const X_ROT: u32 = 16;
    pub const DELTA_X: u32 = 32;
    pub const DELTA_Y: u32 = 64;
    pub const DELTA_Z: u32 = 128;
    pub const ROTATE_DELTA: u32 = 256;

    pub const ALL: u32 = Self::X
        | Self::Y
        | Self::Z
        | Self::Y_ROT
        | Self::X_ROT
        | Self::DELTA_X
        | Self::DELTA_Y
        | Self::DELTA_Z
        | Self::ROTATE_DELTA;
    pub const ROTATION: u32 = Self::Y_ROT | Self::X_ROT;
    pub const DELTA: u32 = Self::DELTA_X | Self::DELTA_Y | Self::DELTA_Z | Self::ROTATE_DELTA;

    pub const fn x(self) -> bool {
        self.0 & Self::X != 0
    }

    pub const fn y(self) -> bool {
        self.0 & Self::Y != 0
    }

    pub const fn z(self) -> bool {
        self.0 & Self::Z != 0
    }

    pub const fn y_rot(self) -> bool {
        self.0 & Self::Y_ROT != 0
    }

    pub const fn x_rot(self) -> bool {
        self.0 & Self::X_ROT != 0
    }

    pub const fn delta_x(self) -> bool {
        self.0 & Self::DELTA_X != 0
    }

    pub const fn delta_y(self) -> bool {
        self.0 & Self::DELTA_Y != 0
    }

    pub const fn delta_z(self) -> bool {
        self.0 & Self::DELTA_Z != 0
    }

    pub const fn rotate_delta(self) -> bool {
        self.0 & Self::ROTATE_DELTA != 0
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PlayerRotation {
    pub y_rot: f32,
    pub relative_y: bool,
    pub x_rot: f32,
    pub relative_x: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RecipeBookAdd<'a> {
    pub entries: List<'a, RecipeBookAddEntry<'a>>,
    pub replace: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RecipeBookAddEntry<'a> {
    pub contents: RecipeDisplayEntry<'a>,
    pub flags: RecipeBookAddFlags,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct RecipeBookAddFlags(pub u8);

impl RecipeBookAddFlags {
    pub const NOTIFICATION: u8 = 1;
    pub const HIGHLIGHT: u8 = 2;
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RecipeBookRemove<'a> {
    pub recipes: List<'a, RecipeDisplayId>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RecipeBookSettings {
    pub crafting: TypeSettings,
    pub furnace: TypeSettings,
    pub blast_furnace: TypeSettings,
    pub smoker: TypeSettings,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TypeSettings {
    pub open: bool,
    pub filtering: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RemoveEntities<'a> {
    pub entity_ids: V32List<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RemoveMobEffect {
    #[mser(varint)]
    pub entity_id: u32,
    pub effect: mob_effect,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ResetScore<'a> {
    pub owner: Utf8<'a>,
    pub objective_name: Option<Utf8<'a>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ResourcePackPop {
    pub id: Option<Uuid>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ResourcePackPush<'a> {
    pub id: Uuid,
    pub url: Utf8<'a>,
    pub hash: Utf8<'a, 40>,
    pub required: bool,
    pub prompt: Option<Component>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Respawn<'a> {
    pub common_player_spawn_info: CommonPlayerSpawnInfo<'a>,
    pub data_to_keep: RespawnFlags,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct RespawnFlags(pub u8);

impl RespawnFlags {
    pub const KEEP_ATTRIBUTE_MODIFIERS: u8 = 1;
    pub const KEEP_ENTITY_DATA: u8 = 2;
    pub const KEEP_ALL_DATA: u8 = 4;
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RotateHead {
    #[mser(varint)]
    pub entity_id: u32,
    pub y_head_rot: ByteAngle,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SectionBlocksUpdate<'a> {
    pub section_pos: ChunkSectionPosPacked,
    pub changes: List<'a, SectionBlocksUpdatePacked>,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(transparent)]
pub struct SectionBlocksUpdatePacked(#[mser(varint)] pub u64);

impl SectionBlocksUpdatePacked {
    pub const fn position(self) -> u64 {
        self.0 & 4095
    }

    pub const fn state(self) -> u64 {
        self.0 >> 12
    }

    pub const fn new(position: u64, state: u64) -> Self {
        Self((state << 12) | position)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SelectAdvancementsTab<'a> {
    pub tab: Option<Ident<'a>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ServerData<'a> {
    pub motd: Component,
    pub icon_bytes: Option<ByteArray<'a>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SetActionBarText {
    pub text: Component,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SetBorderCenter {
    pub new_center_x: f64,
    pub new_center_z: f64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SetBorderLerpSize {
    pub old_size: f64,
    pub new_size: f64,
    #[mser(varint)]
    pub lerp_time: u64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SetBorderSize {
    pub size: f64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SetBorderWarningDelay {
    #[mser(varint)]
    pub warning_delay: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SetBorderWarningDistance {
    #[mser(varint)]
    pub warning_blocks: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SetCamera {
    #[mser(varint)]
    pub camera_id: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SetChunkCacheCenter {
    #[mser(varint)]
    pub x: i32,
    #[mser(varint)]
    pub z: i32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SetChunkCacheRadius {
    #[mser(varint)]
    pub radius: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SetCursorItem<'a> {
    pub contents: OptionalItemStack<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SetDefaultSpawnPosition<'a> {
    pub respawn_data: RespawnData<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SetDisplayObjective<'a> {
    pub slot: DisplaySlot,
    pub objective_name: Utf8<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SetEntityData<'a> {
    #[mser(varint)]
    pub id: u32,
    pub packed_items: SetEntityDataValues<'a>,
}

#[derive(Clone)]
pub struct SetEntityDataValues<'a>(pub List<'a, (u8, EntityDataSerializer<'a>)>);

impl<'a> Read<'a> for SetEntityDataValues<'a> {
    fn read(buf: &mut Reader<'a>) -> Result<Self, Error> {
        let mut vec = Vec::new();
        loop {
            let id = u8::read(buf)?;
            if id == 255 {
                break;
            }
            let value = EntityDataSerializer::read(buf)?;
            vec.push((id, value));
        }
        Ok(Self(List::Owned(vec)))
    }
}

impl<'a> Write for SetEntityDataValues<'a> {
    unsafe fn write(&self, w: &mut Writer) {
        unsafe {
            for (id, val) in self.0.as_slice() {
                id.write(w);
                val.write(w);
            }
            255u8.write(w);
        }
    }

    fn len_s(&self) -> usize {
        let mut l = 0;
        for (id, val) in self.0.as_slice() {
            l += id.len_s() + val.len_s();
        }
        l + 1
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SetEntityLink {
    pub source_id: u32,
    pub dest_id: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SetEntityMotion {
    #[mser(varint)]
    pub id: u32,
    pub movement: LpVec3,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SetEquipment<'a> {
    pub entity: u32,
    pub slots: SetEquipmentSlots<'a>,
}

#[derive(Clone)]
pub struct SetEquipmentSlots<'a>(pub List<'a, (EquipmentSlot, OptionalItemStack<'a>)>);

impl<'a> Read<'a> for SetEquipmentSlots<'a> {
    fn read(buf: &mut Reader<'a>) -> Result<Self, Error> {
        let mut vec = Vec::new();
        loop {
            let id = buf.read_byte()?;
            let slot = EquipmentSlot::new(id & 127);
            let stack = OptionalItemStack::read(buf)?;
            vec.push((slot, stack));
            if id & 128 == 0 {
                break;
            }
        }
        Ok(Self(List::Owned(vec)))
    }
}

impl<'a> Write for SetEquipmentSlots<'a> {
    unsafe fn write(&self, w: &mut Writer) {
        unsafe {
            let mut iter = self.0.as_slice().iter();
            let mut curr = match iter.next() {
                Some(x) => x,
                None => return,
            };
            loop {
                let next = iter.next();
                let (slot, stack) = curr;
                let c = if next.is_none() { 0 } else { 128 };
                w.write_byte((*slot as u8) | c);
                stack.write(w);
                curr = match next {
                    Some(x) => x,
                    None => return,
                };
            }
        }
    }

    fn len_s(&self) -> usize {
        let mut l = 0;
        for (_, stack) in self.0.as_slice() {
            l += 1;
            l += stack.len_s();
        }
        l
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SetExperience {
    pub experience_progress: f32,
    #[mser(varint)]
    pub total_experience: u32,
    #[mser(varint)]
    pub experience_level: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SetHealth {
    pub health: f32,
    #[mser(varint)]
    pub food: u32,
    pub saturation: f32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SetHeldSlot {
    #[mser(varint)]
    pub slot: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SetObjective<'a> {
    pub objective_name: Utf8<'a>,
    pub method: SetObjectiveMethod,
}

#[derive(Clone, Serialize, Deserialize)]
#[mser(header = SetObjectiveMethodType, camel_case)]
pub enum SetObjectiveMethod {
    Add {
        display_name: Component,
        render_type: ObjectiveCriteriaRenderType,
        number_format: Option<number_format_type>,
    },
    Remove,
    Change {
        display_name: Component,
        render_type: ObjectiveCriteriaRenderType,
        number_format: Option<number_format_type>,
    },
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
pub enum SetObjectiveMethodType {
    Add,
    Remove,
    Change,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SetPassengers<'a> {
    #[mser(varint)]
    pub vehicle: u32,
    pub passengers: V32List<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SetPlayerInventory<'a> {
    #[mser(varint)]
    pub slot: u32,
    pub contents: OptionalItemStack<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SetPlayerTeam<'a> {
    pub name: Utf8<'a>,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
pub enum SetPlayerTeamMethodType {
    Add,
    Remove,
    Change,
    Join,
    Leave,
}

#[derive(Clone, Serialize, Deserialize)]
#[mser(header = SetPlayerTeamMethodType, camel_case)]
pub enum SetPlayerTeamMethod<'a> {
    Add {
        parameters: SetPlayerTeamParameters,
        players: List<'a, Utf8<'a>>,
    },
    Remove,
    Change {
        parameters: SetPlayerTeamParameters,
    },
    Join {
        players: List<'a, Utf8<'a>>,
    },
    Leave {
        players: List<'a, Utf8<'a>>,
    },
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SetPlayerTeamParameters {
    pub display_name: Component,
    pub options: u8,
    pub nametag_visibility: TeamVisibility,
    pub collision_rule: TeamCollisionRule,
    pub color: ChatFormatting,
    pub player_prefix: Component,
    pub player_suffix: Component,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SetScore<'a> {
    pub owner: Utf8<'a>,
    pub objective_name: Utf8<'a>,
    #[mser(varint)]
    pub score: u32,
    pub display: Option<Component>,
    pub number_format: Option<number_format_type>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SetSimulationDistance {
    #[mser(varint)]
    pub simulation_distance: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SetSubtitleText {
    pub text: Component,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SetTime {
    pub game_time: u64,
    pub day_time: u64,
    pub tick_day_time: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SetTitleText {
    pub text: Component,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SetTitlesAnimation {
    pub fade_in: u32,
    pub stay: u32,
    pub fade_out: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SoundEntity<'a> {
    pub sound: Holder<SoundEvent<'a>, sound_event>,
    pub source: SoundSource,
    #[mser(varint)]
    pub id: u32,
    pub volume: f32,
    pub pitch: f32,
    pub seed: u64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Sound<'a> {
    pub sound: Holder<SoundEvent<'a>, sound_event>,
    pub source: SoundSource,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub volume: f32,
    pub pitch: f32,
    pub seed: u64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct StartConfiguration {}
