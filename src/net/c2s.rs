use crate::inventory::{ItemStack, SlotActionType};
use crate::math::{BlockPos, Direction, Position};
use crate::text::ChatVisibility;
use crate::{Bytes, Read, UnsafeWriter, Writable, Write, V21, V32, Arm, CommandBlockType, Hand, Difficulty, JigsawBlockJoint, BlockHitResult, RecipeBookCategory, StructureBlockAction, BlockMirror, StructureBlockMode, BlockRotation};
use core::mem::transmute;
use glam::{IVec3, Vec3};
use minecraft_data::{configuration_c2s, handshake_c2s, login_c2s, mob_effect, play_c2s};
use simdutf8::basic::from_utf8;
use uuid::Uuid;

#[derive(Writable, Clone, Copy)]
#[ser(prefix = handshake_c2s::Handshake)]
pub struct Handshake<'a> {
    pub protocol_version: u32,
    pub address: &'a str,
    pub port: u16,
    pub intended_state: u32,
}

#[derive(Writable, Clone, Copy)]
#[ser(prefix = login_c2s::LoginHello)]
pub struct LoginHello<'a> {
    pub name: &'a str,
    pub profile_id: Uuid,
}

#[derive(Clone, Copy, Writable)]
#[ser(prefix = login_c2s::EnterConfiguration)]
pub struct EnterConfiguration;

impl Read for EnterConfiguration {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        let _ = buf;
        Some(Self)
    }
}

#[derive(Clone, Copy)]
pub struct LoginKey {
    pub encrypted_secret_key: *const [u8],
    pub nonce: *const [u8],
}

impl Read for LoginKey {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        Some(Self {
            encrypted_secret_key: s2(buf)?,
            nonce: s2(buf)?,
        })
    }
}

#[derive(Clone, Copy, Writable)]
#[ser(prefix = configuration_c2s::Ready)]
pub struct ConfReady;
impl Read for ConfReady {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        let _ = buf;
        Some(Self)
    }
}

#[derive(Clone, Copy)]
pub struct CustomPayload {
    pub id: *const str,
    pub payload: *const [u8],
}

impl Read for CustomPayload {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        Some(Self {
            id: s(buf, 32767)?,
            payload: *buf,
        })
    }
}

#[derive(Clone, Copy, Writable)]
#[ser(prefix = configuration_c2s::KeepAlive)]
pub struct ConfKeepAlive {
    pub id: u64,
}

impl Read for ConfKeepAlive {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        Some(Self { id: buf.u64()? })
    }
}

#[derive(Clone, Copy, Writable)]
#[ser(prefix = play_c2s::KeepAlive)]
pub struct KeepAlive {
    pub id: u64,
}

impl Read for KeepAlive {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        Some(Self { id: buf.u64()? })
    }
}

#[derive(Clone, Copy)]
pub struct ConfClientOptions {
    pub language: *const str,
    pub view_distance: u8,
    pub chat_visibility: ChatVisibility,
    pub chat_colors: bool,
    pub model_customisation: u8,
    pub main_hand: Arm,
    pub text_filtering_enabled: bool,
    pub allows_listing: bool,
}

impl Write for ConfClientOptions {
    fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            configuration_c2s::ClientOptions.write(w);
            V21((*self.language).len() as u32).write(w);
            w.write((*self.language).as_bytes());
            w.write_byte(self.view_distance);
            self.chat_visibility.write(w);
            self.chat_colors.write(w);
            w.write_byte(self.model_customisation);
            self.main_hand.write(w);
            self.text_filtering_enabled.write(w);
            self.allows_listing.write(w);
        }
    }

    fn len(&self) -> usize {
        unsafe {
            configuration_c2s::ClientOptions.len()
                + V21((*self.language).len() as u32).len()
                + (*self.language).len()
                + 1
                + self.chat_visibility.len()
                + self.chat_colors.len()
                + 1
                + self.main_hand.len()
                + 2
        }
    }
}

#[derive(Clone, Copy)]
pub struct ClientOptions {
    pub language: *const str,
    pub view_distance: u8,
    pub chat_visibility: ChatVisibility,
    pub chat_colors: bool,
    pub model_customisation: u8,
    pub main_hand: Arm,
    pub text_filtering_enabled: bool,
    pub allows_listing: bool,
}

impl Read for ClientOptions {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        Some(Self {
            language: s(buf, 32767)?,
            view_distance: buf.u8()?,
            chat_visibility: ChatVisibility::from(buf.u8()?),
            chat_colors: buf.u8()? == 1,
            model_customisation: buf.u8()?,
            main_hand: Arm::from(buf.u8()?),
            text_filtering_enabled: buf.u8()? == 1,
            allows_listing: buf.u8()? == 1,
        })
    }
}

#[derive(Clone, Copy)]
pub struct AdvancementTabOpen {
    pub action: AdvancementTabOpenAction,
    pub tab_to_open: *const str,
}
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum AdvancementTabOpenAction {
    OpenedTab,
    ClosedScreen,
}

impl AdvancementTabOpenAction {
    #[inline]
    pub const fn new(id: u8) -> Self {
        if id == 0 {
            Self::OpenedTab
        } else {
            Self::ClosedScreen
        }
    }
}
impl Read for AdvancementTabOpen {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        Some(Self {
            action: AdvancementTabOpenAction::new(buf.u8()?),
            tab_to_open: s(buf, 32767)?,
        })
    }
}

#[derive(Clone, Copy)]
pub struct BeaconUpdate {
    pub primary_effect_id: Option<mob_effect>,
    pub secondary_effect_id: Option<mob_effect>,
}

impl Read for BeaconUpdate {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        Some(Self {
            primary_effect_id: o(buf)?,
            secondary_effect_id: o(buf)?,
        })
    }
}

#[derive(Clone, Copy)]
pub struct BlockNbtQuery {
    pub transaction_id: u32,
    pub pos: BlockPos,
}

impl Read for BlockNbtQuery {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        Some(Self {
            transaction_id: V32::read(buf)?.0,
            pos: BlockPos::from(buf.i64()?),
        })
    }
}

#[derive(Clone, Copy)]
pub struct BoatPaddleStateUpdate {
    pub left_paddling: bool,
    pub right_paddling: bool,
}

impl Read for BoatPaddleStateUpdate {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        Some(Self {
            left_paddling: buf.u8()? == 1,
            right_paddling: buf.u8()? == 1,
        })
    }
}

#[derive(Clone, Copy)]
pub struct BookUpdate {
    pub slot: u32,
    pub pages: *mut [*const str],
    pub title: *const str,
}

impl Read for BookUpdate {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        Some(Self {
            slot: V32::read(buf)?.0,
            pages: {
                let len = V32::read(buf)?.0 as usize;
                if len > 200 {
                    return None;
                }
                let mut vec = Vec::with_capacity(len);
                for _ in 0..len {
                    vec.push(s(buf, 8192)?);
                }
                Box::into_raw(vec.into_boxed_slice())
            },
            title: s(buf, 128)?,
        })
    }
}

#[derive(Clone, Copy)]
pub struct ButtonClick {
    pub sync_id: u8,
    pub button_id: u8,
}

impl Read for ButtonClick {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        Some(Self {
            sync_id: buf.u8()?,
            button_id: buf.u8()?,
        })
    }
}

#[derive(Clone, Copy)]
pub struct CommandExecution {
    pub command: *const str,
    pub timestamp: u64,
    pub salt: u64,
    // pub argument_signatures
    // pub message_acknowledgments
}

impl Read for CommandExecution {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        Some(Self {
            command: s(buf, 256)?,
            timestamp: buf.u64()?,
            salt: buf.u64()?,
        })
    }
}

#[derive(Clone, Copy)]
pub struct ChatMessage {
    pub message: *const str,
    pub timestamp: u64,
    pub salt: u64,
    pub signature: Option<*const [u8; 256]>,
    // pub message_acknowledgments
}

impl Read for ChatMessage {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        Some(Self {
            message: s(buf, 256)?,
            timestamp: buf.u64()?,
            salt: buf.u64()?,
            signature: {
                if buf.u8()? == 1 {
                    Some(buf.array()?)
                } else {
                    None
                }
            },
        })
    }
}

#[derive(Clone, Copy)]
pub struct ChatSessionUpdate {
    pub uuid: Uuid,
    pub expires_at: u64,
    pub key: *const [u8],
    pub key_signature: *const [u8],
}

impl Read for ChatSessionUpdate {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        Some(Self {
            uuid: Uuid::from_bytes(*buf.array()?),
            expires_at: buf.u64()?,
            key: s2(buf)?,
            key_signature: s2(buf)?,
        })
    }
}

#[derive(Clone, Copy)]
pub struct ChunkBatchAcknowledgement {
    pub desired_chunks_per_tick: f32,
}

impl Read for ChunkBatchAcknowledgement {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        Some(Self {
            desired_chunks_per_tick: buf.f32()?,
        })
    }
}

#[derive(Clone, Copy)]
pub struct ClientCommand {
    pub entity_id: u32,
    pub mode: ClientCommandMode,
    pub mount_jump_height: u32,
}

impl Read for ClientCommand {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        Some(Self {
            entity_id: V32::read(buf)?.0,
            mode: ClientCommandMode::new(buf.u8()?),
            mount_jump_height: V32::read(buf)?.0,
        })
    }
}

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum ClientCommandMode {
    PressShiftKey,
    ReleaseShiftKey,
    StopSleeping,
    StartSprinting,
    StopSprinting,
    StartRidingJump,
    StopRidingJump,
    OpenInventory,
    StartFallFlying,
}

impl ClientCommandMode {
    #[inline]
    pub const fn new(id: u8) -> Self {
        if id > 8 {
            unsafe { transmute(0_u8) }
        } else {
            unsafe { transmute(id) }
        }
    }
}

#[derive(Clone, Copy)]
pub struct ClientStatusUpdate {
    pub mode: ClientStatusUpdateMode,
}

impl Read for ClientStatusUpdate {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        Some(Self {
            mode: ClientStatusUpdateMode::new(buf.u8()?),
        })
    }
}

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum ClientStatusUpdateMode {
    PerformRespawn,
    RequestStats,
}

impl ClientStatusUpdateMode {
    #[inline]
    pub const fn new(id: u8) -> Self {
        if id == 0 {
            Self::PerformRespawn
        } else {
            Self::RequestStats
        }
    }
}

#[derive(Clone, Copy)]
pub struct CommandBlockMinecartUpdate {
    pub entity_id: u32,
    pub command: *const str,
    pub track_output: bool,
}

impl Read for CommandBlockMinecartUpdate {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        Some(Self {
            entity_id: V32::read(buf)?.0,
            command: s(buf, 32767)?,
            track_output: buf.u8()? == 1,
        })
    }
}

#[derive(Clone, Copy)]
pub struct CommandBlockUpdate {
    pub pos: BlockPos,
    pub command: *const str,
    pub r#type: CommandBlockType,
    pub track_output: bool,
    pub conditional: bool,
    pub always_active: bool,
}

impl Read for CommandBlockUpdate {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        let pos = buf.i64()?.into();
        let command = s(buf, 32767)?;
        let r#type = buf.u8()?.into();
        let flags = buf.u8()?;
        let track_output = (flags & 1) != 0;
        let conditional = (flags & 2) != 0;
        let always_active = (flags & 4) != 0;

        Some(Self {
            pos,
            command,
            r#type,
            track_output,
            conditional,
            always_active,
        })
    }
}

#[derive(Clone, Copy)]
pub struct CommandCompletionRequest {
    pub completion_id: u32,
    pub partial_command: *const str,
}

impl Read for CommandCompletionRequest {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        Some(Self {
            completion_id: V32::read(buf)?.0,
            partial_command: s(buf, 32500)?,
        })
    }
}

#[derive(Clone, Copy)]
pub struct CraftRequest {
    pub sync_id: u8,
    pub recipe: *const str,
    pub craft_all: bool,
}

impl Read for CraftRequest {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        Some(Self {
            sync_id: buf.u8()?,
            recipe: s(buf, 32767)?,
            craft_all: buf.u8()? == 1,
        })
    }
}

#[derive(Clone, Copy)]
pub struct CreativeInventoryAction {
    pub slot: u16,
    pub stack: ItemStack,
}

impl Read for CreativeInventoryAction {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        Some(Self {
            slot: buf.u16()?,
            stack: ItemStack::read(buf)?,
        })
    }
}

#[derive(Clone, Copy)]
pub struct DifficultyLockUpdate {
    pub difficulty_locked: bool,
}

impl Read for DifficultyLockUpdate {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        Some(Self {
            difficulty_locked: buf.u8()? == 1,
        })
    }
}

#[derive(Clone, Copy)]
pub struct DifficultyUpdate {
    pub difficulty: Difficulty,
}

impl Read for DifficultyUpdate {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        Some(Self {
            difficulty: buf.u8()?.into(),
        })
    }
}

#[derive(Clone, Copy)]
pub struct EntityNbtQuery {
    pub transaction_id: u32,
    pub entity_id: u32,
}

impl Read for EntityNbtQuery {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        Some(Self {
            transaction_id: V32::read(buf)?.0,
            entity_id: V32::read(buf)?.0,
        })
    }
}

#[derive(Clone, Copy)]
pub struct HandledScreenClose {
    pub sync_id: u8,
}

impl Read for HandledScreenClose {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        Some(Self { sync_id: buf.u8()? })
    }
}

#[derive(Clone, Copy)]
pub struct HandSwing {
    pub hand: Hand,
}

impl Read for HandSwing {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        Some(Self {
            hand: buf.u8()?.into(),
        })
    }
}

#[derive(Clone, Copy)]
pub struct InventoryItemPick {
    pub slot: u32,
}

impl Read for InventoryItemPick {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        Some(Self {
            slot: V32::read(buf)?.0,
        })
    }
}

#[derive(Clone, Copy)]
pub struct ItemRename {
    pub name: *const str,
}

impl Read for ItemRename {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        Some(Self {
            name: s(buf, 32767)?,
        })
    }
}

#[derive(Clone, Copy)]
pub struct JigsawGeneration {
    pub pos: BlockPos,
    pub max_depth: u32,
    pub keep_jigsaws: bool,
}

impl Read for JigsawGeneration {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        Some(Self {
            pos: buf.i64()?.into(),
            max_depth: V32::read(buf)?.0,
            keep_jigsaws: buf.u8()? == 1,
        })
    }
}

#[derive(Clone, Copy)]
pub struct JigsawUpdate {
    pub pos: BlockPos,
    pub attachment_type: *const str,
    pub target_pool: *const str,
    pub pool: *const str,
    pub final_state: *const str,
    pub joint_type: JigsawBlockJoint,
    pub selection: i32,
    pub placement: i32,
}

impl Read for JigsawUpdate {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        Some(Self {
            pos: buf.i64()?.into(),
            attachment_type: s(buf, 32767)?,
            target_pool: s(buf, 32767)?,
            pool: s(buf, 32767)?,
            final_state: s(buf, 32767)?,
            joint_type: JigsawBlockJoint::parse(unsafe { &*s(buf, 32767)? }),
            selection: V32::read(buf)?.0 as i32,
            placement: V32::read(buf)?.0 as i32,
        })
    }
}

#[derive(Clone, Copy)]
pub struct MerchantTradeSelection {
    pub trade_id: u32,
}

impl Read for MerchantTradeSelection {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        Some(Self {
            trade_id: V32::read(buf)?.0,
        })
    }
}

#[derive(Clone, Copy)]
pub struct PlayerAbilityUpdate {
    pub flying: bool,
}

impl Read for PlayerAbilityUpdate {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        Some(Self {
            flying: buf.u8()? & 2 != 0,
        })
    }
}

#[derive(Clone, Copy)]
pub struct PlayerAction {
    pub action: PlayerActionAction,
    pub pos: BlockPos,
    pub direction: Direction,
    pub sequence: u32,
}

impl Read for PlayerAction {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        Some(Self {
            action: PlayerActionAction::new(buf.u8()?),
            pos: buf.i64()?.into(),
            direction: buf.u8()?.into(),
            sequence: V32::read(buf)?.0,
        })
    }
}

#[derive(Copy, Clone)]
pub enum PlayerActionAction {
    StartDestroyBlock,
    AbortDestroyBlock,
    StopDestroyBlock,
    DropAllItems,
    DropItem,
    ReleaseUseItem,
    SwapItemWithOffhand,
}

impl PlayerActionAction {
    #[inline]
    pub const fn new(id: u8) -> Self {
        if id > 6 {
            Self::StartDestroyBlock
        } else {
            unsafe { transmute(id) }
        }
    }
}

#[derive(Clone, Copy)]
pub struct PlayerInput {
    pub sideways: f32,
    pub forward: f32,
    pub jumping: bool,
    pub sneaking: bool,
}

impl Read for PlayerInput {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        let sideways = buf.f32()?;
        let forward = buf.f32()?;
        let flags = buf.u8()?;
        let jumping = flags & 1 != 0;
        let sneaking = flags & 2 != 0;
        Some(Self {
            sideways,
            forward,
            jumping,
            sneaking,
        })
    }
}

#[derive(Clone, Copy)]
pub struct PlayerInteractBlock {
    pub hand: Hand,
    pub result: BlockHitResult,
    pub sequence: u32,
}

impl Read for PlayerInteractBlock {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        Some(Self {
            hand: buf.u8()?.into(),
            result: Read::read(buf)?,
            sequence: V32::read(buf)?.0,
        })
    }
}

pub struct PlayerInteractionWithEntity {
    pub entity_id: u32,
    pub r#type: PlayerInteractionWithEntityInteractType,
    pub player_sneaking: bool,
}

impl Read for PlayerInteractionWithEntity {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        Some(Self {
            entity_id: V32::read(buf)?.0,
            r#type: Read::read(buf)?,
            player_sneaking: buf.u8()? == 1,
        })
    }
}

#[derive(Copy, Clone)]
pub enum PlayerInteractionWithEntityInteractType {
    Interact(Hand),
    Attack,
    InteractAt { pos: Vec3, hand: Hand },
}

impl Read for PlayerInteractionWithEntityInteractType {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        match buf.u8()? {
            0 => Some(Self::Interact(buf.u8()?.into())),
            1 => Some(Self::Attack),
            _ => Some(Self::InteractAt {
                pos: Vec3 {
                    x: buf.f32()?,
                    y: buf.f32()?,
                    z: buf.f32()?,
                },
                hand: buf.u8()?.into(),
            }),
        }
    }
}

#[derive(Clone, Copy)]
pub struct PlayerInteractItem {
    pub hand: Hand,
    pub sequence: u32,
}

impl Read for PlayerInteractItem {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        Some(Self {
            hand: buf.u8()?.into(),
            sequence: V32::read(buf)?.0,
        })
    }
}

#[derive(Clone, Copy)]
pub struct PlayerMovePositionAndOnGround {
    pub pos: Position,
    pub on_ground: bool,
}

impl Read for PlayerMovePositionAndOnGround {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        Some(Self {
            pos: Read::read(buf)?,
            on_ground: buf.u8()? == 1,
        })
    }
}

#[derive(Clone, Copy)]
pub struct PlayerMoveFull {
    pub pos: Position,
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
}

impl Read for PlayerMoveFull {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        Some(Self {
            pos: Read::read(buf)?,
            yaw: buf.f32()?,
            pitch: buf.f32()?,
            on_ground: buf.u8()? == 1,
        })
    }
}

#[derive(Clone, Copy)]
pub struct PlayerMoveLookAndOnGround {
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
}

impl Read for PlayerMoveLookAndOnGround {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        Some(Self {
            yaw: buf.f32()?,
            pitch: buf.f32()?,
            on_ground: buf.u8()? == 1,
        })
    }
}

#[derive(Clone, Copy)]
pub struct PlayerMoveOnGroundOnly {
    pub on_ground: bool,
}

impl Read for PlayerMoveOnGroundOnly {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        Some(Self {
            on_ground: buf.u8()? == 1,
        })
    }
}

#[derive(Clone, Copy)]
pub struct RecipeBookUpdate {
    pub recipe_id: *const str,
}

impl Read for RecipeBookUpdate {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        Some(Self {
            recipe_id: s(buf, 32767)?,
        })
    }
}

#[derive(Clone, Copy)]
pub struct RecipeCategoryOptionUpdate {
    pub category: RecipeBookCategory,
    pub gui_open: bool,
    pub filtering_craftable: bool,
}

impl Read for RecipeCategoryOptionUpdate {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        Some(Self {
            category: buf.u8()?.into(),
            gui_open: buf.u8()? == 1,
            filtering_craftable: buf.u8()? == 1,
        })
    }
}

#[derive(Copy, Clone)]
pub struct ReconfigurationAcknowledgement;

impl Read for ReconfigurationAcknowledgement {
    fn read(_: &mut &[u8]) -> Option<Self> {
        Some(Self)
    }
}

#[derive(Clone, Copy)]
pub struct UpdateSelectedSlot {
    pub selected_slot: u16,
}

impl Read for UpdateSelectedSlot {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        Some(Self {
            selected_slot: buf.u16()?,
        })
    }
}

#[derive(Clone, Copy)]
pub struct SignUpdate {
    pub pos: BlockPos,
    pub front: bool,
    pub text: [*const str; 4],
}

impl Read for SignUpdate {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        Some(Self {
            pos: buf.i64()?.into(),
            front: buf.u8()? == 1,
            text: [s(buf, 384)?, s(buf, 384)?, s(buf, 384)?, s(buf, 384)?],
        })
    }
}

#[derive(Clone, Copy)]
pub struct ClickSlot {
    pub sync_id: u8,
    pub revision: u32,
    pub slot: u16,
    pub button: u8,
    pub action_type: SlotActionType,
    pub modified_stacks: *mut [(u16, ItemStack)],
    pub stack: ItemStack,
}

impl Read for ClickSlot {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        Some(Self {
            sync_id: buf.u8()?,
            revision: V32::read(buf)?.0,
            slot: buf.u16()?,
            button: buf.u8()?,
            action_type: buf.u8()?.into(),
            modified_stacks: {
                let len = V32::read(buf)?.0 as usize;
                let mut vec = Vec::with_capacity(len);
                for _ in 0..len {
                    vec.push((buf.u16()?, ItemStack::read(buf)?))
                }
                Box::into_raw(vec.into_boxed_slice())
            },
            stack: Read::read(buf)?,
        })
    }
}

#[derive(Clone, Copy)]
pub struct SpectatorTeleportation {
    pub target_uuid: Uuid,
}

impl Read for SpectatorTeleportation {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        Some(Self {
            target_uuid: Uuid::from_bytes(*buf.array()?),
        })
    }
}

#[derive(Clone, Copy)]
pub struct StructureBlockUpdate {
    pub pos: BlockPos,
    pub action: StructureBlockAction,
    pub mode: StructureBlockMode,
    pub structure_name: *const str,
    pub offset: BlockPos,
    pub size: IVec3,
    pub mirror: BlockMirror,
    pub rotation: BlockRotation,
    pub metadata: *const str,
    pub integrity: f32,
    pub seed: u64,
    pub ignore_entities: bool,
    pub show_air: bool,
    pub show_bounding_box: bool,
}

impl Read for StructureBlockUpdate {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        let mut x = Self {
            pos: buf.i64()?.into(),
            action: buf.u8()?.into(),
            mode: buf.u8()?.into(),
            structure_name: s(buf, 128)?,
            offset: BlockPos(IVec3 {
                x: buf.i8()?.clamp(-48, 48) as i32,
                y: buf.i8()?.clamp(-48, 48) as i32,
                z: buf.i8()?.clamp(-48, 48) as i32,
            }),
            size: IVec3 {
                x: buf.i8()?.clamp(0, 48) as i32,
                y: buf.i8()?.clamp(0, 48) as i32,
                z: buf.i8()?.clamp(0, 48) as i32,
            },
            mirror: buf.u8()?.into(),
            rotation: buf.u8()?.into(),
            metadata: s(buf, 32767)?,
            integrity: buf.f32()?.clamp(0.0, 1.0),
            seed: buf.v64()?,
            ignore_entities: false,
            show_air: false,
            show_bounding_box: false,
        };
        let y = buf.u8()?;
        x.ignore_entities = y & 1 != 0;
        x.show_air = y & 2 != 0;
        x.show_bounding_box = y & 4 != 0;

        Some(x)
    }
}

#[derive(Clone, Copy)]
pub struct TeleportConfirmation {
    pub teleport_id: u32,
}

impl Read for TeleportConfirmation {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        Some(Self {
            teleport_id: V32::read(buf)?.0,
        })
    }
}

#[derive(Clone, Copy)]
pub struct VehicleMove {
    pub pos: Position,
    pub yaw: f32,
    pub pitch: f32,
}

impl Read for VehicleMove {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        Some(Self {
            pos: Position::read(buf)?,
            yaw: buf.f32()?,
            pitch: buf.f32()?,
        })
    }
}

#[inline]
fn s(buf: &mut &[u8], l: usize) -> Option<*const str> {
    let len = V32::read(buf)?.0 as usize;
    if len > l * 3 {
        return None;
    }
    match from_utf8(buf.slice(len)?) {
        Ok(x) => Some(x),
        Err(_) => None,
    }
}

#[inline]
fn s2(buf: &mut &[u8]) -> Option<*const [u8]> {
    let len = V32::read(buf)?.0 as usize;
    match buf.slice(len) {
        Some(x) => Some(x),
        None => None,
    }
}

#[inline]
fn o<T: Read>(buf: &mut &[u8]) -> Option<Option<T>> {
    match buf.u8()? == 1 {
        true => Some(Some(T::read(buf)?)),
        false => Some(None),
    }
}
