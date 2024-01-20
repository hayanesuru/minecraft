use mser_macro::Writable;

pub type raw_play_s2c = u8;
#[derive(Clone, Copy, PartialEq, Eq, Hash, Writable)]
#[repr(u8)]
pub enum play_s2c {
    BundleDelimiter,
    EntitySpawn,
    ExperienceOrbSpawn,
    EntityAnimation,
    Statistics,
    PlayerActionResponse,
    BlockBreakingProgress,
    BlockEntityUpdate,
    BlockEvent,
    BlockUpdate,
    BossBar,
    Difficulty,
    ChunkSent,
    StartChunkSend,
    ChunkBiomeData,
    ClearTitle,
    CommandSuggestions,
    CommandTree,
    CloseScreen,
    Inventory,
    ScreenHandlerPropertyUpdate,
    ScreenHandlerSlotUpdate,
    CookieRequest,
    CooldownUpdate,
    ChatSuggestions,
    CustomPayload,
    EntityDamage,
    RemoveMessage,
    Disconnect,
    ProfilelessChatMessage,
    EntityStatus,
    Explosion,
    UnloadChunk,
    GameStateChange,
    OpenHorseScreen,
    DamageTilt,
    WorldBorderInitialize,
    KeepAlive,
    ChunkData,
    WorldEvent,
    Particle,
    LightUpdate,
    GameJoin,
    MapUpdate,
    SetTradeOffers,
    EntityMoveRelative,
    EntityRotateAndMoveRelative,
    EntityRotate,
    VehicleMove,
    OpenWrittenBook,
    OpenScreen,
    SignEditorOpen,
    CommonPing,
    PingResult,
    CraftFailedResponse,
    PlayerAbilities,
    ChatMessage,
    EndCombat,
    EnterCombat,
    DeathMessage,
    PlayerRemove,
    PlayerList,
    LookAt,
    PlayerPositionLook,
    UnlockRecipes,
    EntitiesDestroy,
    RemoveEntityStatusEffect,
    ScoreboardScoreReset,
    ResourcePackRemove,
    ResourcePackSend,
    PlayerRespawn,
    EntitySetHeadYaw,
    ChunkDeltaUpdate,
    SelectAdvancementTab,
    ServerMetadata,
    OverlayMessage,
    WorldBorderCenterChanged,
    WorldBorderInterpolateSize,
    WorldBorderSizeChanged,
    WorldBorderWarningTimeChanged,
    WorldBorderWarningBlocksChanged,
    SetCameraEntity,
    UpdateSelectedSlot,
    ChunkRenderDistanceCenter,
    ChunkLoadDistance,
    PlayerSpawnPosition,
    ScoreboardDisplay,
    EntityTrackerUpdate,
    EntityAttach,
    EntityVelocityUpdate,
    EntityEquipmentUpdate,
    ExperienceBarUpdate,
    HealthUpdate,
    ScoreboardObjectiveUpdate,
    EntityPassengersSet,
    Team,
    ScoreboardScoreUpdate,
    SimulationDistance,
    Subtitle,
    WorldTimeUpdate,
    Title,
    TitleFade,
    PlaySoundFromEntity,
    PlaySound,
    EnterReconfiguration,
    StopSound,
    StoreCookie,
    GameMessage,
    PlayerListHeader,
    NbtQueryResponse,
    ItemPickupAnimation,
    EntityPosition,
    UpdateTickRate,
    TickStep,
    ServerTransfer,
    AdvancementUpdate,
    EntityAttributes,
    EntityStatusEffect,
    SynchronizeRecipes,
    SynchronizeTags,
}

impl play_s2c {
    pub const MAX: raw_play_s2c = Self::SynchronizeTags as _;
}

pub type raw_status_s2c = u8;
#[derive(Clone, Copy, PartialEq, Eq, Hash, Writable)]
#[repr(u8)]
pub enum status_s2c {
    QueryResponse,
    PingResult,
}

impl status_s2c {
    pub const MAX: raw_status_s2c = Self::PingResult as _;
}

pub type raw_login_s2c = u8;
#[derive(Clone, Copy, PartialEq, Eq, Hash, Writable)]
#[repr(u8)]
pub enum login_s2c {
    LoginDisconnect,
    LoginHello,
    LoginSuccess,
    LoginCompression,
    LoginQueryRequest,
    CookieRequest,
}

impl login_s2c {
    pub const MAX: raw_login_s2c = Self::CookieRequest as _;
}

pub type raw_configuration_s2c = u8;
#[derive(Clone, Copy, PartialEq, Eq, Hash, Writable)]
#[repr(u8)]
pub enum configuration_s2c {
    CookieRequest,
    CustomPayload,
    Disconnect,
    Ready,
    KeepAlive,
    CommonPing,
    DynamicRegistries,
    ResourcePackRemove,
    ResourcePackSend,
    StoreCookie,
    ServerTransfer,
    Features,
    SynchronizeTags,
}

impl configuration_s2c {
    pub const MAX: raw_configuration_s2c = Self::SynchronizeTags as _;
}

pub type raw_handshake_c2s = u8;
#[derive(Clone, Copy, PartialEq, Eq, Hash, Writable)]
#[repr(u8)]
pub enum handshake_c2s {
    Handshake,
}

impl handshake_c2s {
    pub const MAX: raw_handshake_c2s = Self::Handshake as _;
}

pub type raw_play_c2s = u8;
#[derive(Clone, Copy, PartialEq, Eq, Hash, Writable)]
#[repr(u8)]
pub enum play_c2s {
    TeleportConfirm,
    QueryBlockNbt,
    UpdateDifficulty,
    MessageAcknowledgment,
    CommandExecution,
    ChatMessage,
    PlayerSession,
    AcknowledgeChunks,
    ClientStatus,
    ClientOptions,
    RequestCommandCompletions,
    AcknowledgeReconfiguration,
    ButtonClick,
    ClickSlot,
    CloseHandledScreen,
    SlotChangedState,
    CookieResponse,
    CustomPayload,
    BookUpdate,
    QueryEntityNbt,
    PlayerInteractEntity,
    JigsawGenerating,
    KeepAlive,
    UpdateDifficultyLock,
    PlayerMovePositionAndOnGround,
    PlayerMoveFull,
    PlayerMoveLookAndOnGround,
    PlayerMoveOnGroundOnly,
    VehicleMove,
    BoatPaddleState,
    PickFromInventory,
    QueryPing,
    CraftRequest,
    UpdatePlayerAbilities,
    PlayerAction,
    ClientCommand,
    PlayerInput,
    CommonPong,
    RecipeCategoryOptions,
    RecipeBookData,
    RenameItem,
    ResourcePackStatus,
    AdvancementTab,
    SelectMerchantTrade,
    UpdateBeacon,
    UpdateSelectedSlot,
    UpdateCommandBlock,
    UpdateCommandBlockMinecart,
    CreativeInventoryAction,
    UpdateJigsaw,
    UpdateStructureBlock,
    UpdateSign,
    HandSwing,
    SpectatorTeleport,
    PlayerInteractBlock,
    PlayerInteractItem,
}

impl play_c2s {
    pub const MAX: raw_play_c2s = Self::PlayerInteractItem as _;
}

pub type raw_status_c2s = u8;
#[derive(Clone, Copy, PartialEq, Eq, Hash, Writable)]
#[repr(u8)]
pub enum status_c2s {
    QueryRequest,
    QueryPing,
}

impl status_c2s {
    pub const MAX: status_c2s = Self::QueryPing as _;
}

pub type raw_login_c2s = u8;
#[derive(Clone, Copy, PartialEq, Eq, Hash, Writable)]
#[repr(u8)]
pub enum login_c2s {
    LoginHello,
    LoginKey,
    LoginQueryResponse,
    EnterConfiguration,
    CookieResponse,
}

impl login_c2s {
    pub const MAX: raw_login_c2s = Self::CookieResponse as _;
}

pub type raw_configuration_c2s = u8;
#[derive(Clone, Copy, PartialEq, Eq, Hash, Writable)]
#[repr(u8)]
pub enum configuration_c2s {
    ClientOptions,
    CookieResponse,
    CustomPayload,
    Ready,
    KeepAlive,
    CommonPong,
    ResourcePackStatus,
}

impl configuration_c2s {
    pub const MAX: raw_configuration_c2s = Self::ResourcePackStatus as _;
}
