use crate::game_event::PositionSource;
use crate::item::ItemStack;
use haya_math::Vec3;
use minecraft_data::{block_state, particle_type};

#[derive(Clone, Serialize, Deserialize)]
#[mser(header = particle_type)]
pub enum Particle<'a> {
    AngryVillager,
    Block(BlockParticleOption),
    BlockMarker(BlockParticleOption),
    Bubble,
    Cloud,
    CopperFireFlame,
    Crit,
    DamageIndicator,
    DragonBreath(PowerParticleOption),
    DrippingLava,
    FallingLava,
    LandingLava,
    DrippingWater,
    FallingWater,
    Dust(DustParticleOptions),
    DustColorTransition(DustColorTransitionOptions),
    Effect(SpellParticleOption),
    ElderGuardian,
    EnchantedHit,
    Enchant,
    EndRod,
    EntityEffect(ColorParticleOption),
    ExplosionEmitter,
    Explosion,
    Gust,
    SmallGust,
    GustEmitterLarge,
    GustEmitterSmall,
    SonicBoom,
    FallingDust(BlockParticleOption),
    Firework,
    Fishing,
    Flame,
    Infested,
    CherryLeaves,
    PaleOakLeaves,
    TintedLeaves(ColorParticleOption),
    SculkSoul,
    SculkCharge(SculkChargeParticleOptions),
    SculkChargePop,
    SoulFireFlame,
    Soul,
    Flash(ColorParticleOption),
    HappyVillager,
    Composter,
    Heart,
    InstantEffect(SpellParticleOption),
    Item(ItemParticleOption<'a>),
    Vibration(VibrationParticleOption),
    Trail(TrailParticleOption),
    ItemSlime,
    ItemCobweb,
    ItemSnowball,
    LargeSmoke,
    Lava,
    Mycelium,
    Note,
    Poof,
    Portal,
    Rain,
    Smoke,
    WhiteSmoke,
    Sneeze,
    Spit,
    SquidInk,
    SweepAttack,
    TotemOfUndying,
    Underwater,
    Splash,
    Witch,
    BubblePop,
    CurrentDown,
    BubbleColumnUp,
    Nautilus,
    Dolphin,
    CampfireCosySmoke,
    CampfireSignalSmoke,
    DrippingHoney,
    FallingHoney,
    LandingHoney,
    FallingNectar,
    FallingSporeBlossom,
    Ash,
    CrimsonSpore,
    WarpedSpore,
    SporeBlossomAir,
    DrippingObsidianTear,
    FallingObsidianTear,
    LandingObsidianTear,
    ReversePortal,
    WhiteAsh,
    SmallFlame,
    Snowflake,
    DrippingDripstoneLava,
    FallingDripstoneLava,
    DrippingDripstoneWater,
    FallingDripstoneWater,
    GlowSquidInk,
    Glow,
    WaxOn,
    WaxOff,
    ElectricSpark,
    Scrape,
    Shriek(ShriekParticleOption),
    EggCrack,
    DustPlume,
    TrialSpawnerDetection,
    TrialSpawnerDetectionOminous,
    VaultConnection,
    DustPillar(BlockParticleOption),
    OminousSpawning,
    RaidOmen,
    TrialOmen,
    BlockCrumble(BlockParticleOption),
    Firefly,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BlockParticleOption {
    pub state: block_state,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PowerParticleOption {
    pub power: f32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DustParticleOptions {
    pub color: u32,
    pub scale: f32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DustColorTransitionOptions {
    pub from_color: u32,
    pub to_color: u32,
    pub scale: f32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SpellParticleOption {
    pub color: u32,
    pub power: f32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ColorParticleOption {
    pub color: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SculkChargeParticleOptions {
    pub roll: f32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ItemParticleOption<'a> {
    pub item: ItemStack<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct VibrationParticleOption {
    pub destination: PositionSource,
    #[mser(varint)]
    pub arrival_in_ticks: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TrailParticleOption {
    pub target: Vec3,
    pub color: u32,
    #[mser(varint)]
    pub duration: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ShriekParticleOption {
    #[mser(varint)]
    pub delay: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ExplosionParticleInfo<'a> {
    pub particle: Particle<'a>,
    pub scaling: f32,
    pub speed: f32,
}
