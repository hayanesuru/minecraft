use crate::{Component, Holder, HolderSet, Rarity};
use haya_collection::{List, Map};
use haya_ident::{Ident, ResourceKey};
use haya_nbt::Tag;
use minecraft_data::item;
use mser::{Either, V32};

#[derive(Clone)]
pub struct ItemStack {
    pub id: item,
    pub count: u32,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct UseEffects {
    pub can_sprint: bool,
    pub interact_vibrations: bool,
    pub speed_multiplier: f32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CustomData(pub Tag);

#[derive(Clone, Serialize, Deserialize)]
pub struct ItemLore<'a>(pub List<'a, Component, 256>);

#[derive(Clone, Serialize, Deserialize)]
pub struct ItemEnchantments<'a>(pub Map<'a, Holder, V32>);

#[derive(Clone, Serialize, Deserialize)]
pub struct CanPlaceOn<'a> {
    pub predicates: Option<HolderSet<'a>>,
}

#[derive(Clone)]
pub enum DataComponentType<'a> {
    CustomData(CustomData),
    MaxStackSize(u32),
    MaxDamage(u32),
    Damage(u32),
    Unbreakable,
    UseEffects(UseEffects),
    CustomName(Component),
    MinimumAttackCharge(f32),
    DamageType(Either<Holder, ResourceKey<'a>>),
    ItemName(Component),
    ItemModel(Ident<'a>),
    Lore(ItemLore<'a>),
    Rarity(Rarity),
    Enchantments(ItemEnchantments<'a>),
    CanPlaceOn,
    CanBreak,
    AttributeModifiers,
    CustomModelData,
    TooltipDisplay,
    RepairCost,
    CreativeSlotLock,
    EnchantmentGlintOverride,
    IntangibleProjectile,
    Food,
    Consumable,
    UseRemainder,
    UseCooldown,
    DamageResistant,
    Tool,
    Weapon,
    AttackRange,
    Enchantable,
    Equippable,
    Repairable,
    Glider,
    TooltipStyle,
    DeathProtection,
    BlocksAttacks,
    PiercingWeapon,
    KineticWeapon,
    SwingAnimation,
    StoredEnchantments,
    DyedColor,
    MapColor,
    MapId,
    MapDecorations,
    MapPostProcessing,
    ChargedProjectiles,
    BundleContents,
    PotionContents,
    PotionDurationScale,
    SuspiciousStewEffects,
    WritableBookContent,
    WrittenBookContent,
    Trim,
    DebugStickState,
    EntityData,
    BucketEntityData,
    BlockEntityData,
    Instrument,
    ProvidesTrimMaterial,
    OminousBottleAmplifier,
    JukeboxPlayable,
    ProvidesBannerPatterns,
    Recipes,
    LodestoneTracker,
    FireworkExplosion,
    Fireworks,
    Profile,
    NoteBlockSound,
    BannerPatterns,
    BaseColor,
    PotDecorations,
    Container,
    BlockState,
    Bees,
    Lock,
    ContainerLoot,
    BreakSound,
    VillagerVariant,
    WolfVariant,
    WolfSoundVariant,
    WolfCollar,
    FoxVariant,
    SalmonSize,
    ParrotVariant,
    TropicalFishPattern,
    TropicalFishBaseColor,
    TropicalFishPatternColor,
    MooshroomVariant,
    RabbitVariant,
    PigVariant,
    CowVariant,
    ChickenVariant,
    ZombieNautilusVariant,
    FrogVariant,
    HorseVariant,
    PaintingVariant,
    LlamaVariant,
    AxolotlVariant,
    CatVariant,
    CatCollar,
    SheepColor,
    ShulkerColor,
}
