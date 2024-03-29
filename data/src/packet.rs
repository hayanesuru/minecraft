#![allow(non_camel_case_types)]

use mser_macro::Writable;

pub type raw_play_s2c = u8;
#[derive(Clone, Copy, PartialEq, Eq, Hash, Writable)]
#[repr(u8)]
pub enum play_s2c {
    EntitySpawn,
    ExperienceOrbSpawn,
    MobSpawn,
    PaintingSpawn,
    PlayerSpawn,
    EntityAnimation,
    Statistics,
    PlayerActionResponse,
    BlockBreakingProgress,
    BlockEntityUpdate,
    BlockEvent,
    BlockUpdate,
    BossBar,
    Difficulty,
    GameMessage,
    CommandSuggestions,
    CommandTree,
    ConfirmScreenAction,
    CloseScreen,
    Inventory,
    ScreenHandlerPropertyUpdate,
    ScreenHandlerSlotUpdate,
    CooldownUpdate,
    CustomPayload,
    PlaySoundId,
    Disconnect,
    EntityStatus,
    Explosion,
    UnloadChunk,
    GameStateChange,
    OpenHorseScreen,
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
    Entity,
    VehicleMove,
    OpenWrittenBook,
    OpenScreen,
    SignEditorOpen,
    CraftFailedResponse,
    PlayerAbilities,
    CombatEvent,
    PlayerList,
    LookAt,
    PlayerPositionLook,
    UnlockRecipes,
    EntitiesDestroy,
    RemoveEntityStatusEffect,
    ResourcePackSend,
    PlayerRespawn,
    EntitySetHeadYaw,
    ChunkDeltaUpdate,
    SelectAdvancementTab,
    WorldBorder,
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
    ScoreboardPlayerUpdate,
    WorldTimeUpdate,
    Title,
    PlaySoundFromEntity,
    PlaySound,
    StopSound,
    PlayerListHeader,
    NbtQueryResponse,
    ItemPickupAnimation,
    EntityPosition,
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
    QueryPong,
}

impl status_s2c {
    pub const MAX: raw_status_s2c = Self::QueryPong as _;
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
}

impl login_s2c {
    pub const MAX: raw_login_s2c = Self::LoginQueryRequest as _;
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
    ChatMessage,
    ClientStatus,
    ClientSettings,
    RequestCommandCompletions,
    ConfirmScreenAction,
    ButtonClick,
    ClickSlot,
    CloseHandledScreen,
    CustomPayload,
    BookUpdate,
    QueryEntityNbt,
    PlayerInteractEntity,
    JigsawGenerating,
    KeepAlive,
    UpdateDifficultyLock,
    PlayerMovePositionOnly,
    PlayerMoveBoth,
    PlayerMoveLookOnly,
    PlayerMove,
    VehicleMove,
    BoatPaddleState,
    PickFromInventory,
    CraftRequest,
    UpdatePlayerAbilities,
    PlayerAction,
    ClientCommand,
    PlayerInput,
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
}

impl login_c2s {
    pub const MAX: raw_login_c2s = Self::LoginQueryResponse as _;
}
