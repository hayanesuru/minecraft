use crate::item::{ItemStack, TypedDataComponent};
use crate::{Component, HolderSet, ResourceTexture, serverbound};
use haya_collection::{List, Map};
use haya_ident::Ident;
use haya_nbt::Tag;
use minecraft_data::{block, data_component_predicate_type, data_component_type};
use mser::{Either, Read, Utf8, Write};

#[derive(Clone, Serialize, Deserialize)]
pub struct BlockPredicate<'a> {
    pub blocks: Option<HolderSet<'a, block>>,
    pub properties: Option<StatePropertiesPredicate<'a>>,
    pub nbt: Option<Tag>,
    pub components: DataComponentMatchers<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct StatePropertiesPredicate<'a> {
    pub properties: List<'a, PropertyMatcher<'a>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PropertyMatcher<'a> {
    pub name: Utf8<'a>,
    pub value_matcher: ValueMatcher<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ValueMatcher<'a>(pub Either<ExactMatcher<'a>, RangedMatcher<'a>>);

#[derive(Clone, Serialize, Deserialize)]
pub struct ExactMatcher<'a>(pub Utf8<'a>);

#[derive(Clone, Serialize, Deserialize)]
pub struct RangedMatcher<'a> {
    pub min: Utf8<'a>,
    pub max: Utf8<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DataComponentMatchers<'a> {
    pub exact: List<'a, TypedDataComponent<'a>>,
    pub partial: List<'a, SingleDataComponentPredicate, 64>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SingleDataComponentPredicate(
    pub Either<data_component_predicate_type, data_component_type>,
    pub Tag,
);

#[derive(Clone, Serialize, Deserialize)]
pub struct AdvancementHolder<'a> {
    pub id: Ident<'a>,
    pub value: Advancement<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Advancement<'a> {
    pub parent: Option<Ident<'a>>,
    pub display: Option<DisplayInfo<'a>>,
    pub requirements: AdvancementRequirements<'a>,
    pub sends_telemetry_event: bool,
}

#[derive(Clone)]
pub struct DisplayInfo<'a> {
    pub title: Component,
    pub description: Component,
    pub icon: ItemStack<'a>,
    pub ty: AdvancementType,
    pub background: Option<ResourceTexture<'a>>,
    pub show_toast: bool,
    pub hidden: bool,
    pub x: f32,
    pub y: f32,
}

impl<'a> Read<'a> for DisplayInfo<'a> {
    fn read(buf: &mut mser::Reader<'a>) -> Result<Self, mser::Error> {
        let title = Component::read(buf)?;
        let description = Component::read(buf)?;
        let icon = ItemStack::read(buf)?;
        let ty = AdvancementType::read(buf)?;
        let flags = u32::read(buf)?;
        let background = if flags & 1 != 0 {
            Some(ResourceTexture::read(buf)?)
        } else {
            None
        };
        let show_toast = flags & 2 != 0;
        let hidden = flags & 4 != 0;
        let x = f32::read(buf)?;
        let y = f32::read(buf)?;
        Ok(Self {
            title,
            description,
            icon,
            ty,
            background,
            show_toast,
            hidden,
            x,
            y,
        })
    }
}

impl<'a> Write for DisplayInfo<'a> {
    unsafe fn write(&self, w: &mut mser::Writer) {
        unsafe {
            self.title.write(w);
            self.description.write(w);
            self.icon.write(w);
            self.ty.write(w);
            let mut flags = self.background.is_some() as u32;
            if self.show_toast {
                flags |= 2;
            }
            if self.hidden {
                flags |= 4;
            }
            flags.write(w);
            if let Some(background) = &self.background {
                background.write(w);
            }
            self.x.write(w);
            self.y.write(w);
        }
    }

    fn len_s(&self) -> usize {
        let mut l = 0;
        l += self.title.len_s();
        l += self.description.len_s();
        l += self.icon.len_s();
        l += self.ty.len_s();
        let mut flags = self.background.is_some() as u32;
        if self.show_toast {
            flags |= 2;
        }
        if self.hidden {
            flags |= 4;
        }
        l += flags.len_s();
        if let Some(background) = &self.background {
            l += background.len_s();
        }
        l += self.x.len_s();
        l += self.y.len_s();
        l
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum AdvancementType {
    Task,
    Challenge,
    Goal,
}

impl AdvancementType {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Task => "task",
            Self::Challenge => "challenge",
            Self::Goal => "goal",
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AdvancementRequirements<'a> {
    pub requirements: List<'a, List<'a, Utf8<'a>>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AdvancementProgress<'a> {
    pub criteria: Map<'a, Utf8<'a>, CriterionProgress>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CriterionProgress {
    pub obtained: Option<u64>,
}
