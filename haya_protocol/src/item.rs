pub mod consume_effect;
pub mod firework_explosion;
pub mod item_attribute_modifiers;
pub mod item_enchantments;
pub mod kinetic_weapon;
pub mod suspicious_stew_effects;
pub mod tool;

use crate::advancement::BlockPredicate;
use crate::block::{BannerPatternLayers, BeehiveOccupant};
use crate::effect::MobEffect;
use crate::entity::{
    AxolotlVariant, EquineVariant, FoxVariant, LlamaVariant, MushroomCowVariant, PaintingVariant,
    ParrotVariant, RabbitVariant, SalmonVariant, TropicalFishPattern,
};
use crate::food::FoodProperties;
use crate::item::consume_effect::ConsumeEffect;
use crate::item::firework_explosion::FireworkExplosion;
use crate::item::item_attribute_modifiers::ItemAttributeModifiers;
use crate::item::item_enchantments::ItemEnchantments;
use crate::item::kinetic_weapon::KineticWeapon;
use crate::item::suspicious_stew_effects::SuspiciousStewEffects;
use crate::item::tool::Tool;
use crate::map::MapId;
use crate::profile::ResolvableProfile;
use crate::registry::{
    CatVariantRef, ChickenVariantRef, CowVariantRef, DamageTypeRef, FrogVariantRef, InstrumentRef,
    JukeboxSongRef, PaintingVariantRef, PigVariantRef, SoundEventRef, TrimMaterialRef,
    TrimPatternRef, VillagerTypeRef, WolfSoundVariantRef, WolfVariantRef, ZombieNautilusVariantRef,
};
use crate::sound::SoundEvent;
use crate::trim::{TrimMaterial, TrimPattern};
use crate::{Component, DyeColor, EquipmentSlot, Filterable, Holder, HolderSet, LockCode, Rarity};
use alloc::vec::Vec;
use haya_collection::{List, Map, capacity_fix};
use haya_ident::{Ident, ResourceKey, TagKey};
use haya_math::BlockPosPacked;
use haya_nbt::Tag;
use minecraft_data::{block_entity_type, data_component_type, entity_type, item, potion};
use mser::{Either, Error, Read, Reader, Utf8, V21, V32, Write, Writer};

#[derive(Clone, Serialize, Deserialize)]
pub struct ItemStack<'a>(#[mser(filter = validate_item_stack)] pub OptionalItemStack<'a>);

fn validate_item_stack(item_stack: &OptionalItemStack<'_>) -> bool {
    !(item_stack.count == 0 || item_stack.id == item::air)
}

#[derive(Clone)]
pub struct OptionalItemStack<'a> {
    pub id: item,
    pub count: u32,
    pub patch_add: List<'a, TypedDataComponent<'a>>,
    pub patch_remove: List<'a, data_component_type>,
}

impl<'a> Read<'a> for OptionalItemStack<'a> {
    fn read(buf: &mut Reader<'a>) -> Result<Self, Error> {
        let count = V32::read(buf)?.0;
        if count as i32 <= 0 {
            Ok(Self {
                id: item::air,
                count: 0,
                patch_add: List::Borrowed(&[]),
                patch_remove: List::Borrowed(&[]),
            })
        } else {
            let id = item::read(buf)?;
            let positive = V32::read(buf)?.0 as usize;
            let negative = V32::read(buf)?.0 as usize;
            if positive == 0 && negative == 0 {
                Ok(Self {
                    id,
                    count,
                    patch_add: List::Borrowed(&[]),
                    patch_remove: List::Borrowed(&[]),
                })
            } else {
                let mut patch_add = Vec::with_capacity(capacity_fix(positive));
                for _ in 0..positive {
                    patch_add.push(TypedDataComponent::read(buf)?);
                }
                let mut patch_remove = Vec::with_capacity(capacity_fix(negative));
                for _ in 0..negative {
                    patch_remove.push(data_component_type::read(buf)?);
                }
                Ok(Self {
                    id,
                    count,
                    patch_add: List::Owned(patch_add),
                    patch_remove: List::Owned(patch_remove),
                })
            }
        }
    }
}

impl<'a> Write for OptionalItemStack<'a> {
    unsafe fn write(&self, w: &mut Writer) {
        unsafe {
            if self.count == 0 || self.id == item::air {
                V32(0).write(w);
            } else {
                V32(self.count).write(w);
                self.id.write(w);
                V21(self.patch_add.len() as u32).write(w);
                V21(self.patch_remove.len() as u32).write(w);
                if !self.patch_add.is_empty() || !self.patch_remove.is_empty() {
                    for x in self.patch_add.as_slice() {
                        x.write(w);
                    }
                    for x in self.patch_remove.as_slice() {
                        x.write(w);
                    }
                }
            }
        }
    }

    fn len_s(&self) -> usize {
        if self.count == 0 || self.id == item::air {
            V32(0).len_s()
        } else {
            let mut w = 0;
            w += V32(self.count).len_s();
            w += self.id.len_s();
            w += V21(self.patch_add.len() as u32).len_s();
            w += V21(self.patch_remove.len() as u32).len_s();
            if !self.patch_add.is_empty() || !self.patch_remove.is_empty() {
                for x in self.patch_add.as_slice() {
                    w += x.len_s();
                }
                for x in self.patch_remove.as_slice() {
                    w += x.len_s();
                }
            }
            w
        }
    }
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
pub struct AdventureModePredicate<'a> {
    pub predicates: List<'a, BlockPredicate<'a>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CustomModelData<'a> {
    pub floats: List<'a, f32>,
    pub flags: List<'a, bool>,
    pub strings: List<'a, Utf8<'a>>,
    pub colors: List<'a, u32>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TooltipDisplay<'a> {
    pub hide_tooltip: bool,
    pub hidden_components: List<'a, TypedDataComponent<'a>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Consumable<'a> {
    pub consume_seconds: f32,
    pub animation: ItemUseAnimation,
    pub sound: Holder<SoundEvent<'a>, SoundEventRef>,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[mser(varint)]
#[repr(u8)]
pub enum ItemUseAnimation {
    None,
    Eat,
    Drink,
    Block,
    Bow,
    Trident,
    Crossbow,
    Spyglass,
    TootHorn,
    Brush,
    Bundle,
    Spear,
}

impl ItemUseAnimation {
    pub const fn name(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Eat => "eat",
            Self::Drink => "drink",
            Self::Block => "block",
            Self::Bow => "bow",
            Self::Trident => "trident",
            Self::Crossbow => "crossbow",
            Self::Spyglass => "spyglass",
            Self::TootHorn => "toot_horn",
            Self::Brush => "brush",
            Self::Bundle => "bundle",
            Self::Spear => "spear",
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct UseRemainder<'a> {
    pub convert_into: OptionalItemStack<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct UseCooldown<'a> {
    pub seconds: f32,
    pub cooldown_group: Option<Ident<'a>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DamageResistant<'a> {
    pub types: TagKey<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Weapon {
    #[mser(varint)]
    pub item_damage_per_attack: u32,
    pub disable_blocking_for_seconds: f32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AttackRange {
    pub min_range: f32,
    pub max_range: f32,
    pub min_creative_range: f32,
    pub max_creative_range: f32,
    pub hitbox_margin: f32,
    pub mob_factor: f32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Enchantable {
    #[mser(varint, filter = validate_enchantable)]
    pub value: i32,
}

fn validate_enchantable(value: &i32) -> bool {
    *value > 0
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Equippable<'a> {
    pub slot: EquipmentSlot,
    pub equip_sound: Holder<SoundEvent<'a>, SoundEventRef>,
    pub asset_id: Option<Ident<'a>>,
    pub camera_overlay: Option<Ident<'a>>,
    pub allowed_entities: Option<HolderSet<'a, entity_type>>,
    pub dispensable: bool,
    pub swappable: bool,
    pub damage_on_hurt: bool,
    pub equip_on_interact: bool,
    pub can_be_sheared: bool,
    pub shearing_sound: Holder<SoundEvent<'a>, SoundEventRef>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Repairable<'a> {
    pub items: HolderSet<'a, item>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DeathProtection<'a> {
    pub death_effects: List<'a, ConsumeEffect<'a>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BlocksAttacks<'a> {
    pub block_delay_seconds: f32,
    pub disable_cooldown_scale: f32,
    pub damage_reductions: List<'a, DamageReduction<'a>>,
    pub item_damage: ItemDamageFunction,
    pub bypassed_by: Option<TagKey<'a>>,
    pub block_sound: Option<Holder<SoundEvent<'a>, SoundEventRef>>,
    pub disable_sound: Option<Holder<SoundEvent<'a>, SoundEventRef>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DamageReduction<'a> {
    pub horizontal_blocking_angle: f32,
    pub ty: Option<HolderSet<'a, DamageTypeRef>>,
    pub base: f32,
    pub factor: f32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ItemDamageFunction {
    pub threshold: f32,
    pub base: f32,
    pub factor: f32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PiercingWeapon<'a> {
    pub deals_knockback: bool,
    pub dismounts: bool,
    pub sound: Option<Holder<SoundEvent<'a>, SoundEventRef>>,
    pub hit_sound: Option<Holder<SoundEvent<'a>, SoundEventRef>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SwingAnimation {
    pub ty: SwingAnimationType,
    #[mser(varint)]
    pub duration: u32,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum SwingAnimationType {
    None,
    Whack,
    Stab,
}

impl SwingAnimationType {
    pub const fn name(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Whack => "whack",
            Self::Stab => "stab",
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct DyedItemColor(pub u32);

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct MapItemColor(pub u32);

#[derive(Clone, Serialize, Deserialize)]
pub struct MapDecorations {
    pub tag: Tag,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum MapPostProcessing {
    Lock,
    Scale,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ChargedProjectiles<'a> {
    pub items: List<'a, ItemStack<'a>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BundleContents<'a> {
    pub items: List<'a, ItemStack<'a>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PotionContents<'a> {
    pub potion: Option<potion>,
    pub custom_color: Option<u32>,
    pub custom_effects: List<'a, MobEffect<'a>>,
    pub custom_name: Option<Utf8<'a>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct WritableBookContent<'a> {
    pub pages: List<'a, Filterable<Utf8<'a, 1024>>, 100>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct WrittenBookContent<'a> {
    pub title: Filterable<Utf8<'a, 32>>,
    pub author: Utf8<'a>,
    #[mser(varint, filter = validate_written_book_content)]
    pub generation: u32,
    pub pages: List<'a, Filterable<Component>>,
    pub resolved: bool,
}

fn validate_written_book_content(generation: &u32) -> bool {
    (0..=3).contains(generation)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ArmorTrim<'a> {
    pub material: Holder<TrimMaterial<'a>, TrimMaterialRef>,
    pub pattern: Holder<TrimPattern<'a>, TrimPatternRef>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DebugStickState {
    pub tag: Tag,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TypedEntityDataEntity {
    pub ty: entity_type,
    pub tag: Tag,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TypedEntityDataBlockEntity {
    pub ty: block_entity_type,
    pub tag: Tag,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Instrument<'a> {
    pub sound_event: Holder<SoundEvent<'a>, SoundEventRef>,
    pub use_duration: f32,
    pub range: f32,
    pub description: Component,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ProvidesTrimMaterial<'a> {
    pub material: Either<TrimMaterial<'a>, ResourceKey<'a>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct OminousBottleAmplifier {
    #[mser(varint)]
    pub value: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct JukeboxPlayable<'a> {
    pub song: Either<Holder<JukeboxSong<'a>, JukeboxSongRef>, ResourceKey<'a>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct JukeboxSong<'a> {
    pub sound_event: Holder<SoundEvent<'a>, SoundEventRef>,
    pub description: Component,
    pub length_in_seconds: f32,
    #[mser(varint)]
    pub comparator_output: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Recipes(pub Tag);

#[derive(Clone, Serialize, Deserialize)]
pub struct LodestoneTracker {
    pub target: Option<BlockPosPacked>,
    pub tracked: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Fireworks<'a> {
    #[mser(varint)]
    pub flight_duration: u32,
    pub explosions: List<'a, FireworkExplosion<'a>, 256>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ItemContainerContents<'a> {
    pub items: List<'a, OptionalItemStack<'a>, 256>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BlockItemStateProperties<'a> {
    pub properties: Map<'a, Utf8<'a>, Utf8<'a>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Bees<'a> {
    pub bees: List<'a, BeehiveOccupant>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SeededContainerLoot(pub Tag);

#[derive(Clone, Serialize, Deserialize)]
#[mser(header = data_component_type)]
pub enum TypedDataComponent<'a> {
    CustomData(CustomData),
    MaxStackSize(#[mser(varint)] u32),
    MaxDamage(#[mser(varint)] u32),
    Damage(#[mser(varint)] u32),
    Unbreakable,
    UseEffects(UseEffects),
    CustomName(Component),
    MinimumAttackCharge(f32),
    DamageType(Either<DamageTypeRef, ResourceKey<'a>>),
    ItemName(Component),
    ItemModel(Ident<'a>),
    Lore(ItemLore<'a>),
    Rarity(Rarity),
    Enchantments(ItemEnchantments<'a>),
    CanPlaceOn(AdventureModePredicate<'a>),
    CanBreak(AdventureModePredicate<'a>),
    AttributeModifiers(ItemAttributeModifiers<'a>),
    CustomModelData(CustomModelData<'a>),
    TooltipDisplay(TooltipDisplay<'a>),
    RepairCost(#[mser(varint)] u32),
    CreativeSlotLock,
    EnchantmentGlintOverride(bool),
    IntangibleProjectile,
    Food(FoodProperties),
    Consumable(Consumable<'a>),
    UseRemainder(UseRemainder<'a>),
    UseCooldown(UseCooldown<'a>),
    DamageResistant(DamageResistant<'a>),
    Tool(Tool<'a>),
    Weapon(Weapon),
    AttackRange(AttackRange),
    Enchantable(Enchantable),
    Equippable(Equippable<'a>),
    Repairable(Repairable<'a>),
    Glider,
    TooltipStyle(Ident<'a>),
    DeathProtection(DeathProtection<'a>),
    BlocksAttacks(BlocksAttacks<'a>),
    PiercingWeapon(PiercingWeapon<'a>),
    KineticWeapon(KineticWeapon<'a>),
    SwingAnimation(SwingAnimation),
    StoredEnchantments(ItemEnchantments<'a>),
    DyedColor(DyedItemColor),
    MapColor(MapItemColor),
    MapId(MapId),
    MapDecorations(MapDecorations),
    MapPostProcessing(MapPostProcessing),
    ChargedProjectiles(ChargedProjectiles<'a>),
    BundleContents(BundleContents<'a>),
    PotionContents(PotionContents<'a>),
    PotionDurationScale(f32),
    SuspiciousStewEffects(SuspiciousStewEffects<'a>),
    WritableBookContent(WritableBookContent<'a>),
    WrittenBookContent(WrittenBookContent<'a>),
    Trim(ArmorTrim<'a>),
    DebugStickState(DebugStickState),
    EntityData(TypedEntityDataEntity),
    BucketEntityData(CustomData),
    BlockEntityData(TypedEntityDataBlockEntity),
    Instrument(Either<Holder<Instrument<'a>, InstrumentRef>, ResourceKey<'a>>),
    ProvidesTrimMaterial(ProvidesTrimMaterial<'a>),
    OminousBottleAmplifier(OminousBottleAmplifier),
    JukeboxPlayable(JukeboxPlayable<'a>),
    ProvidesBannerPatterns(TagKey<'a>),
    Recipes(Recipes),
    LodestoneTracker(LodestoneTracker),
    FireworkExplosion(FireworkExplosion<'a>),
    Fireworks(Fireworks<'a>),
    Profile(ResolvableProfile<'a>),
    NoteBlockSound(Ident<'a>),
    BannerPatterns(BannerPatternLayers<'a>),
    BaseColor(DyeColor),
    PotDecorations(List<'a, item, 4>),
    Container(ItemContainerContents<'a>),
    BlockState(BlockItemStateProperties<'a>),
    Bees(Bees<'a>),
    Lock(LockCode),
    ContainerLoot(SeededContainerLoot),
    BreakSound(Holder<SoundEvent<'a>, SoundEventRef>),
    VillagerVariant(VillagerTypeRef),
    WolfVariant(WolfVariantRef),
    WolfSoundVariant(WolfSoundVariantRef),
    WolfCollar(DyeColor),
    FoxVariant(FoxVariant),
    SalmonSize(SalmonVariant),
    ParrotVariant(ParrotVariant),
    TropicalFishPattern(TropicalFishPattern),
    TropicalFishBaseColor(DyeColor),
    TropicalFishPatternColor(DyeColor),
    MooshroomVariant(MushroomCowVariant),
    RabbitVariant(RabbitVariant),
    PigVariant(PigVariantRef),
    CowVariant(CowVariantRef),
    ChickenVariant(Either<ChickenVariantRef, ResourceKey<'a>>),
    ZombieNautilusVariant(Either<ZombieNautilusVariantRef, ResourceKey<'a>>),
    FrogVariant(FrogVariantRef),
    HorseVariant(EquineVariant),
    PaintingVariant(Holder<PaintingVariant<'a>, PaintingVariantRef>),
    LlamaVariant(LlamaVariant),
    AxolotlVariant(AxolotlVariant),
    CatVariant(CatVariantRef),
    CatCollar(DyeColor),
    SheepColor(DyeColor),
    ShulkerColor(DyeColor),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DataComponentExactPredicate<'a> {
    pub expected_components: List<'a, TypedDataComponent<'a>>,
}
