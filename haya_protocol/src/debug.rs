use haya_collection::List;
use haya_math::BlockPosPacked;
use minecraft_data::debug_subscription;
use mser::Utf8;

#[derive(Clone)]
pub enum DebugSubscriptionUpdate<'a> {
    DedicatedServerTickTime,
    Bees(Option<DebugBeeInfo<'a>>),
    Brains(Option<DebugBrainDump<'a>>),
    Breezes(Option<DebugBreezeInfo>),
    GoalSelectors(Option<DebugGoalInfo<'a>>),
    EntityPaths,
    EntityBlockIntersections,
    BeeHives,
    Pois,
    RedstoneWireOrientations,
    VillageSections,
    Raids,
    Structures,
    GameEventListeners,
    NeighborUpdates,
    GameEvents,
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

#[derive(Clone)]
pub enum DebugSubscriptionEvent<'a> {
    DedicatedServerTickTime,
    Bees(DebugBeeInfo<'a>),
    Brains(DebugBrainDump<'a>),
    Breezes(DebugBreezeInfo),
    GoalSelectors(DebugGoalInfo<'a>),
    EntityPaths,
    EntityBlockIntersections,
    BeeHives,
    Pois,
    RedstoneWireOrientations,
    VillageSections,
    Raids,
    Structures,
    GameEventListeners,
    NeighborUpdates,
    GameEvents,
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
