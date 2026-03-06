use crate::advancement::BlockPredicate;
use crate::attribute::AttributeModifier;
use crate::food::FoodProperties;
use crate::{Component, EquipmentSlotGroup, Holder, Rarity};
use haya_collection::{List, Map};
use haya_ident::{Ident, ResourceKey};
use haya_nbt::Tag;
use minecraft_data::{attribute, data_component_type, item};
use mser::{Either, Error, Read, Reader, Utf8, V21, V32, Write, Writer};

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
pub struct AdventureModePredicate<'a> {
    pub predicates: List<'a, BlockPredicate<'a>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ItemAttributeModifiers<'a> {
    pub modifiers: List<'a, ItemAttributeModifiersEntry<'a>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ItemAttributeModifiersEntry<'a> {
    pub attribute: attribute,
    pub modifier: AttributeModifier<'a>,
    pub slot: EquipmentSlotGroup,
    pub display: ItemAttributeModifiersDisplay,
}

#[derive(Clone)]
pub enum ItemAttributeModifiersDisplay {
    Default,
    Hidden,
    Override(Component),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CustomModelData<'a> {
    pub floats: List<'a, f32>,
    pub flags: List<'a, bool>,
    pub strings: List<'a, Utf8<'a>>,
    pub colors: List<'a, u32>,
}

impl<'a> Read<'a> for ItemAttributeModifiersDisplay {
    fn read(buf: &mut Reader<'a>) -> Result<Self, Error> {
        Ok(match V21::read(buf)?.0 {
            1 => Self::Hidden,
            2 => Self::Override(Component::read(buf)?),
            _ => Self::Default,
        })
    }
}

impl Write for ItemAttributeModifiersDisplay {
    unsafe fn write(&self, w: &mut Writer) {
        unsafe {
            match self {
                Self::Default => w.write_byte(0),
                Self::Hidden => w.write_byte(1),
                Self::Override(component) => {
                    w.write_byte(2);
                    component.write(w);
                }
            }
        }
    }

    fn len_s(&self) -> usize {
        match self {
            Self::Default => 1,
            Self::Hidden => 1,
            Self::Override(component) => 1 + component.len_s(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TooltipDisplay<'a> {
    pub hide_tooltip: bool,
    pub hidden_components: List<'a, TypedDataComponentType<'a>>,
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
    DamageType(Either<Holder, ResourceKey<'a>>),
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
            Self::Consumable => consumable,
            Self::UseRemainder => use_remainder,
            Self::UseCooldown => use_cooldown,
            Self::DamageResistant => damage_resistant,
            Self::Tool => tool,
            Self::Weapon => weapon,
            Self::AttackRange => attack_range,
            Self::Enchantable => enchantable,
            Self::Equippable => equippable,
            Self::Repairable => repairable,
            Self::Glider => glider,
            Self::TooltipStyle => tooltip_style,
            Self::DeathProtection => death_protection,
            Self::BlocksAttacks => blocks_attacks,
            Self::PiercingWeapon => piercing_weapon,
            Self::KineticWeapon => kinetic_weapon,
            Self::SwingAnimation => swing_animation,
            Self::StoredEnchantments => stored_enchantments,
            Self::DyedColor => dyed_color,
            Self::MapColor => map_color,
            Self::MapId => map_id,
            Self::MapDecorations => map_decorations,
            Self::MapPostProcessing => map_post_processing,
            Self::ChargedProjectiles => charged_projectiles,
            Self::BundleContents => bundle_contents,
            Self::PotionContents => potion_contents,
            Self::PotionDurationScale => potion_duration_scale,
            Self::SuspiciousStewEffects => suspicious_stew_effects,
            Self::WritableBookContent => writable_book_content,
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
