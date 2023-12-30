use crate::inventory::ItemStack;
use crate::math::BlockPos;
use crate::{UnsafeWriter, Write, V32};

#[derive(Writable, Clone, Copy)]
#[repr(u8)]
pub enum DataType {
    Byte,
    Integer,
    Long,
    Float,
    String,
    TextComponent,
    OptionalTextComponent,
    ItemStack,
    Boolean,
    Rotation,
    BlockPos,
    OptionalBlockPos,
    Direction,
    OptionalUuid,
    BlockState,
    OptionalBlockState,
    TagCompound,
    Particle,
    VillagerData,
    OptionalUnsignedInt,
    EntityPose,
    CatVariant,
    FrogVariant,
    OptionalGlobalPos,
    PaintingVariant,
    SnifferState,
    Vertor3,
    Quaternion,
}

#[derive(Clone, Copy)]
pub(crate) enum Data {
    Byte(u8),
    Integer(V32),
    ItemStack(ItemStack),
    OptionalBlockPos(Option<i64>),
}

impl Write for Data {
    fn write(&self, w: &mut UnsafeWriter) {
        match self {
            Self::Byte(x) => {
                DataType::Byte.write(w);
                w.write_byte(*x)
            }
            Self::Integer(x) => {
                DataType::Integer.write(w);
                x.write(w);
            }
            Self::ItemStack(x) => {
                DataType::Integer.write(w);
                x.write(w);
            }
            Self::OptionalBlockPos(x) => {
                DataType::OptionalBlockPos.write(w);
                match x {
                    Some(x) => {
                        w.write_byte(1);
                        x.write(w);
                    }
                    None => w.write_byte(0),
                }
            }
        }
    }

    fn len(&self) -> usize {
        match self {
            Self::Byte(_) => DataType::Byte.len() + 1,
            Self::Integer(x) => DataType::Integer.len() + x.len(),
            Self::ItemStack(x) => DataType::ItemStack.len() + x.len(),
            Self::OptionalBlockPos(x) => {
                DataType::OptionalBlockPos.len() + if x.is_some() { 9 } else { 1 }
            }
        }
    }
}

#[derive(Clone, Copy)]
pub struct TrackedData {
    pub(crate) index: u8,
    pub(crate) data: Data,
}

impl Write for TrackedData {
    #[inline]
    fn write(&self, w: &mut UnsafeWriter) {
        self.index.write(w);
        self.data.write(w);
    }

    #[inline]
    fn len(&self) -> usize {
        self.index.len() + self.data.len()
    }
}

impl TrackedData {
    #[inline]
    pub const fn shared_flags(flags: u8) -> Self {
        Self {
            index: 0,
            data: Data::Byte(flags),
        }
    }

    #[inline]
    pub const fn air_supply(supply: u32) -> Self {
        Self {
            index: 1,
            data: Data::Integer(V32(supply)),
        }
    }

    #[inline]
    pub const fn item_stack(stack: ItemStack) -> Self {
        Self {
            index: 8,
            data: Data::ItemStack(stack),
        }
    }

    #[inline]
    pub const fn sleeping_pos(pos: Option<BlockPos>) -> Self {
        Self {
            index: 14,
            data: Data::OptionalBlockPos(match pos {
                Some(x) => Some(x.to_i64()),
                None => None,
            }),
        }
    }
}
