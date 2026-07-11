use crate::entity::{
    ArmadilloState, CopperGolemState, EntityReference, PaintingVariant, Pose, SnifferState,
    VillagerData,
};
use crate::inventory::HumanoidArm;
use crate::item_stack::OptionalItemStack;
use crate::particle::Particle;
use crate::profile::ResolvableProfileRef;
use crate::registry::{
    CatSoundVariantRef, CatVariantRef, ChickenSoundVariantRef, ChickenVariantRef,
    CowSoundVariantRef, CowVariantRef, FrogVariantRef, PaintingVariantRef, PigSoundVariantRef,
    PigVariantRef, WolfSoundVariantRef, WolfVariantRef, ZombieNautilusVariantRef,
};
use crate::{ComponentRaw, GlobalPos, Holder, OptionalV32, Rotations, WeatheringCopperState};
use haya_collection::List;
use haya_math::{BlockPosPacked, Direction, FQuat, FVec3};
use minecraft_data::block_state;
use mser::Utf8;

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
}

#[derive(Clone, Serialize, Deserialize)]
#[mser(header = EntityDataSerializers, camel_case)]
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
}
