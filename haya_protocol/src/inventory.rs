use crate::Translatable;

#[derive(Clone, Copy, Serialize, Deserialize, Name)]
#[repr(u8)]
#[mser(varint)]
pub enum HumanoidArm {
    Left,
    Right,
}

impl HumanoidArm {
    pub const fn translation_key(self) -> Translatable<'static> {
        Translatable("options.mainHand.", self.name())
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct ContainerId(#[mser(varint)] pub u32);

#[derive(Clone, Copy, Serialize, Deserialize, Name)]
#[repr(u8)]
#[mser(varint)]
pub enum EquipmentSlotGroup {
    Any,
    Mainhand,
    Offhand,
    Hand,
    Feet,
    Legs,
    Chest,
    Head,
    Armor,
    Body,
    Saddle,
}

#[derive(Clone, Copy, Serialize, Deserialize, Name)]
#[repr(u8)]
#[mser(varint)]
pub enum EquipmentSlot {
    Mainhand,
    Feet,
    Legs,
    Chest,
    Head,
    Offhand,
    Body,
    Saddle,
}

impl EquipmentSlot {
    pub const fn new(n: u8) -> Self {
        if n > Self::Saddle as u8 {
            Self::Mainhand
        } else {
            unsafe { core::mem::transmute::<u8, Self>(n) }
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum InteractionHand {
    MainHand,
    OffHand,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum RecipeBookType {
    Crafting,
    Furnace,
    BlastFurnace,
    Smoker,
}
