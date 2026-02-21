use crate::stat::Stat;
use crate::{Component, Difficulty, Map};
use haya_math::{BlockPosPacked, ByteAngle, LpVec3, Vec3};
use haya_nbt::Tag;
use minecraft_data::{block, block_entity_type, block_state, entity_type};
use mser::{Error, Read, UnsafeWriter, V32, Write};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize)]
pub struct BundleDelimiter {}

#[derive(Clone, Serialize, Deserialize)]
pub struct AddEntity {
    pub id: V32,
    pub uuid: Uuid,
    pub r#type: entity_type,
    pub pos: Vec3,
    pub movement: LpVec3,
    pub x_rot: ByteAngle,
    pub y_rot: ByteAngle,
    pub y_head_rot: ByteAngle,
    pub data: V32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Animate {
    pub id: V32,
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
    pub stats: Map<'a, Stat, V32>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BlockChangedAck {
    pub sequence: V32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BlockDestruction {
    pub id: V32,
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
    fn read(buf: &mut &'a [u8]) -> Result<Self, Error> {
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
    unsafe fn write(&self, w: &mut UnsafeWriter) {
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
    pub batch_size: V32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ChunkBatchStart {}
