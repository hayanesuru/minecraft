use crate::path::Path;
use crate::redstone::Orientation;
use crate::structure::{BoundingBoxPacked, PiecePacked};
use haya_collection::List;
use haya_math::{BlockPosPacked, Vec3};
use minecraft_data::{block, debug_subscription, game_event, point_of_interest_type};
use mser::{Error, Read, Reader, Utf8, Write, Writer};

#[derive(Clone)]
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

impl<'a> DebugSubscriptionUpdate<'a> {
    pub const fn id(&self) -> debug_subscription {
        match self {
            Self::DedicatedServerTickTime { .. } => debug_subscription::dedicated_server_tick_time,
            Self::Bees { .. } => debug_subscription::bees,
            Self::Brains { .. } => debug_subscription::brains,
            Self::Breezes { .. } => debug_subscription::breezes,
            Self::GoalSelectors { .. } => debug_subscription::goal_selectors,
            Self::EntityPaths { .. } => debug_subscription::entity_paths,
            Self::EntityBlockIntersections { .. } => debug_subscription::entity_block_intersections,
            Self::BeeHives { .. } => debug_subscription::bee_hives,
            Self::Pois { .. } => debug_subscription::pois,
            Self::RedstoneWireOrientations { .. } => debug_subscription::redstone_wire_orientations,
            Self::VillageSections { .. } => debug_subscription::village_sections,
            Self::Raids { .. } => debug_subscription::raids,
            Self::Structures { .. } => debug_subscription::structures,
            Self::GameEventListeners { .. } => debug_subscription::game_event_listeners,
            Self::NeighborUpdates { .. } => debug_subscription::neighbor_updates,
            Self::GameEvents { .. } => debug_subscription::game_events,
        }
    }
}

impl<'a> Write for DebugSubscriptionUpdate<'a> {
    unsafe fn write(&self, w: &mut Writer) {
        unsafe {
            self.id().write(w);
            match self {
                Self::DedicatedServerTickTime => (),
                Self::Bees(x) => x.write(w),
                Self::Brains(x) => x.write(w),
                Self::Breezes(x) => x.write(w),
                Self::GoalSelectors(x) => x.write(w),
                Self::EntityPaths(x) => x.write(w),
                Self::EntityBlockIntersections(x) => x.write(w),
                Self::BeeHives(x) => x.write(w),
                Self::Pois(x) => x.write(w),
                Self::RedstoneWireOrientations(x) => x.write(w),
                Self::VillageSections(x) => x.write(w),
                Self::Raids(x) => x.write(w),
                Self::Structures(x) => x.write(w),
                Self::GameEventListeners(x) => x.write(w),
                Self::NeighborUpdates(x) => x.write(w),
                Self::GameEvents(x) => x.write(w),
            }
        }
    }

    fn len_s(&self) -> usize {
        self.id().len_s()
            + match self {
                Self::DedicatedServerTickTime => 0,
                Self::Bees(x) => x.len_s(),
                Self::Brains(x) => x.len_s(),
                Self::Breezes(x) => x.len_s(),
                Self::GoalSelectors(x) => x.len_s(),
                Self::EntityPaths(x) => x.len_s(),
                Self::EntityBlockIntersections(x) => x.len_s(),
                Self::BeeHives(x) => x.len_s(),
                Self::Pois(x) => x.len_s(),
                Self::RedstoneWireOrientations(x) => x.len_s(),
                Self::VillageSections(x) => x.len_s(),
                Self::Raids(x) => x.len_s(),
                Self::Structures(x) => x.len_s(),
                Self::GameEventListeners(x) => x.len_s(),
                Self::NeighborUpdates(x) => x.len_s(),
                Self::GameEvents(x) => x.len_s(),
            }
    }
}

impl<'a> Read<'a> for DebugSubscriptionUpdate<'a> {
    fn read(buf: &mut Reader<'a>) -> Result<Self, Error> {
        Ok(match debug_subscription::read(buf)? {
            debug_subscription::dedicated_server_tick_time => Self::DedicatedServerTickTime,
            debug_subscription::bees => Self::Bees(Read::read(buf)?),
            debug_subscription::brains => Self::Brains(Read::read(buf)?),
            debug_subscription::breezes => Self::Breezes(Read::read(buf)?),
            debug_subscription::goal_selectors => Self::GoalSelectors(Read::read(buf)?),
            debug_subscription::entity_paths => Self::EntityPaths(Read::read(buf)?),
            debug_subscription::entity_block_intersections => {
                Self::EntityBlockIntersections(Read::read(buf)?)
            }
            debug_subscription::bee_hives => Self::BeeHives(Read::read(buf)?),
            debug_subscription::pois => Self::Pois(Read::read(buf)?),
            debug_subscription::redstone_wire_orientations => {
                Self::RedstoneWireOrientations(Read::read(buf)?)
            }
            debug_subscription::village_sections => Self::VillageSections(Read::read(buf)?),
            debug_subscription::raids => Self::Raids(Read::read(buf)?),
            debug_subscription::structures => Self::Structures(Read::read(buf)?),
            debug_subscription::game_event_listeners => Self::GameEventListeners(Read::read(buf)?),
            debug_subscription::neighbor_updates => Self::NeighborUpdates(Read::read(buf)?),
            debug_subscription::game_events => Self::GameEvents(Read::read(buf)?),
        })
    }
}

#[derive(Clone)]
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

impl<'a> DebugSubscriptionEvent<'a> {
    pub const fn id(&self) -> debug_subscription {
        match self {
            Self::DedicatedServerTickTime { .. } => debug_subscription::dedicated_server_tick_time,
            Self::Bees { .. } => debug_subscription::bees,
            Self::Brains { .. } => debug_subscription::brains,
            Self::Breezes { .. } => debug_subscription::breezes,
            Self::GoalSelectors { .. } => debug_subscription::goal_selectors,
            Self::EntityPaths { .. } => debug_subscription::entity_paths,
            Self::EntityBlockIntersections { .. } => debug_subscription::entity_block_intersections,
            Self::BeeHives { .. } => debug_subscription::bee_hives,
            Self::Pois { .. } => debug_subscription::pois,
            Self::RedstoneWireOrientations { .. } => debug_subscription::redstone_wire_orientations,
            Self::VillageSections { .. } => debug_subscription::village_sections,
            Self::Raids { .. } => debug_subscription::raids,
            Self::Structures { .. } => debug_subscription::structures,
            Self::GameEventListeners { .. } => debug_subscription::game_event_listeners,
            Self::NeighborUpdates { .. } => debug_subscription::neighbor_updates,
            Self::GameEvents { .. } => debug_subscription::game_events,
        }
    }
}

impl<'a> Write for DebugSubscriptionEvent<'a> {
    unsafe fn write(&self, w: &mut Writer) {
        unsafe {
            self.id().write(w);
            match self {
                Self::DedicatedServerTickTime => (),
                Self::Bees(x) => x.write(w),
                Self::Brains(x) => x.write(w),
                Self::Breezes(x) => x.write(w),
                Self::GoalSelectors(x) => x.write(w),
                Self::EntityPaths(x) => x.write(w),
                Self::EntityBlockIntersections(x) => x.write(w),
                Self::BeeHives(x) => x.write(w),
                Self::Pois(x) => x.write(w),
                Self::RedstoneWireOrientations(x) => x.write(w),
                Self::VillageSections(x) => x.write(w),
                Self::Raids(x) => x.write(w),
                Self::Structures(x) => x.write(w),
                Self::GameEventListeners(x) => x.write(w),
                Self::NeighborUpdates(x) => x.write(w),
                Self::GameEvents(x) => x.write(w),
            }
        }
    }

    fn len_s(&self) -> usize {
        self.id().len_s()
            + match self {
                Self::DedicatedServerTickTime => 0,
                Self::Bees(x) => x.len_s(),
                Self::Brains(x) => x.len_s(),
                Self::Breezes(x) => x.len_s(),
                Self::GoalSelectors(x) => x.len_s(),
                Self::EntityPaths(x) => x.len_s(),
                Self::EntityBlockIntersections(x) => x.len_s(),
                Self::BeeHives(x) => x.len_s(),
                Self::Pois(x) => x.len_s(),
                Self::RedstoneWireOrientations(x) => x.len_s(),
                Self::VillageSections(x) => x.len_s(),
                Self::Raids(x) => x.len_s(),
                Self::Structures(x) => x.len_s(),
                Self::GameEventListeners(x) => x.len_s(),
                Self::NeighborUpdates(x) => x.len_s(),
                Self::GameEvents(x) => x.len_s(),
            }
    }
}

impl<'a> Read<'a> for DebugSubscriptionEvent<'a> {
    fn read(buf: &mut Reader<'a>) -> Result<Self, Error> {
        Ok(match debug_subscription::read(buf)? {
            debug_subscription::dedicated_server_tick_time => Self::DedicatedServerTickTime,
            debug_subscription::bees => Self::Bees(Read::read(buf)?),
            debug_subscription::brains => Self::Brains(Read::read(buf)?),
            debug_subscription::breezes => Self::Breezes(Read::read(buf)?),
            debug_subscription::goal_selectors => Self::GoalSelectors(Read::read(buf)?),
            debug_subscription::entity_paths => Self::EntityPaths(Read::read(buf)?),
            debug_subscription::entity_block_intersections => {
                Self::EntityBlockIntersections(Read::read(buf)?)
            }
            debug_subscription::bee_hives => Self::BeeHives(Read::read(buf)?),
            debug_subscription::pois => Self::Pois(Read::read(buf)?),
            debug_subscription::redstone_wire_orientations => {
                Self::RedstoneWireOrientations(Read::read(buf)?)
            }
            debug_subscription::village_sections => Self::VillageSections(Read::read(buf)?),
            debug_subscription::raids => Self::Raids(Read::read(buf)?),
            debug_subscription::structures => Self::Structures(Read::read(buf)?),
            debug_subscription::game_event_listeners => Self::GameEventListeners(Read::read(buf)?),
            debug_subscription::neighbor_updates => Self::NeighborUpdates(Read::read(buf)?),
            debug_subscription::game_events => Self::GameEvents(Read::read(buf)?),
        })
    }
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
