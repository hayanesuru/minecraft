use crate::entity::{
    ArmadilloState, CopperGolemState, EntityReference, PaintingVariant, Pose, SnifferState,
    VillagerData,
};
use crate::item::OptionalItemStack;
use crate::profile::ResolvableProfile;
use crate::registry::{
    CatVariantRef, ChickenVariantRef, CowVariantRef, FrogVariantRef, PaintingVariantRef,
    PigVariantRef, WolfSoundVariantRef, WolfVariantRef, ZombieNautilusVariantRef,
};
use crate::{
    Component, GlobalPos, Holder, HumanoidArm, OptionalV32, Rotations, WeatheringCopperState,
};
use haya_collection::List;
use haya_math::{BlockPosPacked, Direction, FQuat, FVec3};
use minecraft_data::{block_state, particle_type};
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
    CowVariant,
    WolfVariant,
    WolfSoundVariant,
    FrogVariant,
    PigVariant,
    ChickenVariant,
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
    Component(Component),
    OptionalComponent(Option<Component>),
    ItemStack(OptionalItemStack<'a>),
    Boolean(bool),
    Rotations(Rotations),
    BlockPos(BlockPosPacked),
    OptionalBlockPos(Option<BlockPosPacked>),
    Direction(Direction),
    OptionalLivingEntityReference(Option<EntityReference>),
    BlockState(block_state),
    OptionalBlockState(block_state),
    Particle(particle_type),
    Particles(List<'a, particle_type>),
    VillagerData(VillagerData),
    OptionalUnsignedInt(OptionalV32),
    Pose(Pose),
    CatVariant(CatVariantRef),
    CowVariant(CowVariantRef),
    WolfVariant(WolfVariantRef),
    WolfSoundVariant(WolfSoundVariantRef),
    FrogVariant(FrogVariantRef),
    PigVariant(PigVariantRef),
    ChickenVariant(ChickenVariantRef),
    ZombieNautilusVariant(ZombieNautilusVariantRef),
    OptionalGlobalPos(Option<GlobalPos<'a>>),
    PaintingVariant(Holder<PaintingVariant<'a>, PaintingVariantRef>),
    SnifferState(SnifferState),
    ArmadilloState(ArmadilloState),
    CopperGolemState(CopperGolemState),
    WeatheringCopperState(WeatheringCopperState),
    Vector3(FVec3),
    Quaternion(FQuat),
    ResolvableProfile(ResolvableProfile<'a>),
    HumanoidArm(HumanoidArm),
}
