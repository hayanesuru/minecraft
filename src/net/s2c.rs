use crate::command::CommandNode;
use crate::entity::TrackedData;
use crate::inventory::ItemStack;
use crate::math::{BlockPos, ChunkSectionPos, GlobalPos, Position};
use crate::nbt::{Compound, COMPOUND};
use crate::recipe::Recipe;
use crate::text::{Literal, Translate};
use crate::{
    block, block_entity_type, block_state, configuration_s2c, entity_type, item, login_s2c,
    play_s2c as C, Difficulty, GameMode, GameProfile, Identifier, UnsafeWriter, Write, V21, V32,
};
use uuid::Uuid;

#[derive(Writable, Clone, Copy)]
#[ser(prefix = login_s2c::LoginDisconnect)]
pub struct LoginDisconnect<'a> {
    pub reason: &'a str,
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = login_s2c::LoginCompression)]
pub struct LoginCompression {
    #[ser(varint)]
    pub compression_threshold: u32,
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = login_s2c::LoginSuccess)]
pub struct LoginSuccess<'a> {
    #[ser(varint)]
    pub profile: &'a GameProfile,
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = login_s2c::LoginHello)]
pub struct LoginHello<'a> {
    pub server_id: &'a str,
    pub public_key: &'a [u8],
    pub nonce: &'a [u8],
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = configuration_s2c::CustomPayload)]
pub struct ConfCustomPayload<'a> {
    pub id: &'a str,
    #[ser(head = none)]
    pub data: &'a [u8],
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = configuration_s2c::Disconnect)]
pub struct ConfDisconnect<'a> {
    pub reason: Literal<'a>,
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = configuration_s2c::Ready)]
pub struct ConfReady;

#[derive(Writable, Clone, Copy)]
#[ser(prefix = configuration_s2c::KeepAlive)]
pub struct ConfKeepAlive {
    pub id: u64,
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = configuration_s2c::CommonPing)]
pub struct ConfCommonPing {
    pub id: u32,
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = configuration_s2c::DynamicRegistries)]
pub struct ConfDynamicRegistries<'a> {
    #[ser(add = COMPOUND)]
    #[ser(head = none)]
    pub nbt: &'a [u8],
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = configuration_s2c::ResourcePackSend)]
pub struct ConfResourcePackSend<'a> {
    pub url: &'a str,
    pub hash: &'a str,
    pub required: bool,
    pub prompt: Option<&'a str>,
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = configuration_s2c::Features)]
pub struct ConfFeatures<'a> {
    pub flag_ids: &'a [&'a str],
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = configuration_s2c::SynchronizeTags)]
pub struct ConfSynchronizeTags<'a> {
    pub groups: TagsGroups<'a>,
}

#[derive(Clone, Copy)]
pub struct TagsGroups<'a>(pub &'a [(&'a str, TagsGroup<'a>)]);

impl Write for TagsGroups<'_> {
    #[inline]
    fn write(&self, w: &mut UnsafeWriter) {
        V21(self.0.len() as u32).write(w);
        self.0.iter().copied().for_each(|(name, group)| {
            V21(name.len() as u32).write(w);
            name.write(w);
            group.write(w);
        });
    }

    #[inline]
    fn len(&self) -> usize {
        let a = V21(self.0.len() as u32).len();
        let b = self
            .0
            .iter()
            .copied()
            .map(|(name, group)| V21(name.len() as u32).len() + name.len() + group.len())
            .sum::<usize>();
        a + b
    }
}

impl Write for TagsGroup<'_> {
    #[inline]
    fn write(&self, w: &mut UnsafeWriter) {
        V21(self.0.len() as u32).write(w);
        self.0.iter().copied().for_each(|(name, val)| {
            V21(name.len() as u32).write(w);
            name.write(w);
            V21(val.len() as u32).write(w);
            val.iter().copied().for_each(|x| V32(x).write(w));
        });
    }

    #[inline]
    fn len(&self) -> usize {
        let a = V21(self.0.len() as u32).len();
        let b = self
            .0
            .iter()
            .copied()
            .map(|(name, val)| {
                V21(name.len() as u32).len()
                    + name.len()
                    + val.iter().copied().map(|x| V32(x).len()).sum::<usize>()
                    + V21(val.len() as u32).len()
            })
            .sum::<usize>();
        a + b
    }
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::BundleSplitter)]
pub struct BundleSplitter;

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::PingResult)]
pub struct PingResult {
    pub start: u64,
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::CustomPayload)]
pub struct CustomPayload<'a> {
    pub id: &'a str,
    #[ser(head = none)]
    pub data: &'a [u8],
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::Disconnect)]
pub struct Disconnect<'a> {
    pub reason: Literal<'a>,
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::Disconnect)]
pub struct DisconnectTranslate<'a> {
    pub reason: Translate<'a>,
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::KeepAlive)]
pub struct KeepAlive {
    pub id: u64,
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::CommonPing)]
pub struct CommonPing {
    pub id: u32,
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::ResourcePackSend)]
pub struct ResourcePackSend<'a> {
    pub url: &'a str,
    pub hash: &'a str,
    pub required: bool,
    pub prompt: Option<&'a str>,
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::SynchronizeTags)]
pub struct SynchronizeTags<'a> {
    pub groups: TagsGroups<'a>,
}

#[derive(Clone, Copy)]
pub struct TagsGroup<'a>(pub &'a [(&'a str, &'a [u32])]);

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::BlockBreakingProgress)]
pub struct BlockBreakingProgress {
    #[ser(varint)]
    entity_id: u32,
    pos: BlockPos,
    progress: u8,
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::BlockEntityUpdate)]
pub struct BlockEntityUpdate<'a> {
    pub pos: BlockPos,
    pub r#type: block_entity_type,
    #[ser(add = COMPOUND)]
    pub nbt: &'a Compound,
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::BlockEvent)]
pub struct BlockEvent {
    pub pos: BlockPos,
    pub r#type: u8,
    pub data: u8,
    pub id: block,
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::PlayerActionResponse)]
pub struct PlayerActionResponse {
    #[ser(varint)]
    pub sequence: u32,
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::BlockUpdate)]
pub struct BlockUpdate {
    pub pos: BlockPos,
    pub state: block_state,
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::SetCameraEntity)]
pub struct SetCameraEntity {
    #[ser(varint)]
    pub entity_id: u32,
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::ChunkSent)]
pub struct ChunkSent {
    #[ser(varint)]
    pub batch_size: u32,
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::ChunkData)]
pub struct ChunkData<'a> {
    pub chunk_x: i32,
    pub chunk_z: i32,
    #[ser(add = COMPOUND)]
    #[ser(head = none)]
    pub heightmaps: &'a [u8],
    pub chunk: crate::chunk::ChunkData<'a>,
    pub light: crate::chunk::LightData<'a>,
}

#[derive(Clone, Copy)]
pub struct ChunkDeltaUpdate<'a> {
    pub section: ChunkSectionPos,
    pub changes: &'a [u32],
}

impl Write for ChunkDeltaUpdate<'_> {
    #[inline]
    fn write(&self, w: &mut UnsafeWriter) {
        C::ChunkDeltaUpdate.write(w);
        self.section.write(w);
        V21(self.changes.len() as u32).write(w);
        for &x in self.changes {
            V32(x).write(w);
        }
    }

    #[inline]
    fn len(&self) -> usize {
        C::ChunkDeltaUpdate.len()
            + self.section.len()
            + V21(self.changes.len() as _).len()
            + self.changes.iter().map(|&x| V32(x).len()).sum::<usize>()
    }
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::ChunkLoadDistance)]
pub struct ChunkLoadDistance {
    #[ser(varint)]
    pub distance: u32,
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::ChunkRenderDistanceCenter)]
pub struct ChunkRenderDistanceCenter {
    #[ser(varint)]
    pub chunk_x: i32,
    #[ser(varint)]
    pub chunk_z: i32,
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::UnloadChunk)]
pub struct UnloadChunk {
    pub x: i32,
    pub z: i32,
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::CommandTree)]
pub struct CommandTree<'a> {
    #[ser(expand)]
    pub nodes: &'a [CommandNode<'a>],
    #[ser(varint)]
    pub root_index: u32,
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::CooldownUpdate)]
pub struct CooldownUpdate {
    pub item: item,
    #[ser(varint)]
    pub cooldown: u32,
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::CraftFailedResponse)]
pub struct CraftFailedResponse<'a> {
    pub sync_id: u8,
    pub recipe_id: Identifier<'a>,
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::ChatSuggestions)]
pub struct ChatSuggestions<'a> {
    /// Add Remove Set
    pub action: u8,
    pub names: &'a [&'a str],
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::DamageTilt)]
pub struct DamageTilt {
    #[ser(varint)]
    pub entity_id: u32,
    pub yaw: f32,
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::EntityDamage)]
pub struct EntityDamage {
    #[ser(varint)]
    pub entity_id: u32,
    #[ser(varint)]
    pub source_type_id: u32,
    #[ser(varint)]
    pub source_cause_id: u32,
    #[ser(varint)]
    pub source_direct_id: u32,
    pub source_position: Option<Position>,
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::DeathMessage)]
pub struct DeathMessage<'a> {
    #[ser(varint)]
    pub entity_id: u32,
    pub message: Literal<'a>,
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::Difficulty)]
pub struct UpdateDifficulty {
    pub difficulty: Difficulty,
    pub difficulty_locked: bool,
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::EndCombat)]
pub struct EndCombat {
    #[ser(varint)]
    pub time_since_last_attack: u32,
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::EnterCombat)]
pub struct EnterCombat;

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::EntityAnimation)]
pub struct EntityAnimation {
    #[ser(varint)]
    pub entity_id: u32,
    pub animation_id: u8,
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::EntityAttach)]
pub struct EntityAttach {
    pub attached_id: u32,
    pub holding_id: u32,
}

#[derive(Clone, Copy)]
pub struct EntitiesDestroy<'a> {
    pub entity_ids: &'a [u32],
}

impl Write for EntitiesDestroy<'_> {
    #[inline]
    fn write(&self, w: &mut UnsafeWriter) {
        C::EntitiesDestroy.write(w);
        V21(self.entity_ids.len() as u32).write(w);
        for &x in self.entity_ids {
            V32(x).write(w);
        }
    }

    #[inline]
    fn len(&self) -> usize {
        C::EntitiesDestroy.len()
            + V21(self.entity_ids.len() as u32).len()
            + self.entity_ids.iter().map(|&x| V32(x).len()).sum::<usize>()
    }
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::EntityStatus)]
pub struct EntityStatus {
    pub entity_id: u32,
    pub status: u8,
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::EntitySetHeadYaw)]
pub struct EntitySetHeadYaw {
    #[ser(varint)]
    pub entity_id: u32,
    pub head_yaw: i8,
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::EntityPosition)]
pub struct EntityPosition {
    #[ser(varint)]
    pub entity_id: u32,
    pub pos: Position,
    pub yaw: i8,
    pub pitch: i8,
    pub on_ground: bool,
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::EntitySpawn)]
pub struct EntitySpawn {
    #[ser(varint)]
    pub entity_id: u32,
    pub uuid: Uuid,
    pub entity_type_id: entity_type,
    pub pos: Position,
    pub pitch: i8,
    pub yaw: i8,
    pub head_yaw: i8,
    #[ser(varint)]
    pub entity_data: u32,
    pub velocity_x: i16,
    pub velocity_y: i16,
    pub velocity_z: i16,
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::EntityTrackerUpdate)]
pub struct EntityTrackerUpdate<'a> {
    #[ser(varint)]
    pub entity_id: u32,
    pub data: EntityTrackerUpdateData<'a>,
}

#[derive(Clone, Copy)]
pub struct EntityTrackerUpdateData<'a>(pub &'a [TrackedData]);

impl<'a> Write for EntityTrackerUpdateData<'a> {
    fn write(&self, w: &mut UnsafeWriter) {
        for val in self.0 {
            val.write(w);
        }
        w.write_byte(0xff);
    }

    fn len(&self) -> usize {
        let mut l = 1;
        for val in self.0 {
            l += val.len();
        }
        l
    }
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::EntityMoveRelative)]
pub struct EntityMoveRelative {
    #[ser(varint)]
    pub entity_id: u32,
    pub delta_x: i16,
    pub delta_y: i16,
    pub delta_z: i16,
    pub on_ground: bool,
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::EntityRotateAndMoveRelative)]
pub struct EntityRotateAndMoveRelative {
    #[ser(varint)]
    pub entity_id: u32,
    pub delta_x: i16,
    pub delta_y: i16,
    pub delta_z: i16,
    pub yaw: i8,
    pub pitch: i8,
    pub on_ground: bool,
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::EntityRotate)]
pub struct EntityRotate {
    #[ser(varint)]
    pub entity_id: u32,
    pub yaw: i8,
    pub pitch: i8,
    pub on_ground: bool,
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::CloseScreen)]
pub struct CloseScreen {
    pub sync_id: u8,
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::ScreenHandlerPropertyUpdate)]
pub struct ScreenHandlerPropertyUpdate {
    pub sync_id: u8,
    pub property_id: u16,
    pub value: u16,
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::ScreenHandlerSlotUpdate)]
pub struct ScreenHandlerSlotUpdate {
    pub sync_id: u8,
    #[ser(varint)]
    pub revision: u32,
    pub slot: u16,
    pub stack: ItemStack,
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::GameJoin)]
pub struct GameJoin<'a> {
    pub player_entity_id: u32,
    pub hardcore: bool,
    #[ser(expand)]
    pub dimension_ids: &'a [Identifier<'a>],
    #[ser(varint)]
    pub max_players: u32,
    #[ser(varint)]
    pub view_distance: u32,
    #[ser(varint)]
    pub simulation_distance: u32,
    pub reduced_debug_info: bool,
    pub show_death_screen: bool,
    pub do_limited_crafting: bool,
    pub common_player_spawn_info: CommonPlayerSpawnInfo<'a>,
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::HealthUpdate)]
pub struct HealthUpdate {
    pub health: f32,
    #[ser(varint)]
    pub food: u32,
    pub saturation: f32,
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::Inventory)]
pub struct Inventory<'a> {
    pub sync_id: u8,
    #[ser(varint)]
    pub revision: u32,
    #[ser(expand)]
    pub contents: &'a [ItemStack],
    pub cursor_stack: ItemStack,
}

#[derive(Writable, Clone, Copy)]
pub struct CommonPlayerSpawnInfo<'a> {
    pub dimension_type: Identifier<'a>,
    pub dimension: Identifier<'a>,
    pub seed: u64,
    pub game_type: GameMode,
    pub previous_game_type: u8,
    pub is_debug: bool,
    pub is_flat: bool,
    pub last_death_location: Option<GlobalPos<'a>>,
    #[ser(varint)]
    pub portal_cooldown: u32,
}

#[derive(Writable, Clone, Copy)]
pub struct CommonPlayerSpawnInfo2<'a> {
    pub dimension_type: &'a [u8],
    pub dimension: &'a [u8],
    pub seed: u64,
    pub game_type: GameMode,
    pub previous_game_type: u8,
    pub is_debug: bool,
    pub is_flat: bool,
    pub last_death_location: Option<GlobalPos<'a>>,
    #[ser(varint)]
    pub portal_cooldown: u32,
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::GameStateChange)]
pub struct GameStateChange {
    pub reason: GameStateChangeReason,
    pub value: f32,
}

#[repr(u8)]
#[derive(Writable, Clone, Copy, PartialEq, Eq)]
pub enum GameStateChangeReason {
    NoRespawnBlock,
    RainStarted,
    RainStopped,
    GameModeChanged,
    GameWon,
    DemoMessageShown,
    ProjectileHitPlayer,
    RainGradientChanged,
    ThunderGradientChanged,
    PufferfishSting,
    ElderGuardianEffect,
    ImmediateRespawn,
    LimitedCrafting,
    Ready,
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::ServerMetadata)]
pub struct ServerMetadata<'a> {
    pub description: Literal<'a>,
    pub favicon: Option<&'a [u8]>,
    pub secure_chat_enfored: bool,
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::OverlayMessage)]
pub struct OverlayMessage<'a> {
    pub message: Literal<'a>,
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::PlayerAbilities)]
pub struct PlayerAbilities {
    /// 0 0 0 0 creative_mode allow_flying flying invulnerable
    pub flags: u8,
    pub fly_speed: f32,
    pub walk_speed: f32,
}

#[derive(Clone, Copy)]
pub struct PlayerList<'a> {
    pub entries: &'a [PlayerListEntry],
}

#[derive(Clone, Copy)]
pub struct PlayerListEntry {
    pub profile: *const GameProfile,
    pub game_mode: GameMode,
    pub listed: bool,
    pub latency: u32,
    pub display_name: Option<*const str>,
}

impl Write for PlayerList<'_> {
    #[inline]
    fn write(&self, w: &mut UnsafeWriter) {
        C::PlayerList.write(w);
        w.write_byte(0b111111);
        V21(self.entries.len() as u32).write(w);
        for entry in self.entries {
            unsafe { (*entry.profile).write(w) }
            false.write(w);
            entry.game_mode.write(w);
            entry.listed.write(w);
            V32(entry.latency).write(w);
            match entry.display_name {
                None => false.write(w),
                Some(x) => unsafe {
                    true.write(w);
                    V21((*x).len() as u32).write(w);
                    w.write((*x).as_bytes());
                },
            }
        }
    }

    #[inline]
    fn len(&self) -> usize {
        let mut l = C::PlayerList.len();
        l += 1;
        l += V21(self.entries.len() as u32).len();
        for entry in self.entries {
            unsafe { l += (*entry.profile).len() }
            l += 1;
            l += entry.game_mode.len();
            l += 1;
            l += V32(entry.latency).len();
            l += 1;
            match entry.display_name {
                None => {}
                Some(x) => unsafe {
                    l += V21((*x).len() as u32).len();
                    l += (*x).len();
                },
            }
        }
        l
    }
}

#[derive(Clone, Copy)]
pub struct PlayerListLatency<'a> {
    pub entries: &'a [PlayerListLatencyEntry],
}

#[derive(Clone, Copy)]
pub struct PlayerListLatencyEntry {
    pub uuid: Uuid,
    pub latency: u32,
}

impl Write for PlayerListLatency<'_> {
    #[inline]
    fn write(&self, w: &mut UnsafeWriter) {
        C::PlayerList.write(w);
        w.write_byte(0b10000);
        V21(self.entries.len() as u32).write(w);
        for entry in self.entries {
            entry.uuid.write(w);
            V32(entry.latency).write(w);
        }
    }

    #[inline]
    fn len(&self) -> usize {
        let mut l = C::PlayerList.len();
        l += 1;
        l += V21(self.entries.len() as u32).len();
        for entry in self.entries {
            l += 16;
            l += V32(entry.latency).len();
        }
        l
    }
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::PlayerPositionLook)]
pub struct PlayerPositionLook {
    pub pos: Position,
    pub yaw: f32,
    pub pitch: f32,
    pub flags: u8,
    #[ser(varint)]
    pub teleport_id: u32,
}

#[derive(Clone, Copy)]
pub struct PlayerRemove<'a> {
    pub profile_ids: &'a [Uuid],
}

impl Write for PlayerRemove<'_> {
    #[inline]
    fn write(&self, w: &mut UnsafeWriter) {
        C::PlayerRemove.write(w);
        V21(self.profile_ids.len() as u32).write(w);
        for id in self.profile_ids {
            id.write(w);
        }
    }

    #[inline]
    fn len(&self) -> usize {
        C::PlayerRemove.len()
            + V21(self.profile_ids.len() as u32).len()
            + self.profile_ids.len() * 16
    }
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::PlayerRespawn)]
pub struct PlayerRespawn<'a> {
    pub common_player_spawn_info: CommonPlayerSpawnInfo<'a>,
    pub kept_data_flag: u8,
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::PlayerRespawn)]
pub struct PlayerRespawn2<'a> {
    pub common_player_spawn_info: CommonPlayerSpawnInfo2<'a>,
    pub kept_data_flag: u8,
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::PlayerSpawnPosition)]
pub struct PlayerSpawnPosition {
    pub pos: BlockPos,
    pub angle: f32,
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::UnlockRecipes)]
pub struct UnlockRecipes<'a> {
    pub action: RecipesUnlockAction,
    pub crafing_gui_open: bool,
    pub crafing_filtering_craftable: bool,
    pub furnace_gui_open: bool,
    pub furnace_filtering_craftable: bool,
    pub blast_furnace_gui_open: bool,
    pub blast_furnace_filtering_craftable: bool,
    pub smoker_gui_open: bool,
    pub smoker_filtering_craftable: bool,
    #[ser(expand)]
    pub recipe_ids_to_change: &'a [Identifier<'a>],
    pub recipe_ids_to_init: RecipesUnlockIdsToInit<'a>,
}

#[derive(Copy, Clone)]
pub struct RecipesUnlockIdsToInit<'a>(pub Option<&'a [Identifier<'a>]>);

impl Write for RecipesUnlockIdsToInit<'_> {
    fn write(&self, w: &mut UnsafeWriter) {
        if let Some(x) = self.0 {
            V21(x.len() as u32).write(w);
            for y in x {
                y.write(w);
            }
        }
    }

    fn len(&self) -> usize {
        match self.0 {
            Some(x) => V21(x.len() as u32).len() + x.iter().map(|x| x.len()).sum::<usize>(),
            None => 0,
        }
    }
}

#[derive(Writable, PartialEq, Eq, Copy, Clone)]
#[repr(u8)]
pub enum RecipesUnlockAction {
    Init,
    Add,
    Remove,
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::SynchronizeRecipes)]
pub struct SynchronizeRecipes<'a> {
    #[ser(expand)]
    pub recipes: &'a [Recipe<'a>],
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::EnterReconfiguration)]
pub struct EnterReconfiguration;

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::Subtitle)]
pub struct Subtitle<'a> {
    pub subtitle: Literal<'a>,
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::GameMessage)]
pub struct GameMessage<'a> {
    pub content: Literal<'a>,
    pub overlay: bool,
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::GameMessage)]
pub struct GameMessageTranslate<'a> {
    pub content: Translate<'a>,
    pub overlay: bool,
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::StartChunkSend)]
pub struct StartChunkSend;

#[derive(Writable, Clone, Copy)]
#[ser(prefix = C::WorldTimeUpdate)]
pub struct WorldTimeUpdate {
    pub time: u64,
    pub time_of_day: u64,
}
