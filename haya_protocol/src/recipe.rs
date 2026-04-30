use crate::item::ItemStack;
use crate::registry::TrimPatternRef;
use crate::trim::TrimPattern;
use crate::{Holder, HolderSet, OptionalV32};
use haya_collection::{Cow, List};
use haya_ident::TagKey;
use minecraft_data::{item, recipe_book_category, recipe_display, slot_display};

#[derive(Clone, Serialize, Deserialize)]
#[mser(header = recipe_display)]
pub enum RecipeDisplay<'a> {
    CraftingShapeless(ShapelessCraftingRecipeDisplay<'a>),
    CraftingShaped(ShapedCraftingRecipeDisplay<'a>),
    Furnace(FurnaceRecipeDisplay<'a>),
    Stonecutter(StonecutterRecipeDisplay<'a>),
    Smithing(SmithingRecipeDisplay<'a>),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ShapelessCraftingRecipeDisplay<'a> {
    pub ingredients: List<'a, SlotDisplay<'a>>,
    pub result: SlotDisplay<'a>,
    pub crafting_station: SlotDisplay<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ShapedCraftingRecipeDisplay<'a> {
    #[mser(varint)]
    pub width: u32,
    #[mser(varint)]
    pub height: u32,
    pub ingredients: List<'a, SlotDisplay<'a>>,
    pub result: SlotDisplay<'a>,
    pub crafting_station: SlotDisplay<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct FurnaceRecipeDisplay<'a> {
    pub ingredient: SlotDisplay<'a>,
    pub fuel: SlotDisplay<'a>,
    pub result: SlotDisplay<'a>,
    pub crafting_station: SlotDisplay<'a>,
    #[mser(varint)]
    pub duration: u32,
    pub experience: f32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct StonecutterRecipeDisplay<'a> {
    pub input: SlotDisplay<'a>,
    pub result: SlotDisplay<'a>,
    pub crafting_station: SlotDisplay<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SmithingRecipeDisplay<'a> {
    pub template: SlotDisplay<'a>,
    pub base: SlotDisplay<'a>,
    pub addition: SlotDisplay<'a>,
    pub result: SlotDisplay<'a>,
    pub crafting_station: SlotDisplay<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
#[mser(header = slot_display)]
pub enum SlotDisplay<'a> {
    Empty {},
    AnyFuel {},
    Item {
        item: item,
    },
    ItemStack {
        stack: ItemStack<'a>,
    },
    Tag {
        tag: TagKey<'a>,
    },
    SmithingTrim {
        smithing_trim: Cow<'a, SmithingTrimDemoSlotDisplay<'a>>,
    },
    WithRemainder {
        with_remainder: Cow<'a, WithRemainder<'a>>,
    },
    Composite {
        contents: List<'a, SlotDisplay<'a>>,
    },
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SmithingTrimDemoSlotDisplay<'a> {
    pub base: SlotDisplay<'a>,
    pub material: SlotDisplay<'a>,
    pub pattern: Holder<TrimPattern<'a>, TrimPatternRef>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct WithRemainder<'a> {
    pub input: SlotDisplay<'a>,
    pub remainder: SlotDisplay<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RecipeDisplayEntry<'a> {
    pub id: RecipeDisplayId,
    pub display: RecipeDisplay<'a>,
    pub group: OptionalV32,
    pub category: recipe_book_category,
    pub crafting_requirements: Option<List<'a, Ingredient<'a>>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RecipeDisplayId {
    #[mser(varint)]
    pub index: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Ingredient<'a> {
    pub values: HolderSet<'a, item>,
}
