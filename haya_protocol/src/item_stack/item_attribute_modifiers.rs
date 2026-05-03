use crate::attribute::AttributeModifier;
use crate::{Component, EquipmentSlotGroup};
use haya_collection::List;
use minecraft_data::attribute;
use mser::{Error, Read, Reader, V21, Write, Writer};

#[derive(Clone, Serialize, Deserialize)]
pub struct ItemAttributeModifiers<'a> {
    pub modifiers: List<'a, Entry<'a>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Entry<'a> {
    pub attribute: attribute,
    pub modifier: AttributeModifier<'a>,
    pub slot: EquipmentSlotGroup,
    pub display: Display,
}

#[derive(Clone)]
pub enum Display {
    Default,
    Hidden,
    Override(Component),
}

impl<'a> Read<'a> for Display {
    fn read(buf: &mut Reader<'a>) -> Result<Self, Error> {
        Ok(match V21::read(buf)?.0 {
            1 => Self::Hidden,
            2 => Self::Override(Component::read(buf)?),
            _ => Self::Default,
        })
    }
}

impl Write for Display {
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
