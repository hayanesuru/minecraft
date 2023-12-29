use crate::{hex_to_u8, u8_to_hex};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
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
    pub fn to_hex_str(self, x: &mut [u8; 6]) -> &str {
        let (a, b) = u8_to_hex(self.0);
        let (c, d) = u8_to_hex(self.1);
        let (e, f) = u8_to_hex(self.2);
        *x = [a, b, c, d, e, f];
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

    /// Creates a new [Color] color with three [f32] values
    pub fn from_f32(r: f32, g: f32, b: f32) -> Self {
        Self(
            (r.clamp(0.0, 1.0) * 255.0) as u8,
            (g.clamp(0.0, 1.0) * 255.0) as u8,
            (b.clamp(0.0, 1.0) * 255.0) as u8,
        )
    }

    /// Creates a grayscale [Color] color
    #[inline]
    pub const fn gray(x: u8) -> Self {
        Self(x, x, x)
    }

    /// Creates a grayscale [Color] color with a [f32] value
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
