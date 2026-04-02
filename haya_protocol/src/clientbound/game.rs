use crate::command::CommandNode;
use crate::debug::DebugSubscriptionUpdate;
use crate::item::OptionalItemStack;
use crate::registry::DamageTypeRef;
use crate::stat::Stat;
use crate::{Component, ContainerId, Difficulty};
use haya_collection::{List, Map};
use haya_ident::Ident;
use haya_math::{BlockPosPacked, ByteAngle, ChunkPos, LpVec3, Vec3};
use haya_nbt::Tag;
use minecraft_data::{block, block_entity_type, block_state, entity_type};
use mser::{ByteArray, Error, Read, Reader, Utf8, V21, Write, Writer};
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

#[derive(Clone)]
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

impl BossEventOperation {
    pub const fn to_type(&self) -> BossEventOperationType {
        match self {
            Self::Add { .. } => BossEventOperationType::Add,
            Self::Remove {} => BossEventOperationType::Remove,
            Self::UpdateProgress { .. } => BossEventOperationType::UpdateProgress,
            Self::UpdateName { .. } => BossEventOperationType::UpdateName,
            Self::UpdateStyle { .. } => BossEventOperationType::UpdateStyle,
            Self::UpdateProperties { .. } => BossEventOperationType::UpdateProperties,
        }
    }
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

impl<'a> Read<'a> for BossEventOperation {
    fn read(buf: &mut Reader<'a>) -> Result<Self, Error> {
        Ok(match BossEventOperationType::read(buf)? {
            BossEventOperationType::Add => Self::Add {
                name: Component::read(buf)?,
                progress: f32::read(buf)?,
                color: BossEventColor::read(buf)?,
                overlay: BossEventOverlay::read(buf)?,
                flags: BossEventFlags::read(buf)?,
            },
            BossEventOperationType::Remove => Self::Remove {},
            BossEventOperationType::UpdateProgress => Self::UpdateProgress {
                progress: f32::read(buf)?,
            },
            BossEventOperationType::UpdateName => Self::UpdateName {
                name: Component::read(buf)?,
            },
            BossEventOperationType::UpdateStyle => Self::UpdateStyle {
                color: BossEventColor::read(buf)?,
                overlay: BossEventOverlay::read(buf)?,
            },
            BossEventOperationType::UpdateProperties => Self::UpdateProperties {
                flags: BossEventFlags::read(buf)?,
            },
        })
    }
}

impl Write for BossEventOperation {
    unsafe fn write(&self, w: &mut Writer) {
        unsafe {
            self.to_type().write(w);
            match self {
                Self::Add {
                    name,
                    progress,
                    color,
                    overlay,
                    flags,
                } => {
                    name.write(w);
                    progress.write(w);
                    color.write(w);
                    overlay.write(w);
                    flags.write(w);
                }
                Self::Remove {} => {}
                Self::UpdateProgress { progress } => {
                    progress.write(w);
                }
                Self::UpdateName { name } => {
                    name.write(w);
                }
                Self::UpdateStyle { color, overlay } => {
                    color.write(w);
                    overlay.write(w);
                }
                Self::UpdateProperties { flags } => {
                    flags.write(w);
                }
            }
        }
    }

    fn len_s(&self) -> usize {
        let ty = self.to_type().len_s();
        let len = match self {
            Self::Add {
                name,
                progress,
                color,
                overlay,
                flags,
            } => name.len_s() + progress.len_s() + color.len_s() + overlay.len_s() + flags.len_s(),
            Self::Remove {} => 0,
            Self::UpdateProgress { progress } => progress.len_s(),
            Self::UpdateName { name } => name.len_s(),
            Self::UpdateStyle { color, overlay } => color.len_s() + overlay.len_s(),
            Self::UpdateProperties { flags } => flags.len_s(),
        };
        ty + len
    }
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
