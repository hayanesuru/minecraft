pub mod item_attribute_modifiers;
pub mod item_enchantments;
pub mod kinetic_weapon;
pub mod suspicious_stew_effects;
pub mod tool;

use crate::advancement::BlockPredicate;
use crate::effect::MobEffect;
use crate::food::FoodProperties;
use crate::item::item_attribute_modifiers::ItemAttributeModifiers;
use crate::item::item_enchantments::ItemEnchantments;
use crate::item::kinetic_weapon::KineticWeapon;
use crate::item::suspicious_stew_effects::SuspiciousStewEffects;
use crate::item::tool::Tool;
use crate::sound::SoundEvent;
use crate::{Component, DamageType, EquipmentSlot, Filterable, Holder, HolderSet, Rarity};
use alloc::vec::Vec;
use haya_collection::List;
use haya_ident::{Ident, ResourceKey, TagKey};
use haya_nbt::Tag;
use minecraft_data::{
    consume_effect_type, data_component_type, entity_type, item, mob_effect, potion, sound_event,
};
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
    pub sound: Holder<SoundEvent<'a>, sound_event>,
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
    pub equip_sound: Holder<SoundEvent<'a>, sound_event>,
    pub asset_id: Option<Ident<'a>>,
    pub camera_overlay: Option<Ident<'a>>,
    pub allowed_entities: Option<HolderSet<'a, entity_type>>,
    pub dispensable: bool,
    pub swappable: bool,
    pub damage_on_hurt: bool,
    pub equip_on_interact: bool,
    pub can_be_sheared: bool,
    pub shearing_sound: Holder<SoundEvent<'a>, sound_event>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Repairable<'a> {
    pub items: HolderSet<'a, item>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DeathProtection<'a> {
    pub death_effects: List<'a, ConsumeEffect<'a>>,
}

#[derive(Clone)]
pub enum ConsumeEffect<'a> {
    Apply {
        effects: List<'a, MobEffect<'a>>,
        probability: f32,
    },
    Remove {
        effects: HolderSet<'a, mob_effect>,
    },
    Clear,
    TeleportRandomly {
        diameter: f32,
    },
    PlaySound {
        sound: Holder<SoundEvent<'a>, sound_event>,
    },
}

impl<'a> ConsumeEffect<'a> {
    pub const fn id(&self) -> consume_effect_type {
        match self {
            ConsumeEffect::Apply { .. } => consume_effect_type::apply_effects,
            ConsumeEffect::Remove { .. } => consume_effect_type::remove_effects,
            ConsumeEffect::Clear => consume_effect_type::clear_all_effects,
            ConsumeEffect::TeleportRandomly { .. } => consume_effect_type::teleport_randomly,
            ConsumeEffect::PlaySound { .. } => consume_effect_type::play_sound,
        }
    }
}

impl<'a> Read<'a> for ConsumeEffect<'a> {
    fn read(buf: &mut Reader<'a>) -> Result<Self, Error> {
        Ok(match consume_effect_type::read(buf)? {
            consume_effect_type::apply_effects => Self::Apply {
                effects: Read::read(buf)?,
                probability: Read::read(buf)?,
            },
            consume_effect_type::remove_effects => Self::Remove {
                effects: Read::read(buf)?,
            },
            consume_effect_type::clear_all_effects => Self::Clear,
            consume_effect_type::teleport_randomly => Self::TeleportRandomly {
                diameter: Read::read(buf)?,
            },
            consume_effect_type::play_sound => Self::PlaySound {
                sound: Read::read(buf)?,
            },
        })
    }
}

impl<'a> Write for ConsumeEffect<'a> {
    unsafe fn write(&self, w: &mut Writer) {
        unsafe {
            self.id().write(w);
            match self {
                Self::Apply {
                    effects,
                    probability,
                } => {
                    effects.write(w);
                    probability.write(w);
                }
                Self::Remove { effects } => {
                    effects.write(w);
                }
                Self::Clear => (),
                Self::TeleportRandomly { diameter } => diameter.write(w),
                Self::PlaySound { sound } => sound.write(w),
            }
        }
    }

    fn len_s(&self) -> usize {
        self.id().len_s()
            + match self {
                Self::Apply {
                    effects,
                    probability,
                } => effects.len_s() + probability.len_s(),
                Self::Remove { effects } => effects.len_s(),
                Self::Clear => 0,
                Self::TeleportRandomly { diameter } => diameter.len_s(),
                Self::PlaySound { sound } => sound.len_s(),
            }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BlocksAttacks<'a> {
    pub block_delay_seconds: f32,
    pub disable_cooldown_scale: f32,
    pub damage_reductions: List<'a, DamageReduction<'a>>,
    pub item_damage: ItemDamageFunction,
    pub bypassed_by: Option<TagKey<'a>>,
    pub block_sound: Option<Holder<SoundEvent<'a>, sound_event>>,
    pub disable_sound: Option<Holder<SoundEvent<'a>, sound_event>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DamageReduction<'a> {
    pub horizontal_blocking_angle: f32,
    pub ty: Option<HolderSet<'a, DamageType>>,
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
    pub sound: Option<Holder<SoundEvent<'a>, sound_event>>,
    pub hit_sound: Option<Holder<SoundEvent<'a>, sound_event>>,
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
pub struct MapDecorations {}

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
    DamageType(Either<DamageType, ResourceKey<'a>>),
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
            _ => todo!(),
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
                _ => todo!(),
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
                _ => todo!(),
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
            Self::WrittenBookContent => written_book_content,
            Self::Trim => trim,
            Self::DebugStickState => debug_stick_state,
            Self::EntityData => entity_data,
            Self::BucketEntityData => bucket_entity_data,
            Self::BlockEntityData => block_entity_data,
            Self::Instrument => instrument,
            Self::ProvidesTrimMaterial => provides_trim_material,
            Self::OminousBottleAmplifier => ominous_bottle_amplifier,
            Self::JukeboxPlayable => jukebox_playable,
            Self::ProvidesBannerPatterns => provides_banner_patterns,
            Self::Recipes => recipes,
            Self::LodestoneTracker => lodestone_tracker,
            Self::FireworkExplosion => firework_explosion,
            Self::Fireworks => fireworks,
            Self::Profile => profile,
            Self::NoteBlockSound => note_block_sound,
            Self::BannerPatterns => banner_patterns,
            Self::BaseColor => base_color,
            Self::PotDecorations => pot_decorations,
            Self::Container => container,
            Self::BlockState => block_state,
            Self::Bees => bees,
            Self::Lock => lock,
            Self::ContainerLoot => container_loot,
            Self::BreakSound => break_sound,
            Self::VillagerVariant => villager_variant,
            Self::WolfVariant => wolf_variant,
            Self::WolfSoundVariant => wolf_sound_variant,
            Self::WolfCollar => wolf_collar,
            Self::FoxVariant => fox_variant,
            Self::SalmonSize => salmon_size,
            Self::ParrotVariant => parrot_variant,
            Self::TropicalFishPattern => tropical_fish_pattern,
            Self::TropicalFishBaseColor => tropical_fish_base_color,
            Self::TropicalFishPatternColor => tropical_fish_pattern_color,
            Self::MooshroomVariant => mooshroom_variant,
            Self::RabbitVariant => rabbit_variant,
            Self::PigVariant => pig_variant,
            Self::CowVariant => cow_variant,
            Self::ChickenVariant => chicken_variant,
            Self::ZombieNautilusVariant => zombie_nautilus_variant,
            Self::FrogVariant => frog_variant,
            Self::HorseVariant => horse_variant,
            Self::PaintingVariant => painting_variant,
            Self::LlamaVariant => llama_variant,
            Self::AxolotlVariant => axolotl_variant,
            Self::CatVariant => cat_variant,
            Self::CatCollar => cat_collar,
            Self::SheepColor => sheep_color,
            Self::ShulkerColor => shulker_color,
        }
    }
}
