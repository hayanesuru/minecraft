use crate::inventory::ItemStack;
use crate::{recipe_serializer, Identifier, UnsafeWriter, Writable, Write, V21, V32};

#[derive(Copy, Clone)]
pub struct Recipe<'a> {
    pub ser: recipe_serializer,
    pub id: Identifier<'a>,
    pub data: RecipeData<'a>,
}

impl Write for Recipe<'_> {
    fn write(&self, w: &mut UnsafeWriter) {
        Identifier(self.ser.name()).write(w);
        self.id.write(w);
        match self.data {
            RecipeData::Shapeless(x) => x.write(w),
            RecipeData::Shaped(x) => x.write(w),
        }
    }

    fn len(&self) -> usize {
        Identifier(self.ser.name()).len()
            + self.id.len()
            + match self.data {
                RecipeData::Shapeless(x) => x.len(),
                RecipeData::Shaped(x) => x.len(),
            }
    }
}

#[derive(Copy, Clone)]
pub enum RecipeData<'a> {
    Shapeless(ShapelessRecipe<'a>),
    Shaped(ShapedRecipe<'a>),
}

#[derive(Writable, Copy, Clone)]
pub struct ShapelessRecipe<'a> {
    pub group: &'a str,
    pub category: CraftingCategory,
    #[ser(expand)]
    pub ingredients: &'a [Ingredient<'a>],
    pub result: ItemStack,
}

#[derive(Writable, Copy, Clone)]
pub struct ShapedRecipe<'a> {
    pub group: &'a str,
    pub category: CraftingCategory,
    #[ser(varint)]
    pub width: u32,
    #[ser(varint)]
    pub height: u32,
    #[ser(expand, head = none)]
    pub ingredients: &'a [Ingredient<'a>],
    pub result: ItemStack,
    pub show_notification: bool,
}

#[derive(Writable, Copy, Clone)]
#[repr(u8)]
pub enum CraftingCategory {
    Building,
    Redstone,
    Equipment,
    Misc,
}

#[derive(Writable, Copy, Clone)]
pub struct Ingredient<'a> {
    #[ser(expand)]
    pub item: &'a [ItemStack],
}
