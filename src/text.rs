mod color;
mod component;
mod formatting;

pub use self::color::Color;
pub use self::component::{Literal, Translate};
pub use self::formatting::Formatting;
use crate::{UnsafeWriter, Write};

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum ChatVisibility {
    Full,
    System,
    Hidden,
}

impl From<u8> for ChatVisibility {
    fn from(value: u8) -> Self {
        if value > 2 {
            Self::Full
        } else {
            unsafe { core::mem::transmute(value) }
        }
    }
}

impl Write for ChatVisibility {
    #[inline]
    fn write(&self, w: &mut UnsafeWriter) {
        w.write_byte(*self as u8);
    }

    #[inline]
    fn len(&self) -> usize {
        1
    }
}
