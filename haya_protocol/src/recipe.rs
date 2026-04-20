use crate::Holder;
use crate::item::ItemStack;
use crate::registry::TrimPatternRef;
use crate::trim::TrimPattern;
use haya_collection::{Cow, List};
use haya_ident::TagKey;
use minecraft_data::{item, recipe_display, slot_display};

#[derive(Clone, Serialize, Deserialize)]
#[mser(header = recipe_display)]
pub enum RecipeDisplay<'a> {
    CraftingShapeless {
        ingredients: List<'a, SlotDisplay<'a>>,
        result: SlotDisplay<'a>,
        crafting_station: SlotDisplay<'a>,
    },
    CraftingShaped {
        #[mser(varint)]
        width: u32,
        #[mser(varint)]
        height: u32,
        ingredients: List<'a, SlotDisplay<'a>>,
        result: SlotDisplay<'a>,
        crafting_station: SlotDisplay<'a>,
    },
    Furnace {
        ingredient: SlotDisplay<'a>,
        fuel: SlotDisplay<'a>,
        result: SlotDisplay<'a>,
        crafting_station: SlotDisplay<'a>,
        #[mser(varint)]
        duration: u32,
        experience: f32,
    },
    Stonecutter {
        input: SlotDisplay<'a>,
        result: SlotDisplay<'a>,
        crafting_station: SlotDisplay<'a>,
    },
    Smithing {
        template: SlotDisplay<'a>,
        base: SlotDisplay<'a>,
        addition: SlotDisplay<'a>,
        result: SlotDisplay<'a>,
        crafting_station: SlotDisplay<'a>,
    },
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
