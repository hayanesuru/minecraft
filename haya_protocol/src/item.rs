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
    EquineVariant, FoxVariant, MushroomCowVariant, ParrotVariant, RabbitVariant, SalmonVariant,
    TropicalFishPattern,
};
use crate::food::FoodProperties;
use crate::item::consume_effect::ConsumeEffect;
use crate::item::firework_explosion::FireworkExplosion;
use crate::item::item_attribute_modifiers::ItemAttributeModifiers;
use crate::item::item_enchantments::ItemEnchantments;
use crate::item::kinetic_weapon::KineticWeapon;
use crate::item::suspicious_stew_effects::SuspiciousStewEffects;
use crate::item::tool::Tool;
use crate::profile::ResolvableProfile;
use crate::registry::{
    ChickenVariantRef, CowVariantRef, DamageTypeRef, FrogVariantRef, InstrumentRef, JukeboxSongRef,
    PigVariantRef, SoundEventRef, TrimMaterialRef, TrimPatternRef, VillagerTypeRef,
    WolfSoundVariantRef, WolfVariantRef, ZombieNautilusVariantRef,
};
use crate::sound::SoundEvent;
use crate::trim::{TrimMaterial, TrimPattern};
use crate::{Component, DyeColor, EquipmentSlot, Filterable, Holder, HolderSet, LockCode, Rarity};
use alloc::vec::Vec;
use haya_collection::{List, Map};
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
    pub patch_add: List<'a, TypedDataComponentType<'a>>,
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
                let mut patch_add = Vec::with_capacity(usize::min(positive, 65536));
                for _ in 0..positive {
                    patch_add.push(TypedDataComponentType::read(buf)?);
                }
                let mut patch_remove = Vec::with_capacity(usize::min(negative, 65536));
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
    pub hidden_components: List<'a, TypedDataComponentType<'a>>,
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

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct MapId(#[mser(varint)] pub u32);

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
pub struct SeededContainerLoot(Tag);

#[derive(Clone)]
pub enum TypedDataComponentType<'a> {
    CustomData(CustomData),
    MaxStackSize(u32),
    MaxDamage(u32),
    Damage(u32),
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
    RepairCost(u32),
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
    PaintingVariant,
    LlamaVariant,
    AxolotlVariant,
    CatVariant,
    CatCollar(DyeColor),
    SheepColor(DyeColor),
    ShulkerColor(DyeColor),
}

impl<'a> Read<'a> for TypedDataComponentType<'a> {
    fn read(buf: &mut Reader<'a>) -> Result<Self, Error> {
        use data_component_type::*;
        Ok(match data_component_type::read(buf)? {
            custom_data => Self::CustomData(CustomData::read(buf)?),
            max_stack_size => Self::MaxStackSize(V32::read(buf)?.0),
            max_damage => Self::MaxDamage(V32::read(buf)?.0),
            damage => Self::Damage(V32::read(buf)?.0),
            unbreakable => Self::Unbreakable,
            use_effects => Self::UseEffects(UseEffects::read(buf)?),
            custom_name => Self::CustomName(Component::read(buf)?),
            minimum_attack_charge => Self::MinimumAttackCharge(f32::read(buf)?),
            damage_type => Self::DamageType(Either::read(buf)?),
            item_name => Self::ItemName(Component::read(buf)?),
            item_model => Self::ItemModel(Ident::read(buf)?),
            lore => Self::Lore(ItemLore::read(buf)?),
            rarity => Self::Rarity(Rarity::read(buf)?),
            enchantments => Self::Enchantments(ItemEnchantments::read(buf)?),
            can_place_on => Self::CanPlaceOn(AdventureModePredicate::read(buf)?),
            can_break => Self::CanBreak(AdventureModePredicate::read(buf)?),
            attribute_modifiers => Self::AttributeModifiers(ItemAttributeModifiers::read(buf)?),
            custom_model_data => Self::CustomModelData(CustomModelData::read(buf)?),
            tooltip_display => Self::TooltipDisplay(TooltipDisplay::read(buf)?),
            repair_cost => Self::RepairCost(V32::read(buf)?.0),
            creative_slot_lock => Self::CreativeSlotLock,
            enchantment_glint_override => Self::EnchantmentGlintOverride(bool::read(buf)?),
            intangible_projectile => Self::IntangibleProjectile,
            food => Self::Food(FoodProperties::read(buf)?),
            consumable => Self::Consumable(Consumable::read(buf)?),
            use_remainder => Self::UseRemainder(UseRemainder::read(buf)?),
            use_cooldown => Self::UseCooldown(UseCooldown::read(buf)?),
            damage_resistant => Self::DamageResistant(DamageResistant::read(buf)?),
            tool => Self::Tool(Tool::read(buf)?),
            weapon => Self::Weapon(Weapon::read(buf)?),
            attack_range => Self::AttackRange(AttackRange::read(buf)?),
            enchantable => Self::Enchantable(Enchantable::read(buf)?),
            equippable => Self::Equippable(Equippable::read(buf)?),
            repairable => Self::Repairable(Repairable::read(buf)?),
            glider => Self::Glider,
            tooltip_style => Self::TooltipStyle(Ident::read(buf)?),
            death_protection => Self::DeathProtection(DeathProtection::read(buf)?),
            blocks_attacks => Self::BlocksAttacks(BlocksAttacks::read(buf)?),
            piercing_weapon => Self::PiercingWeapon(PiercingWeapon::read(buf)?),
            kinetic_weapon => Self::KineticWeapon(KineticWeapon::read(buf)?),
            swing_animation => Self::SwingAnimation(SwingAnimation::read(buf)?),
            stored_enchantments => Self::StoredEnchantments(ItemEnchantments::read(buf)?),
            dyed_color => Self::DyedColor(DyedItemColor::read(buf)?),
            map_color => Self::MapColor(MapItemColor::read(buf)?),
            map_id => Self::MapId(MapId::read(buf)?),
            map_decorations => Self::MapDecorations(MapDecorations::read(buf)?),
            map_post_processing => Self::MapPostProcessing(MapPostProcessing::read(buf)?),
            charged_projectiles => Self::ChargedProjectiles(ChargedProjectiles::read(buf)?),
            bundle_contents => Self::BundleContents(BundleContents::read(buf)?),
            potion_contents => Self::PotionContents(PotionContents::read(buf)?),
            potion_duration_scale => Self::PotionDurationScale(f32::read(buf)?),
            suspicious_stew_effects => {
                Self::SuspiciousStewEffects(SuspiciousStewEffects::read(buf)?)
            }
            writable_book_content => Self::WritableBookContent(WritableBookContent::read(buf)?),
            written_book_content => Self::WrittenBookContent(WrittenBookContent::read(buf)?),
            trim => Self::Trim(ArmorTrim::read(buf)?),
            debug_stick_state => Self::DebugStickState(DebugStickState::read(buf)?),
            entity_data => Self::EntityData(TypedEntityDataEntity::read(buf)?),
            bucket_entity_data => Self::BucketEntityData(CustomData::read(buf)?),
            block_entity_data => Self::BlockEntityData(TypedEntityDataBlockEntity::read(buf)?),
            instrument => Self::Instrument(Either::read(buf)?),
            provides_trim_material => Self::ProvidesTrimMaterial(ProvidesTrimMaterial::read(buf)?),
            ominous_bottle_amplifier => {
                Self::OminousBottleAmplifier(OminousBottleAmplifier::read(buf)?)
            }
            jukebox_playable => Self::JukeboxPlayable(JukeboxPlayable::read(buf)?),
            provides_banner_patterns => Self::ProvidesBannerPatterns(TagKey::read(buf)?),
            recipes => Self::Recipes(Recipes::read(buf)?),
            lodestone_tracker => Self::LodestoneTracker(LodestoneTracker::read(buf)?),
            firework_explosion => Self::FireworkExplosion(FireworkExplosion::read(buf)?),
            fireworks => Self::Fireworks(Fireworks::read(buf)?),
            profile => Self::Profile(ResolvableProfile::read(buf)?),
            note_block_sound => Self::NoteBlockSound(Ident::read(buf)?),
            banner_patterns => Self::BannerPatterns(BannerPatternLayers::read(buf)?),
            base_color => Self::BaseColor(DyeColor::read(buf)?),
            pot_decorations => Self::PotDecorations(List::read(buf)?),
            container => Self::Container(ItemContainerContents::read(buf)?),
            block_state => Self::BlockState(BlockItemStateProperties::read(buf)?),
            bees => Self::Bees(Bees::read(buf)?),
            lock => Self::Lock(LockCode::read(buf)?),
            container_loot => Self::ContainerLoot(SeededContainerLoot::read(buf)?),
            break_sound => Self::BreakSound(Holder::read(buf)?),
            villager_variant => Self::VillagerVariant(VillagerTypeRef::read(buf)?),
            wolf_variant => Self::WolfVariant(WolfVariantRef::read(buf)?),
            wolf_sound_variant => Self::WolfSoundVariant(WolfSoundVariantRef::read(buf)?),
            wolf_collar => Self::WolfCollar(DyeColor::read(buf)?),
            fox_variant => Self::FoxVariant(FoxVariant::read(buf)?),
            salmon_size => Self::SalmonSize(SalmonVariant::read(buf)?),
            parrot_variant => Self::ParrotVariant(ParrotVariant::read(buf)?),
            tropical_fish_pattern => Self::TropicalFishPattern(TropicalFishPattern::read(buf)?),
            tropical_fish_base_color => Self::TropicalFishBaseColor(DyeColor::read(buf)?),
            tropical_fish_pattern_color => Self::TropicalFishPatternColor(DyeColor::read(buf)?),
            mooshroom_variant => Self::MooshroomVariant(MushroomCowVariant::read(buf)?),
            rabbit_variant => Self::RabbitVariant(RabbitVariant::read(buf)?),
            pig_variant => Self::PigVariant(PigVariantRef::read(buf)?),
            cow_variant => Self::CowVariant(CowVariantRef::read(buf)?),
            chicken_variant => Self::ChickenVariant(Either::read(buf)?),
            zombie_nautilus_variant => Self::ZombieNautilusVariant(Either::read(buf)?),
            frog_variant => Self::FrogVariant(FrogVariantRef::read(buf)?),
            horse_variant => Self::HorseVariant(EquineVariant::read(buf)?),
            painting_variant => todo!(),
            llama_variant => todo!(),
            axolotl_variant => todo!(),
            cat_variant => todo!(),
            cat_collar => Self::CatCollar(DyeColor::read(buf)?),
            sheep_color => Self::SheepColor(DyeColor::read(buf)?),
            shulker_color => Self::ShulkerColor(DyeColor::read(buf)?),
        })
    }
}

impl<'a> Write for TypedDataComponentType<'a> {
    unsafe fn write(&self, w: &mut Writer) {
        unsafe {
            self.id().write(w);
            match self {
                Self::CustomData(x) => x.write(w),
                Self::MaxStackSize(x) => V32(*x).write(w),
                Self::MaxDamage(x) => V32(*x).write(w),
                Self::Damage(x) => V32(*x).write(w),
                Self::Unbreakable => (),
                Self::UseEffects(x) => x.write(w),
                Self::CustomName(x) => x.write(w),
                Self::MinimumAttackCharge(x) => x.write(w),
                Self::DamageType(x) => x.write(w),
                Self::ItemName(x) => x.write(w),
                Self::ItemModel(x) => x.write(w),
                Self::Lore(x) => x.write(w),
                Self::Rarity(x) => x.write(w),
                Self::Enchantments(x) => x.write(w),
                Self::CanPlaceOn(x) => x.write(w),
                Self::CanBreak(x) => x.write(w),
                Self::AttributeModifiers(x) => x.write(w),
                Self::CustomModelData(x) => x.write(w),
                Self::TooltipDisplay(x) => x.write(w),
                Self::RepairCost(x) => V32(*x).write(w),
                Self::CreativeSlotLock => (),
                Self::EnchantmentGlintOverride(x) => x.write(w),
                Self::IntangibleProjectile => (),
                Self::Food(x) => x.write(w),
                Self::Consumable(x) => x.write(w),
                Self::UseRemainder(x) => x.write(w),
                Self::UseCooldown(x) => x.write(w),
                Self::DamageResistant(x) => x.write(w),
                Self::Tool(x) => x.write(w),
                Self::Weapon(x) => x.write(w),
                Self::AttackRange(x) => x.write(w),
                Self::Enchantable(x) => x.write(w),
                Self::Equippable(x) => x.write(w),
                Self::Repairable(x) => x.write(w),
                Self::Glider => (),
                Self::TooltipStyle(x) => x.write(w),
                Self::DeathProtection(x) => x.write(w),
                Self::BlocksAttacks(x) => x.write(w),
                Self::PiercingWeapon(x) => x.write(w),
                Self::KineticWeapon(x) => x.write(w),
                Self::SwingAnimation(x) => x.write(w),
                Self::StoredEnchantments(x) => x.write(w),
                Self::DyedColor(x) => x.write(w),
                Self::MapColor(x) => x.write(w),
                Self::MapId(x) => x.write(w),
                Self::MapDecorations(x) => x.write(w),
                Self::MapPostProcessing(x) => x.write(w),
                Self::ChargedProjectiles(x) => x.write(w),
                Self::BundleContents(x) => x.write(w),
                Self::PotionContents(x) => x.write(w),
                Self::PotionDurationScale(x) => x.write(w),
                Self::SuspiciousStewEffects(x) => x.write(w),
                Self::WritableBookContent(x) => x.write(w),
                Self::WrittenBookContent(x) => x.write(w),
                Self::Trim(x) => x.write(w),
                Self::DebugStickState(x) => x.write(w),
                Self::EntityData(x) => x.write(w),
                Self::BucketEntityData(x) => x.write(w),
                Self::BlockEntityData(x) => x.write(w),
                Self::Instrument(x) => x.write(w),
                Self::ProvidesTrimMaterial(x) => x.write(w),
                Self::OminousBottleAmplifier(x) => x.write(w),
                Self::JukeboxPlayable(x) => x.write(w),
                Self::ProvidesBannerPatterns(x) => x.write(w),
                Self::Recipes(x) => x.write(w),
                Self::LodestoneTracker(x) => x.write(w),
                Self::FireworkExplosion(x) => x.write(w),
                Self::Fireworks(x) => x.write(w),
                Self::Profile(x) => x.write(w),
                Self::NoteBlockSound(x) => x.write(w),
                Self::BannerPatterns(x) => x.write(w),
                Self::BaseColor(x) => x.write(w),
                Self::PotDecorations(x) => x.write(w),
                Self::Container(x) => x.write(w),
                Self::BlockState(x) => x.write(w),
                Self::Bees(x) => x.write(w),
                Self::Lock(x) => x.write(w),
                Self::ContainerLoot(x) => x.write(w),
                Self::BreakSound(x) => x.write(w),
                Self::VillagerVariant(x) => x.write(w),
                Self::WolfVariant(x) => x.write(w),
                Self::WolfSoundVariant(x) => x.write(w),
                Self::WolfCollar(x) => x.write(w),
                Self::FoxVariant(x) => x.write(w),
                Self::SalmonSize(x) => x.write(w),
                Self::ParrotVariant(x) => x.write(w),
                Self::TropicalFishPattern(x) => x.write(w),
                Self::TropicalFishBaseColor(x) => x.write(w),
                Self::TropicalFishPatternColor(x) => x.write(w),
                Self::MooshroomVariant(x) => x.write(w),
                Self::RabbitVariant(x) => x.write(w),
                Self::PigVariant(x) => x.write(w),
                Self::CowVariant(x) => x.write(w),
                Self::ChickenVariant(x) => x.write(w),
                Self::ZombieNautilusVariant(x) => x.write(w),
                Self::FrogVariant(x) => x.write(w),
                Self::HorseVariant(x) => x.write(w),
                Self::PaintingVariant => todo!(),
                Self::LlamaVariant => todo!(),
                Self::AxolotlVariant => todo!(),
                Self::CatVariant => todo!(),
                Self::CatCollar(x) => x.write(w),
                Self::SheepColor(x) => x.write(w),
                Self::ShulkerColor(x) => x.write(w),
            }
        }
    }

    fn len_s(&self) -> usize {
        self.id().len_s()
            + match self {
                Self::CustomData(x) => x.len_s(),
                Self::MaxStackSize(x) => V32(*x).len_s(),
                Self::MaxDamage(x) => V32(*x).len_s(),
                Self::Damage(x) => V32(*x).len_s(),
                Self::Unbreakable => 0,
                Self::UseEffects(x) => x.len_s(),
                Self::CustomName(x) => x.len_s(),
                Self::MinimumAttackCharge(x) => x.len_s(),
                Self::DamageType(x) => x.len_s(),
                Self::ItemName(x) => x.len_s(),
                Self::ItemModel(x) => x.len_s(),
                Self::Lore(x) => x.len_s(),
                Self::Rarity(x) => x.len_s(),
                Self::Enchantments(x) => x.len_s(),
                Self::CanPlaceOn(x) => x.len_s(),
                Self::CanBreak(x) => x.len_s(),
                Self::AttributeModifiers(x) => x.len_s(),
                Self::CustomModelData(x) => x.len_s(),
                Self::TooltipDisplay(x) => x.len_s(),
                Self::RepairCost(x) => V32(*x).len_s(),
                Self::CreativeSlotLock => 0,
                Self::EnchantmentGlintOverride(x) => x.len_s(),
                Self::IntangibleProjectile => 0,
                Self::Food(x) => x.len_s(),
                Self::Consumable(x) => x.len_s(),
                Self::UseRemainder(x) => x.len_s(),
                Self::UseCooldown(x) => x.len_s(),
                Self::DamageResistant(x) => x.len_s(),
                Self::Tool(x) => x.len_s(),
                Self::Weapon(x) => x.len_s(),
                Self::AttackRange(x) => x.len_s(),
                Self::Enchantable(x) => x.len_s(),
                Self::Equippable(x) => x.len_s(),
                Self::Repairable(x) => x.len_s(),
                Self::Glider => 0,
                Self::TooltipStyle(x) => x.len_s(),
                Self::DeathProtection(x) => x.len_s(),
                Self::BlocksAttacks(x) => x.len_s(),
                Self::PiercingWeapon(x) => x.len_s(),
                Self::KineticWeapon(x) => x.len_s(),
                Self::SwingAnimation(x) => x.len_s(),
                Self::StoredEnchantments(x) => x.len_s(),
                Self::DyedColor(x) => x.len_s(),
                Self::MapColor(x) => x.len_s(),
                Self::MapId(x) => x.len_s(),
                Self::MapDecorations(x) => x.len_s(),
                Self::MapPostProcessing(x) => x.len_s(),
                Self::ChargedProjectiles(x) => x.len_s(),
                Self::BundleContents(x) => x.len_s(),
                Self::PotionContents(x) => x.len_s(),
                Self::PotionDurationScale(x) => x.len_s(),
                Self::SuspiciousStewEffects(x) => x.len_s(),
                Self::WritableBookContent(x) => x.len_s(),
                Self::WrittenBookContent(x) => x.len_s(),
                Self::Trim(x) => x.len_s(),
                Self::DebugStickState(x) => x.len_s(),
                Self::EntityData(x) => x.len_s(),
                Self::BucketEntityData(x) => x.len_s(),
                Self::BlockEntityData(x) => x.len_s(),
                Self::Instrument(x) => x.len_s(),
                Self::ProvidesTrimMaterial(x) => x.len_s(),
                Self::OminousBottleAmplifier(x) => x.len_s(),
                Self::JukeboxPlayable(x) => x.len_s(),
                Self::ProvidesBannerPatterns(x) => x.len_s(),
                Self::Recipes(x) => x.len_s(),
                Self::LodestoneTracker(x) => x.len_s(),
                Self::FireworkExplosion(x) => x.len_s(),
                Self::Fireworks(x) => x.len_s(),
                Self::Profile(x) => x.len_s(),
                Self::NoteBlockSound(x) => x.len_s(),
                Self::BannerPatterns(x) => x.len_s(),
                Self::BaseColor(x) => x.len_s(),
                Self::PotDecorations(x) => x.len_s(),
                Self::Container(x) => x.len_s(),
                Self::BlockState(x) => x.len_s(),
                Self::Bees(x) => x.len_s(),
                Self::Lock(x) => x.len_s(),
                Self::ContainerLoot(x) => x.len_s(),
                Self::BreakSound(x) => x.len_s(),
                Self::VillagerVariant(x) => x.len_s(),
                Self::WolfVariant(x) => x.len_s(),
                Self::WolfSoundVariant(x) => x.len_s(),
                Self::WolfCollar(x) => x.len_s(),
                Self::FoxVariant(x) => x.len_s(),
                Self::SalmonSize(x) => x.len_s(),
                Self::ParrotVariant(x) => x.len_s(),
                Self::TropicalFishPattern(x) => x.len_s(),
                Self::TropicalFishBaseColor(x) => x.len_s(),
                Self::TropicalFishPatternColor(x) => x.len_s(),
                Self::MooshroomVariant(x) => x.len_s(),
                Self::RabbitVariant(x) => x.len_s(),
                Self::PigVariant(x) => x.len_s(),
                Self::CowVariant(x) => x.len_s(),
                Self::ChickenVariant(x) => x.len_s(),
                Self::ZombieNautilusVariant(x) => x.len_s(),
                Self::FrogVariant(x) => x.len_s(),
                Self::HorseVariant(x) => x.len_s(),
                Self::PaintingVariant => todo!(),
                Self::LlamaVariant => todo!(),
                Self::AxolotlVariant => todo!(),
                Self::CatVariant => todo!(),
                Self::CatCollar(x) => x.len_s(),
                Self::SheepColor(x) => x.len_s(),
                Self::ShulkerColor(x) => x.len_s(),
            }
    }
}

impl TypedDataComponentType<'_> {
    pub const fn id(&self) -> data_component_type {
        use data_component_type::*;

        match self {
            Self::CustomData(..) => custom_data,
            Self::MaxStackSize(..) => max_stack_size,
            Self::MaxDamage(..) => max_damage,
            Self::Damage(..) => damage,
            Self::Unbreakable => unbreakable,
            Self::UseEffects(..) => use_effects,
            Self::CustomName(..) => custom_name,
            Self::MinimumAttackCharge(..) => minimum_attack_charge,
            Self::DamageType(..) => damage_type,
            Self::ItemName(..) => item_name,
            Self::ItemModel(..) => item_model,
            Self::Lore(..) => lore,
            Self::Rarity(..) => rarity,
            Self::Enchantments(..) => enchantments,
            Self::CanPlaceOn(..) => can_place_on,
            Self::CanBreak(..) => can_break,
            Self::AttributeModifiers(..) => attribute_modifiers,
            Self::CustomModelData(..) => custom_model_data,
            Self::TooltipDisplay(..) => tooltip_display,
            Self::RepairCost(..) => repair_cost,
            Self::CreativeSlotLock => creative_slot_lock,
            Self::EnchantmentGlintOverride(..) => enchantment_glint_override,
            Self::IntangibleProjectile => intangible_projectile,
            Self::Food(..) => food,
            Self::Consumable(..) => consumable,
            Self::UseRemainder(..) => use_remainder,
            Self::UseCooldown(..) => use_cooldown,
            Self::DamageResistant(..) => damage_resistant,
            Self::Tool(..) => tool,
            Self::Weapon(..) => weapon,
            Self::AttackRange(..) => attack_range,
            Self::Enchantable(..) => enchantable,
            Self::Equippable(..) => equippable,
            Self::Repairable(..) => repairable,
            Self::Glider => glider,
            Self::TooltipStyle(..) => tooltip_style,
            Self::DeathProtection(..) => death_protection,
            Self::BlocksAttacks(..) => blocks_attacks,
            Self::PiercingWeapon(..) => piercing_weapon,
            Self::KineticWeapon(..) => kinetic_weapon,
            Self::SwingAnimation(..) => swing_animation,
            Self::StoredEnchantments(..) => stored_enchantments,
            Self::DyedColor(..) => dyed_color,
            Self::MapColor(..) => map_color,
            Self::MapId(..) => map_id,
            Self::MapDecorations(..) => map_decorations,
            Self::MapPostProcessing(..) => map_post_processing,
            Self::ChargedProjectiles(..) => charged_projectiles,
            Self::BundleContents(..) => bundle_contents,
            Self::PotionContents(..) => potion_contents,
            Self::PotionDurationScale(..) => potion_duration_scale,
            Self::SuspiciousStewEffects(..) => suspicious_stew_effects,
            Self::WritableBookContent(..) => writable_book_content,
            Self::WrittenBookContent(..) => written_book_content,
            Self::Trim(..) => trim,
            Self::DebugStickState(..) => debug_stick_state,
            Self::EntityData(..) => entity_data,
            Self::BucketEntityData(..) => bucket_entity_data,
            Self::BlockEntityData(..) => block_entity_data,
            Self::Instrument(..) => instrument,
            Self::ProvidesTrimMaterial(..) => provides_trim_material,
            Self::OminousBottleAmplifier(..) => ominous_bottle_amplifier,
            Self::JukeboxPlayable(..) => jukebox_playable,
            Self::ProvidesBannerPatterns(..) => provides_banner_patterns,
            Self::Recipes(..) => recipes,
            Self::LodestoneTracker(..) => lodestone_tracker,
            Self::FireworkExplosion(..) => firework_explosion,
            Self::Fireworks(..) => fireworks,
            Self::Profile(..) => profile,
            Self::NoteBlockSound(..) => note_block_sound,
            Self::BannerPatterns(..) => banner_patterns,
            Self::BaseColor(..) => base_color,
            Self::PotDecorations(..) => pot_decorations,
            Self::Container(..) => container,
            Self::BlockState(..) => block_state,
            Self::Bees(..) => bees,
            Self::Lock(..) => lock,
            Self::ContainerLoot(..) => container_loot,
            Self::BreakSound(..) => break_sound,
            Self::VillagerVariant(..) => villager_variant,
            Self::WolfVariant(..) => wolf_variant,
            Self::WolfSoundVariant(..) => wolf_sound_variant,
            Self::WolfCollar(..) => wolf_collar,
            Self::FoxVariant(..) => fox_variant,
            Self::SalmonSize(..) => salmon_size,
            Self::ParrotVariant(..) => parrot_variant,
            Self::TropicalFishPattern(..) => tropical_fish_pattern,
            Self::TropicalFishBaseColor(..) => tropical_fish_base_color,
            Self::TropicalFishPatternColor(..) => tropical_fish_pattern_color,
            Self::MooshroomVariant(..) => mooshroom_variant,
            Self::RabbitVariant(..) => rabbit_variant,
            Self::PigVariant(..) => pig_variant,
            Self::CowVariant(..) => cow_variant,
            Self::ChickenVariant(..) => chicken_variant,
            Self::ZombieNautilusVariant(..) => zombie_nautilus_variant,
            Self::FrogVariant(..) => frog_variant,
            Self::HorseVariant(..) => horse_variant,
            Self::PaintingVariant => painting_variant,
            Self::LlamaVariant => llama_variant,
            Self::AxolotlVariant => axolotl_variant,
            Self::CatVariant => cat_variant,
            Self::CatCollar(..) => cat_collar,
            Self::SheepColor(..) => sheep_color,
            Self::ShulkerColor(..) => shulker_color,
        }
    }
}
