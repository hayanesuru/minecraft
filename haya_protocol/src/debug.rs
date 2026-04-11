use crate::path::Path;
use crate::redstone::Orientation;
use crate::structure::{BoundingBoxPacked, PiecePacked};
use haya_collection::List;
use haya_math::{BlockPosPacked, Vec3};
use minecraft_data::{block, debug_subscription, game_event, point_of_interest_type};
use mser::Utf8;

#[derive(Clone, Serialize, Deserialize)]
#[mser(header = debug_subscription)]
pub enum DebugSubscriptionUpdate<'a> {
    DedicatedServerTickTime,
    Bees(Option<DebugBeeInfo<'a>>),
    Brains(Option<DebugBrainDump<'a>>),
    Breezes(Option<DebugBreezeInfo>),
    GoalSelectors(Option<DebugGoalInfo<'a>>),
    EntityPaths(Option<DebugPathInfo<'a>>),
    EntityBlockIntersections(Option<DebugEntityBlockIntersection>),
    BeeHives(Option<DebugHiveInfo>),
    Pois(Option<DebugPoiInfo>),
    RedstoneWireOrientations(Option<Orientation>),
    VillageSections(Option<DebugVillageSection>),
    Raids(Option<List<'a, BlockPosPacked>>),
    Structures(Option<List<'a, DebugStructureInfo<'a>>>),
    GameEventListeners(Option<DebugGameEventListenerInfo>),
    NeighborUpdates(Option<BlockPosPacked>),
    GameEvents(Option<DebugGameEventInfo>),
}

#[derive(Clone, Serialize, Deserialize)]
#[mser(header = debug_subscription)]
pub enum DebugSubscriptionEvent<'a> {
    DedicatedServerTickTime,
    Bees(DebugBeeInfo<'a>),
    Brains(DebugBrainDump<'a>),
    Breezes(DebugBreezeInfo),
    GoalSelectors(DebugGoalInfo<'a>),
    EntityPaths(DebugPathInfo<'a>),
    EntityBlockIntersections(DebugEntityBlockIntersection),
    BeeHives(DebugHiveInfo),
    Pois(DebugPoiInfo),
    RedstoneWireOrientations(Orientation),
    VillageSections(DebugVillageSection),
    Raids(List<'a, BlockPosPacked>),
    Structures(List<'a, DebugStructureInfo<'a>>),
    GameEventListeners(DebugGameEventListenerInfo),
    NeighborUpdates(BlockPosPacked),
    GameEvents(DebugGameEventInfo),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DebugBeeInfo<'a> {
    pub hive_pos: Option<BlockPosPacked>,
    pub flower_pos: Option<BlockPosPacked>,
    #[mser(varint)]
    pub travel_ticks: u32,
    pub blacklisted_hives: List<'a, BlockPosPacked>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DebugBrainDump<'a> {
    pub name: Utf8<'a>,
    pub profession: Utf8<'a>,
    pub xp: u32,
    pub health: f32,
    pub max_health: f32,
    pub inventory: Utf8<'a>,
    pub wants_golem: bool,
    pub anger_level: u32,
    pub activities: List<'a, Utf8<'a>>,
    pub behaviors: List<'a, Utf8<'a>>,
    pub memories: List<'a, Utf8<'a>>,
    pub gossips: List<'a, Utf8<'a>>,
    pub pois: List<'a, BlockPosPacked>,
    pub potential_pois: List<'a, BlockPosPacked>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DebugBreezeInfo {
    pub attack_target: Option<AttackTarget>,
    pub jump_target: Option<BlockPosPacked>,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct AttackTarget(#[mser(varint)] pub u32);

#[derive(Clone, Serialize, Deserialize)]
pub struct DebugGoalInfo<'a> {
    pub goals: List<'a, DebugGoal<'a>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DebugGoal<'a> {
    #[mser(varint)]
    pub priority: u32,
    pub is_running: bool,
    pub name: Utf8<'a, 255>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DebugPathInfo<'a> {
    pub path: Path<'a>,
    pub max_node_distance: f32,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum DebugEntityBlockIntersection {
    InBlock,
    InFluid,
    InAir,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DebugHiveInfo {
    pub ty: block,
    #[mser(varint)]
    pub occupant_count: u32,
    #[mser(varint)]
    pub honey_level: u32,
    pub sedated: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DebugPoiInfo {
    pub pos: BlockPosPacked,
    pub poi_type: point_of_interest_type,
    #[mser(varint)]
    pub free_ticket_count: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DebugStructureInfo<'a> {
    pub bounding_box: BoundingBoxPacked,
    pub pieces: List<'a, PiecePacked>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DebugGameEventListenerInfo {
    #[mser(varint)]
    pub listener_radius: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DebugGameEventInfo {
    pub event: game_event,
    pub pos: Vec3,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DebugVillageSection {}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum RemoteDebugSampleType {
    TickTime,
}
