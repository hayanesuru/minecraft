use crate::nbt::{MUTF8Tag, UTF8Tag, COMPOUND, END, LIST, STRING};
use crate::{hex_to_u8, u8_to_hex, UnsafeWriter, Write};

#[derive(Clone, Copy)]
pub struct Literal<'a> {
    pub text: &'a str,
    pub color: Option<Color>,
}

impl Write for Literal<'_> {
    fn write(&self, w: &mut UnsafeWriter) {
        if let Some(color) = self.color {
            *w += COMPOUND;
            *w += STRING;
            *w += MUTF8Tag(b"text"); // 6
            *w += UTF8Tag(self.text.as_bytes());
            *w += STRING;
            *w += MUTF8Tag(b"color"); // 7
            *w += MUTF8Tag(color.to_str(&mut [0; 7]).as_bytes()); // 9
            *w += END;
        } else {
            *w += STRING;
            *w += UTF8Tag(self.text.as_bytes());
        }
    }

    fn len(&self) -> usize {
        let text = UTF8Tag(self.text.as_bytes()).len();
        if self.color.is_some() {
            text + 22 + 4
        } else {
            text + 1
        }
    }
}

#[derive(Clone, Copy)]
pub struct Translate<'a> {
    pub translate: &'a str,
    pub args: &'a [&'a str],
    pub color: Option<Color>,
}

impl Write for Translate<'_> {
    fn write(&self, w: &mut UnsafeWriter) {
        *w += COMPOUND;
        *w += STRING;
        *w += MUTF8Tag(b"translate"); // 11
        *w += MUTF8Tag(self.translate.as_bytes());
        if !self.args.is_empty() {
            *w += LIST;
            *w += MUTF8Tag(b"with");
            *w += STRING;
            *w += self.args.len() as u32;
            for x in self.args {
                *w += UTF8Tag(x.as_bytes());
            }
        }
        if let Some(color) = self.color {
            *w += STRING;
            *w += MUTF8Tag(b"color"); // 7
            *w += MUTF8Tag(color.to_str(&mut [0; 7]).as_bytes()); // 9
        }
        *w += END;
    }

    fn len(&self) -> usize {
        let mut l = 13;
        l += MUTF8Tag(self.translate.as_bytes()).len();
        if !self.args.is_empty() {
            l += 12;
            for x in self.args {
                l += UTF8Tag(x.as_bytes()).len();
            }
        }
        if self.color.is_some() {
            l += 17;
        }
        l
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(align(4))]
pub struct Color(pub u8, pub u8, pub u8);

impl Color {
    pub const BLACK: Self = Self(0, 0, 0);
    pub const DARK_BLUE: Self = Self(0, 0, 170);
    pub const DARK_GREEN: Self = Self(0, 170, 0);
    pub const DARK_AQUA: Self = Self(0, 170, 170);
    pub const DARK_RED: Self = Self(170, 0, 0);
    pub const DARK_PURPLE: Self = Self(170, 0, 170);
    pub const GOLD: Self = Self(255, 170, 0);
    pub const GRAY: Self = Self(170, 170, 170);
    pub const DARK_GRAY: Self = Self(85, 85, 85);
    pub const BLUE: Self = Self(85, 85, 255);
    pub const GREEN: Self = Self(85, 255, 85);
    pub const AQUA: Self = Self(85, 255, 255);
    pub const RED: Self = Self(255, 85, 85);
    pub const LIGHT_PURPLE: Self = Self(255, 85, 255);
    pub const YELLOW: Self = Self(255, 255, 85);
    pub const WHITE: Self = Self(255, 255, 255);

    #[inline]
    pub const fn to_hex(self) -> [u8; 6] {
        let (a, b) = u8_to_hex(self.0);
        let (c, d) = u8_to_hex(self.1);
        let (e, f) = u8_to_hex(self.2);
        [a, b, c, d, e, f]
    }

    #[inline]
    pub fn to_str(self, x: &mut [u8; 7]) -> &str {
        let (a, b) = u8_to_hex(self.0);
        let (c, d) = u8_to_hex(self.1);
        let (e, f) = u8_to_hex(self.2);
        *x = [b'#', a, b, c, d, e, f];
        unsafe { core::str::from_utf8_unchecked(x) }
    }

    pub fn parse_ascii(s: &[u8]) -> Option<Self> {
        match s {
            [b'#', r0, r1, g0, g1, b0, b1] => Some(Self(
                hex_to_u8(*r0)? << 4 | hex_to_u8(*r1)?,
                hex_to_u8(*g0)? << 4 | hex_to_u8(*g1)?,
                hex_to_u8(*b0)? << 4 | hex_to_u8(*b1)?,
            )),
            b"aqua" => Some(Self::AQUA),
            b"black" => Some(Self::BLACK),
            b"blue" => Some(Self::BLUE),
            b"dark_aqua" => Some(Self::DARK_AQUA),
            b"dark_blue" => Some(Self::DARK_BLUE),
            b"dark_gray" => Some(Self::DARK_GRAY),
            b"dark_green" => Some(Self::DARK_GREEN),
            b"dark_purple" => Some(Self::DARK_PURPLE),
            b"dark_red" => Some(Self::DARK_RED),
            b"gold" => Some(Self::GOLD),
            b"gray" => Some(Self::GRAY),
            b"green" => Some(Self::GREEN),
            b"light_purple" => Some(Self::LIGHT_PURPLE),
            b"red" => Some(Self::RED),
            b"white" => Some(Self::WHITE),
            b"yellow" => Some(Self::YELLOW),
            _ => None,
        }
    }

    pub fn from_f32(r: f32, g: f32, b: f32) -> Self {
        Self(
            (r.clamp(0.0, 1.0) * 255.0) as u8,
            (g.clamp(0.0, 1.0) * 255.0) as u8,
            (b.clamp(0.0, 1.0) * 255.0) as u8,
        )
    }

    #[inline]
    pub const fn gray(x: u8) -> Self {
        Self(x, x, x)
    }

    #[inline]
    pub fn gray_f32(x: f32) -> Self {
        Self::from_f32(x, x, x)
    }
}

impl From<(u8, u8, u8)> for Color {
    #[inline]
    fn from((r, g, b): (u8, u8, u8)) -> Self {
        Self(r, g, b)
    }
}

impl From<(f32, f32, f32)> for Color {
    #[inline]
    fn from((r, g, b): (f32, f32, f32)) -> Self {
        Self::from_f32(r, g, b)
    }
}

impl From<u32> for Color {
    #[inline]
    fn from(value: u32) -> Self {
        Self((value >> 16) as u8, (value >> 8) as u8, value as u8)
    }
}
