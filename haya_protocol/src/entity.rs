use crate::inventory::HumanoidArm;
use crate::item_stack::OptionalItemStack;
use crate::particle::Particle;
use crate::profile::ResolvableProfileRef;
use crate::registry::{
    CatSoundVariantRef, CatVariantRef, ChickenSoundVariantRef, ChickenVariantRef,
    CowSoundVariantRef, CowVariantRef, FrogVariantRef, PaintingVariantRef, PigSoundVariantRef,
    PigVariantRef, WolfSoundVariantRef, WolfVariantRef, ZombieNautilusVariantRef,
};
use crate::{
    ComponentRaw, DyeColor, GlobalPos, Holder, OptionalV32, Rotations, WeatheringCopperState,
};
use haya_collection::List;
use haya_ident::Ident;
use haya_math::{BlockPosPacked, Direction, FQuat, FVec3};
use minecraft_data::{block_state, villager_profession, villager_type};
use mser::{Read, Utf8, V21, Write};
use uuid::Uuid;

#[derive(Clone, Copy, Serialize, Deserialize, Name)]
#[repr(u8)]
#[mser(varint)]
pub enum FoxVariant {
    Red,
    Snow,
}

#[derive(Clone, Copy, Serialize, Deserialize, Name)]
#[repr(u8)]
#[mser(varint)]
pub enum SalmonVariant {
    Small,
    Medium,
    Large,
}

#[derive(Clone, Copy, Serialize, Deserialize, Name)]
#[repr(u8)]
#[mser(varint)]
pub enum ParrotVariant {
    RedBlue,
    Blue,
    Green,
    YellowBlue,
    Gray,
}

#[derive(Clone, Copy, Serialize, Deserialize, Name)]
#[repr(u8)]
#[mser(varint)]
pub enum TropicalFishPattern {
    Kob,
    Sunstreak,
    Snooper,
    Dasher,
    Brinely,
    Spotty,
    Flopper,
    Stripey,
    Glitter,
    Blockfish,
    Betty,
    Clayfish,
}

#[derive(Clone, Copy, Serialize, Deserialize, Name)]
#[repr(u8)]
#[mser(varint)]
pub enum MushroomCowVariant {
    Red,
    Brown,
}

#[derive(Clone, Copy, Name)]
#[repr(u8)]
#[mser(varint)]
pub enum RabbitVariant {
    Brown,
    White,
    Black,
    WhiteSplotched,
    Gold,
    Salt,
    Evil = 99,
}

impl Write for RabbitVariant {
    unsafe fn write(&self, w: &mut mser::Writer) {
        unsafe { w.write_byte(*self as u8) }
    }

    fn len_s(&self) -> usize {
        1
    }
}

impl<'a> Read<'a> for RabbitVariant {
    fn read(buf: &mut mser::Reader<'a>) -> Result<Self, mser::Error> {
        Ok(match V21::read(buf)?.0 {
            1 => Self::White,
            2 => Self::Black,
            3 => Self::WhiteSplotched,
            4 => Self::Gold,
            5 => Self::Salt,
            99 => Self::Evil,
            _ => Self::Brown,
        })
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, Name)]
#[repr(u8)]
#[mser(varint)]
pub enum EquineVariant {
    White,
    Creamy,
    Chestnut,
    Brown,
    Black,
    Gray,
    DarkBrown,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PaintingVariant<'a> {
    #[mser(varint)]
    pub width: u32,
    #[mser(varint)]
    pub height: u32,
    pub asset_id: Ident<'a>,
    pub title: Option<ComponentRaw>,
    pub author: Option<ComponentRaw>,
}

#[derive(Clone, Copy, Serialize, Deserialize, Name)]
#[repr(u8)]
#[mser(varint)]
pub enum LlamaVariant {
    Creamy,
    White,
    Brown,
    Gray,
}

#[derive(Clone, Copy, Serialize, Deserialize, Name)]
#[repr(u8)]
#[mser(varint)]
pub enum AxolotlVariant {
    Lucy,
    Wild,
    Gold,
    Cyan,
    Blue,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct EntityReference {
    pub entity: Uuid,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct VillagerData {
    pub ty: villager_type,
    pub profession: villager_profession,
    #[mser(varint)]
    pub level: u32,
}

#[derive(Clone, Copy, Serialize, Deserialize, Name)]
#[repr(u8)]
#[mser(varint)]
pub enum ArmadilloState {
    Idle,
    Rolling,
    Scared,
    Unrolling,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum SnifferState {
    Idling,
    FeelingHappy,
    Scenting,
    Sniffing,
    Searching,
    Digging,
    Rising,
}

#[derive(Clone, Copy, Serialize, Deserialize, Name)]
#[repr(u8)]
#[mser(varint)]
pub enum CopperGolemState {
    Idle,
    GettingItem,
    GettingNoItem,
    DroppingItem,
    DroppingNoItem,
}

#[derive(Clone, Copy, Serialize, Deserialize, Name)]
#[repr(u8)]
#[mser(varint)]
pub enum Pose {
    Standing,
    FallFlying,
    Sleeping,
    Swimming,
    SpinAttack,
    Crouching,
    LongJumping,
    Dying,
    Croaking,
    UsingTongue,
    Sitting,
    Roaring,
    Sniffing,
    Emerging,
    Digging,
    Sliding,
    Shooting,
    Inhaling,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum EntityDataSerializers {
    Byte,
    Int,
    Long,
    Float,
    String,
    Component,
    OptionalComponent,
    ItemStack,
    Boolean,
    Rotations,
    BlockPos,
    OptionalBlockPos,
    Direction,
    OptionalLivingEntityReference,
    BlockState,
    OptionalBlockState,
    Particle,
    Particles,
    VillagerData,
    OptionalUnsignedInt,
    Pose,
    CatVariant,
    CatSoundVariant,
    CowVariant,
    CowSoundVariant,
    WolfVariant,
    WolfSoundVariant,
    FrogVariant,
    PigVariant,
    PigSoundVariant,
    ChickenVariant,
    ChickenSoundVariant,
    ZombieNautilusVariant,
    OptionalGlobalPos,
    PaintingVariant,
    SnifferState,
    ArmadilloState,
    CopperGolemState,
    WeatheringCopperState,
    Vector3,
    Quaternion,
    ResolvableProfile,
    HumanoidArm,
    DyeColor,
}

#[derive(Clone, Serialize, Deserialize)]
#[mser(header = EntityDataSerializers, PascalCase)]
pub enum EntityDataSerializer<'a> {
    Byte(u8),
    Int(#[mser(varint)] u32),
    Long(#[mser(varint)] u64),
    Float(f32),
    String(Utf8<'a>),
    Component(ComponentRaw),
    OptionalComponent(Option<ComponentRaw>),
    ItemStack(OptionalItemStack<'a>),
    Boolean(bool),
    Rotations(Rotations),
    BlockPos(BlockPosPacked),
    OptionalBlockPos(Option<BlockPosPacked>),
    Direction(Direction),
    OptionalLivingEntityReference(Option<EntityReference>),
    BlockState(block_state),
    OptionalBlockState(block_state),
    Particle(Particle<'a>),
    Particles(List<'a, Particle<'a>>),
    VillagerData(VillagerData),
    OptionalUnsignedInt(OptionalV32),
    Pose(Pose),
    CatVariant(CatVariantRef),
    CatSoundVariant(CatSoundVariantRef),
    CowVariant(CowVariantRef),
    CowSoundVariant(CowSoundVariantRef),
    WolfVariant(WolfVariantRef),
    WolfSoundVariant(WolfSoundVariantRef),
    FrogVariant(FrogVariantRef),
    PigVariant(PigVariantRef),
    PigSoundVariant(PigSoundVariantRef),
    ChickenVariant(ChickenVariantRef),
    ChickenSoundVariant(ChickenSoundVariantRef),
    ZombieNautilusVariant(ZombieNautilusVariantRef),
    OptionalGlobalPos(Option<GlobalPos<'a>>),
    PaintingVariant(Holder<PaintingVariant<'a>, PaintingVariantRef>),
    SnifferState(SnifferState),
    ArmadilloState(ArmadilloState),
    CopperGolemState(CopperGolemState),
    WeatheringCopperState(WeatheringCopperState),
    Vector3(FVec3),
    Quaternion(FQuat),
    ResolvableProfile(ResolvableProfileRef<'a>),
    HumanoidArm(HumanoidArm),
    DyeColor(DyeColor),
}
