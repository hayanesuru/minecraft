#![no_std]
#![allow(non_camel_case_types)]

use mser_macro::Writable;

include!(concat!(env!("OUT_DIR"), "/data.rs"));

#[macro_export]
macro_rules! encode_state {
    ($b:ident($x:expr)) => {
        $crate::block_state::new(
            $x.encode() as $crate::raw_block_state + $crate::block::$b.state_index(),
        )
    };
}

#[macro_export]
macro_rules! decode_state {
    ($b:ident($x:expr)) => {
        $crate::$b::decode((($x.id() - $crate::block::$b.state_index()) as _))
    };
}

#[cold]
#[inline(always)]
const fn cold__() {}

impl Default for biome {
    #[inline]
    fn default() -> Self {
        Self::plains
    }
}

impl Default for dimension_type {
    #[inline]
    fn default() -> Self {
        Self::overworld
    }
}

#[derive(Copy, Clone)]
struct NameMap<T: 'static> {
    key: [u64; 4],
    disps: &'static [(u32, u32)],
    names: *const u8,
    vals: &'static [T],
}

fn hash(key: [u64; 4], name: &[u8], disps: &'static [(u32, u32)], len: u32) -> u32 {
    let hasher = highway::HighwayHasher::new(highway::Key(key));
    let [a, b] = highway::HighwayHash::hash128(hasher, name);
    let g = (a >> 32) as u32;
    let f1 = a as u32;
    let f2 = b as u32;
    let (d1, d2) = disps[(g % (disps.len() as u32)) as usize];
    d2.wrapping_add(f1.wrapping_mul(d1)).wrapping_add(f2) % len
}

impl NameMap<u16> {
    fn get(&self, name: &[u8]) -> Option<u16> {
        let index = hash(self.key, name, self.disps, self.vals.len() as u32);
        let v = unsafe { *self.vals.get_unchecked(index as usize) };
        let offset =
            unsafe { u32::from_ne_bytes(*self.names.add(4 * v as usize).cast::<[u8; 4]>()) };
        let len = unsafe {
            u16::from_ne_bytes(*self.names.add(offset as usize).cast::<[u8; 2]>()) as usize
        };
        let k = unsafe { core::slice::from_raw_parts(self.names.add(offset as usize + 2), len) };
        if name == k {
            Some(v)
        } else {
            None
        }
    }
}

impl NameMap<u8> {
    fn get(&self, name: &[u8]) -> Option<u8> {
        let index = hash(self.key, name, self.disps, self.vals.len() as u32);
        let v = unsafe { *self.vals.get_unchecked(index as usize) };
        let offset =
            unsafe { u32::from_ne_bytes(*self.names.add(4 * v as usize).cast::<[u8; 4]>()) };
        let len = unsafe {
            u16::from_ne_bytes(*self.names.add(offset as usize).cast::<[u8; 2]>()) as usize
        };
        let k = unsafe { core::slice::from_raw_parts(self.names.add(offset as usize + 2), len) };
        if name == k {
            Some(v)
        } else {
            None
        }
    }
}

pub fn make_block_state(
    mut buf: &mut [(block_state_property_key, block_state_property_value)],
    block: block,
) -> block_state {
    let mut offset = 0_u16;
    let mut index = 0_u16;

    for &prop in block.props().iter().rev() {
        let key = prop.key();
        let vals = prop.val();

        let val = buf.iter().position(|&(x, _)| x == key);
        let val = match val {
            Some(x) => unsafe {
                let y = buf.len() - 1;
                let x = if x != y {
                    let y = *buf.get_unchecked_mut(y);
                    let x = buf.get_unchecked_mut(x);
                    Some(core::mem::replace(x, y))
                } else {
                    let x = buf.get_unchecked_mut(x);
                    Some(*x)
                };
                buf = buf.get_unchecked_mut(0..buf.len() - 1);
                x
            },
            None => None,
        };
        let val = match val {
            Some((_, val)) => match vals.iter().position(|&v| v == val) {
                None => 0,
                Some(x) => x as u16,
            },
            None => {
                let def = block.state_default().id() - block.state_index();
                let x = if index == 0 { def } else { def / index };
                x % vals.len() as u16
            }
        };
        if index == 0 {
            offset = val;
            index = vals.len() as u16;
        } else {
            offset += val * index;
            index *= vals.len() as u16;
        }
    }
    block_state::new(block.state_index() + offset)
}

pub fn block_state_props(
    state: block_state,
    buf: &mut [(block_state_property_key, block_state_property_value)],
) -> &[(block_state_property_key, block_state_property_value)] {
    let mut iter = buf.iter_mut();
    let kind = state.to_block();
    let mut raw = state.id() - kind.state_index();
    for prop in kind.props().iter().rev() {
        let v = prop.val();
        let idx = raw % v.len() as u16;
        raw /= v.len() as u16;
        match iter.next_back() {
            Some(x) => {
                *x = (prop.key(), unsafe { *v.get_unchecked(idx as usize) });
                continue;
            }
            None => break,
        }
    }
    let rest = iter.into_slice().len();
    unsafe { buf.get_unchecked(rest..) }
}

impl core::fmt::Debug for block_state {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let mut s = f.debug_struct(self.to_block().name());
        let mut prop_buf = [(
            block_state_property_key::age,
            block_state_property_value::d_0,
        ); 16];
        for (k, v) in block_state_props(*self, &mut prop_buf) {
            s.field(k.name(), v);
        }
        s.finish()
    }
}

impl core::fmt::Display for block_state {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        <Self as core::fmt::Debug>::fmt(self, f)
    }
}

impl block_state {
    #[inline]
    pub const fn to_block(self) -> block {
        unsafe { ::core::mem::transmute(*BLOCK_STATE_TO_BLOCK.add(self.0 as usize)) }
    }
    #[inline]
    pub const fn to_fluid(self) -> fluid_state {
        unsafe { ::core::mem::transmute(*FLUID_STATE.add(self.0 as usize)) }
    }
    #[inline]
    pub const fn luminance(self) -> u8 {
        unsafe { *BLOCK_STATE_SETTINGS.add(self.0 as usize).cast::<u8>() }
    }
    #[inline]
    pub const fn has_sided_transparency(self) -> bool {
        let x = unsafe {
            *BLOCK_STATE_SETTINGS
                .add(self.0 as usize)
                .cast::<u8>()
                .add(1)
        };
        x & 128 != 0
    }
    #[inline]
    pub const fn lava_ignitable(self) -> bool {
        let x = unsafe {
            *BLOCK_STATE_SETTINGS
                .add(self.0 as usize)
                .cast::<u8>()
                .add(1)
        };
        x & 64 != 0
    }
    #[inline]
    pub const fn material_replaceable(self) -> bool {
        let x = unsafe {
            *BLOCK_STATE_SETTINGS
                .add(self.0 as usize)
                .cast::<u8>()
                .add(1)
        };
        x & 32 != 0
    }
    #[inline]
    pub const fn opaque(self) -> bool {
        let x = unsafe {
            *BLOCK_STATE_SETTINGS
                .add(self.0 as usize)
                .cast::<u8>()
                .add(1)
        };
        x & 16 != 0
    }
    #[inline]
    pub const fn tool_required(self) -> bool {
        let x = unsafe {
            *BLOCK_STATE_SETTINGS
                .add(self.0 as usize)
                .cast::<u8>()
                .add(1)
        };
        x & 8 != 0
    }
    #[inline]
    pub const fn exceeds_cube(self) -> bool {
        let x = unsafe {
            *BLOCK_STATE_SETTINGS
                .add(self.0 as usize)
                .cast::<u8>()
                .add(1)
        };
        x & 4 != 0
    }
    #[inline]
    pub const fn redstone_power_source(self) -> bool {
        let x = unsafe {
            *BLOCK_STATE_SETTINGS
                .add(self.0 as usize)
                .cast::<u8>()
                .add(1)
        };
        x & 2 != 0
    }
    #[inline]
    pub const fn has_comparator_output(self) -> bool {
        let x = unsafe {
            *BLOCK_STATE_SETTINGS
                .add(self.0 as usize)
                .cast::<u8>()
                .add(1)
        };
        x & 1 != 0
    }
    #[inline]
    pub const fn opacity(self) -> Option<u8> {
        let n = self.0 as usize;
        let n = unsafe { u16::from_ne_bytes(*BLOCK_STATE_BOUNDS_INDEX.add(n)) };
        if n == 0 {
            cold__();
            None
        } else {
            Some(unsafe { *BLOCK_STATE_BOUNDS.add(n as usize - 1).cast::<u8>() >> 4 })
        }
    }
    #[inline]
    pub const fn solid(self) -> Option<bool> {
        let n = self.0 as usize;
        let n = unsafe { u16::from_ne_bytes(*BLOCK_STATE_BOUNDS_INDEX.add(n)) };
        if n == 0 {
            cold__();
            None
        } else {
            Some(unsafe { *BLOCK_STATE_BOUNDS.add(n as usize - 1).cast::<u8>() & 8 != 0 })
        }
    }
    #[inline]
    pub const fn translucent(self) -> Option<bool> {
        let n = self.0 as usize;
        let n = unsafe { u16::from_ne_bytes(*BLOCK_STATE_BOUNDS_INDEX.add(n)) };
        if n == 0 {
            cold__();
            None
        } else {
            Some(unsafe { *BLOCK_STATE_BOUNDS.add(n as usize - 1).cast::<u8>() & 4 != 0 })
        }
    }
    #[inline]
    pub const fn full_cube(self) -> Option<bool> {
        let n = self.0 as usize;
        let n = unsafe { u16::from_ne_bytes(*BLOCK_STATE_BOUNDS_INDEX.add(n)) };
        if n == 0 {
            cold__();
            None
        } else {
            Some(unsafe { *BLOCK_STATE_BOUNDS.add(n as usize - 1).cast::<u8>() & 2 != 0 })
        }
    }
    #[inline]
    pub const fn opaque_full_cube(self) -> Option<bool> {
        let n = self.0 as usize;
        let n = unsafe { u16::from_ne_bytes(*BLOCK_STATE_BOUNDS_INDEX.add(n)) };
        if n == 0 {
            cold__();
            None
        } else {
            Some(unsafe { *BLOCK_STATE_BOUNDS.add(n as usize - 1).cast::<u8>() & 1 != 0 })
        }
    }
    #[inline]
    pub const fn side_solid_full(self) -> Option<u8> {
        let n = self.0 as usize;
        let n = unsafe { u16::from_ne_bytes(*BLOCK_STATE_BOUNDS_INDEX.add(n)) };
        if n == 0 {
            cold__();
            None
        } else {
            Some(unsafe { *BLOCK_STATE_BOUNDS.add(n as usize - 1).cast::<u8>().add(1) })
        }
    }
    #[inline]
    pub const fn side_solid_center(self) -> Option<u8> {
        let n = self.0 as usize;
        let n = unsafe { u16::from_ne_bytes(*BLOCK_STATE_BOUNDS_INDEX.add(n)) };
        if n == 0 {
            cold__();
            None
        } else {
            Some(unsafe { *BLOCK_STATE_BOUNDS.add(n as usize - 1).cast::<u8>().add(2) })
        }
    }
    #[inline]
    pub const fn side_solid_rigid(self) -> Option<u8> {
        let n = self.0 as usize;
        let n = unsafe { u16::from_ne_bytes(*BLOCK_STATE_BOUNDS_INDEX.add(n)) };
        if n == 0 {
            cold__();
            None
        } else {
            Some(unsafe { *BLOCK_STATE_BOUNDS.add(n as usize - 1).cast::<u8>().add(3) })
        }
    }
    #[inline]
    pub const fn collision_shape(self) -> Option<&'static [[f64; 6]]> {
        let n = self.0 as usize;
        let n = unsafe { u16::from_ne_bytes(*BLOCK_STATE_BOUNDS_INDEX.add(n)) };
        if n == 0 {
            cold__();
            None
        } else {
            let index = unsafe { *BLOCK_STATE_BOUNDS.add(n as usize - 1 + 4).cast::<[u8; 2]>() };
            let index = u16::from_ne_bytes(index) as usize;
            Some(unsafe { *SHAPES.as_ptr().add(index) })
        }
    }
    #[inline]
    pub const fn culling_shape(self) -> Option<&'static [[f64; 6]]> {
        let n = self.0 as usize;
        let n = unsafe { u16::from_ne_bytes(*BLOCK_STATE_BOUNDS_INDEX.add(n)) };
        if n == 0 {
            cold__();
            None
        } else {
            let index = unsafe { *BLOCK_STATE_BOUNDS.add(n as usize - 1 + 6).cast::<[u8; 2]>() };
            let index = u16::from_ne_bytes(index) as usize;
            Some(unsafe { *SHAPES.as_ptr().add(index) })
        }
    }
}

impl item {
    #[inline]
    pub const fn max_count(self) -> u8 {
        unsafe { *ITEM_MAX_COUNT.as_ptr().add(self as usize) }
    }

    #[inline]
    pub const fn to_block(self) -> block {
        unsafe { block::new(*ITEM.as_ptr().add(self as usize)) }
    }
}

impl block {
    #[inline]
    pub const fn hardness(self) -> f32 {
        unsafe {
            let x = *BLOCK_SETTINGS_INDEX.as_ptr().add(self as usize) as usize;
            *BLOCK_SETTINGS.as_ptr().add(x).cast::<f32>()
        }
    }
    #[inline]
    pub const fn blast_resistance(self) -> f32 {
        unsafe {
            let x = *BLOCK_SETTINGS_INDEX.as_ptr().add(self as usize) as usize;
            *BLOCK_SETTINGS.as_ptr().add(x).cast::<f32>().add(1)
        }
    }
    #[inline]
    pub const fn slipperiness(self) -> f32 {
        unsafe {
            let x = *BLOCK_SETTINGS_INDEX.as_ptr().add(self as usize) as usize;
            *BLOCK_SETTINGS.as_ptr().add(x).cast::<f32>().add(2)
        }
    }
    #[inline]
    pub const fn velocity_multiplier(self) -> f32 {
        unsafe {
            let x = *BLOCK_SETTINGS_INDEX.as_ptr().add(self as usize) as usize;
            *BLOCK_SETTINGS.as_ptr().add(x).cast::<f32>().add(3)
        }
    }
    #[inline]
    pub const fn jump_velocity_multiplier(self) -> f32 {
        unsafe {
            let x = *BLOCK_SETTINGS_INDEX.as_ptr().add(self as usize) as usize;
            *BLOCK_SETTINGS.as_ptr().add(x).cast::<f32>().add(4)
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum fluid_state {
    empty,
    flowing_water_falling_1,
    flowing_water_falling_2,
    flowing_water_falling_3,
    flowing_water_falling_4,
    flowing_water_falling_5,
    flowing_water_falling_6,
    flowing_water_falling_7,
    flowing_water_falling_8,
    flowing_water_1,
    flowing_water_2,
    flowing_water_3,
    flowing_water_4,
    flowing_water_5,
    flowing_water_6,
    flowing_water_7,
    flowing_water_8,
    water_falling,
    water,
    flowing_lava_falling_1,
    flowing_lava_falling_2,
    flowing_lava_falling_3,
    flowing_lava_falling_4,
    flowing_lava_falling_5,
    flowing_lava_falling_6,
    flowing_lava_falling_7,
    flowing_lava_falling_8,
    flowing_lava_1,
    flowing_lava_2,
    flowing_lava_3,
    flowing_lava_4,
    flowing_lava_5,
    flowing_lava_6,
    flowing_lava_7,
    flowing_lava_8,
    lava_falling,
    lava,
}

impl fluid_state {
    #[inline]
    pub const fn to_fluid(self) -> fluid {
        match self as u8 {
            0 => fluid::empty,
            1..=16 => fluid::flowing_water,
            17..=18 => fluid::water,
            19..=34 => fluid::flowing_lava,
            _ => fluid::lava,
        }
    }

    #[inline]
    pub const fn is_empty(self) -> bool {
        matches!(self as u8, 0)
    }

    #[inline]
    pub const fn is_flowing_water(self) -> bool {
        matches!(self as u8, 1..=16)
    }

    #[inline]
    pub const fn is_water(self) -> bool {
        matches!(self as u8, 17..=18)
    }

    #[inline]
    pub const fn is_flowing_lava(self) -> bool {
        matches!(self as u8, 19..=34)
    }

    #[inline]
    pub const fn is_lava(self) -> bool {
        matches!(self as u8, 35..=36)
    }
}

impl From<bool> for prop_waterlogged {
    #[inline]
    fn from(value: bool) -> Self {
        if value {
            Self::r#true
        } else {
            Self::r#false
        }
    }
}

impl From<prop_waterlogged> for bool {
    #[inline]
    fn from(value: prop_waterlogged) -> Self {
        match value {
            prop_waterlogged::r#true => true,
            prop_waterlogged::r#false => false,
        }
    }
}

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
    #[inline]
    pub const fn id(self) -> raw_play_s2c {
        self as raw_play_s2c
    }
    #[inline]
    pub const fn new(x: raw_play_s2c) -> Self {
        debug_assert!(x <= Self::SynchronizeTags as raw_play_s2c);
        unsafe { ::core::mem::transmute(x) }
    }
}

pub type raw_status_s2c = u8;
#[derive(Clone, Copy, PartialEq, Eq, Hash, Writable)]
#[repr(u8)]
pub enum status_s2c {
    QueryResponse,
    PingResult,
}

impl status_s2c {
    #[inline]
    pub const fn id(self) -> raw_status_s2c {
        self as raw_status_s2c
    }

    #[inline]
    pub const fn new(x: raw_status_s2c) -> Self {
        debug_assert!(x <= Self::PingResult as raw_status_s2c);
        unsafe { ::core::mem::transmute(x) }
    }
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
    #[inline]
    pub const fn id(self) -> raw_login_s2c {
        self as raw_login_s2c
    }
    #[inline]
    pub const fn new(x: raw_login_s2c) -> Self {
        debug_assert!(x <= Self::CookieRequest as raw_login_s2c);
        unsafe { ::core::mem::transmute(x) }
    }
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
    #[inline]
    pub const fn id(self) -> raw_configuration_s2c {
        self as raw_configuration_s2c
    }
    #[inline]
    pub const fn new(x: raw_configuration_s2c) -> Self {
        debug_assert!(x <= Self::SynchronizeTags as raw_configuration_s2c);
        unsafe { ::core::mem::transmute(x) }
    }
}

pub type raw_handshake_c2s = u8;
#[derive(Clone, Copy, PartialEq, Eq, Hash, Writable)]
#[repr(u8)]
pub enum handshake_c2s {
    Handshake,
}

impl handshake_c2s {
    #[inline]
    pub const fn id(self) -> raw_handshake_c2s {
        self as raw_handshake_c2s
    }
    #[inline]
    pub const fn new(x: raw_handshake_c2s) -> Self {
        debug_assert!(x <= Self::Handshake as raw_handshake_c2s);
        unsafe { ::core::mem::transmute(x) }
    }
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
    #[inline]
    pub const fn id(self) -> raw_play_c2s {
        self as raw_play_c2s
    }
    #[inline]
    pub const fn new(x: raw_play_c2s) -> Self {
        debug_assert!(x <= Self::PlayerInteractItem as raw_play_c2s);
        unsafe { ::core::mem::transmute(x) }
    }
}

pub type raw_status_c2s = u8;
#[derive(Clone, Copy, PartialEq, Eq, Hash, Writable)]
#[repr(u8)]
pub enum status_c2s {
    QueryRequest,
    QueryPing,
}

impl status_c2s {
    #[inline]
    pub const fn id(self) -> raw_status_c2s {
        self as raw_status_c2s
    }
    #[inline]
    pub const fn new(x: u8) -> Self {
        debug_assert!(x <= Self::QueryPing as raw_status_c2s);
        unsafe { ::core::mem::transmute(x) }
    }
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
    #[inline]
    pub const fn id(self) -> raw_login_c2s {
        self as raw_login_c2s
    }
    #[inline]
    pub const fn new(x: raw_login_c2s) -> Self {
        debug_assert!(x <= Self::CookieResponse as raw_login_c2s);
        unsafe { ::core::mem::transmute(x) }
    }
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
    #[inline]
    pub const fn id(self) -> raw_configuration_c2s {
        self as raw_configuration_c2s
    }
    #[inline]
    pub const fn new(x: raw_configuration_c2s) -> Self {
        debug_assert!(x <= Self::ResourcePackStatus as raw_configuration_c2s);
        unsafe { ::core::mem::transmute(x) }
    }
}

#[test]
fn test() {
    assert_eq!(block::white_concrete.name(), "white_concrete");
    assert_eq!(Some(block::white_concrete), block::parse(b"white_concrete"));

    let x = block::white_concrete.state_default();
    assert_eq!(x.side_solid_full(), Some(0b111111));
    assert_eq!(x.side_solid_rigid(), Some(0b111111));
    assert_eq!(x.side_solid_center(), Some(0b111111));
    let x = block::torch.state_default();
    assert_eq!(x.side_solid_full(), Some(0b000000));
    assert_eq!(x.side_solid_center(), Some(0b000000));
    assert_eq!(x.side_solid_rigid(), Some(0b000000));
}
