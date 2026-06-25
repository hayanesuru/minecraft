use crate::key;
use core::str::FromStr;
use haya_nbt::{Deserialize, Serialize, StringTag, Tag};
use haya_str::{hex_to_u8, u8_to_hex};
use mser::Error;

#[derive(Clone, Copy)]
pub enum TextColor {
    Named(TextColorNamed),
    Rgb(TextColorRgb),
}

impl TextColor {
    pub const fn name(self, buf: &mut [u8; 7]) -> &str {
        match self {
            Self::Named(named) => named.name(),
            Self::Rgb(rgb) => {
                let (a, b) = u8_to_hex(rgb.red);
                let (c, d) = u8_to_hex(rgb.green);
                let (e, f) = u8_to_hex(rgb.blue);
                *buf = [b'#', a, b, c, d, e, f];
                unsafe { core::str::from_utf8_unchecked(buf) }
            }
        }
    }
}

impl FromStr for TextColor {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(x) = s.parse() {
            return Ok(Self::Named(x));
        }
        let hex = match s.strip_prefix('#') {
            Some(rest) => rest,
            None => return Err(Error),
        };
        let a = match u32::from_str_radix(hex, 16) {
            Ok(x) => x,
            Err(_) => return Err(Error),
        };
        if a <= 0xffffff {
            Ok(Self::Rgb(TextColorRgb {
                red: (a >> 16) as u8,
                green: ((a >> 8) & 0xff) as u8,
                blue: (a & 0xff) as u8,
            }))
        } else {
            Err(Error)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextColorRgb {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextColorNamed {
    Black,
    DarkBlue,
    DarkGreen,
    DarkAqua,
    DarkRed,
    DarkPurple,
    Gold,
    Gray,
    DarkGray,
    Blue,
    Green,
    Aqua,
    Red,
    LightPurple,
    Yellow,
    White,
}

impl TextColorRgb {
    pub const fn to_u32(self) -> u32 {
        ((self.red as u32) << 16) | ((self.green as u32) << 8) | (self.blue as u32)
    }

    pub const fn to_named(self) -> Option<TextColorNamed> {
        match self.to_u32() {
            0x000000 => Some(TextColorNamed::Black),
            0x0000aa => Some(TextColorNamed::DarkBlue),
            0x00aa00 => Some(TextColorNamed::DarkGreen),
            0x00aaaa => Some(TextColorNamed::DarkAqua),
            0xaa0000 => Some(TextColorNamed::DarkRed),
            0xaa00aa => Some(TextColorNamed::DarkPurple),
            0xffaa00 => Some(TextColorNamed::Gold),
            0xaaaaaa => Some(TextColorNamed::Gray),
            0x555555 => Some(TextColorNamed::DarkGray),
            0x5555ff => Some(TextColorNamed::Blue),
            0x55ff55 => Some(TextColorNamed::Green),
            0x55ffff => Some(TextColorNamed::Aqua),
            0xff5555 => Some(TextColorNamed::Red),
            0xff55ff => Some(TextColorNamed::LightPurple),
            0xffff55 => Some(TextColorNamed::Yellow),
            0xffffff => Some(TextColorNamed::White),
            _ => None,
        }
    }
}

impl TextColorNamed {
    pub const fn to_rgb(&self) -> TextColorRgb {
        match self {
            Self::Black => TextColorRgb {
                red: 0,
                green: 0,
                blue: 0,
            },
            Self::DarkBlue => TextColorRgb {
                red: 0,
                green: 0,
                blue: 170,
            },
            Self::DarkGreen => TextColorRgb {
                red: 0,
                green: 170,
                blue: 0,
            },
            Self::DarkAqua => TextColorRgb {
                red: 0,
                green: 170,
                blue: 170,
            },
            Self::DarkRed => TextColorRgb {
                red: 170,
                green: 0,
                blue: 0,
            },
            Self::DarkPurple => TextColorRgb {
                red: 170,
                green: 0,
                blue: 170,
            },
            Self::Gold => TextColorRgb {
                red: 255,
                green: 170,
                blue: 0,
            },
            Self::Gray => TextColorRgb {
                red: 170,
                green: 170,
                blue: 170,
            },
            Self::DarkGray => TextColorRgb {
                red: 85,
                green: 85,
                blue: 85,
            },
            Self::Blue => TextColorRgb {
                red: 85,
                green: 85,
                blue: 255,
            },
            Self::Green => TextColorRgb {
                red: 85,
                green: 255,
                blue: 85,
            },
            Self::Aqua => TextColorRgb {
                red: 85,
                green: 255,
                blue: 255,
            },
            Self::Red => TextColorRgb {
                red: 255,
                green: 85,
                blue: 85,
            },
            Self::LightPurple => TextColorRgb {
                red: 255,
                green: 85,
                blue: 255,
            },
            Self::Yellow => TextColorRgb {
                red: 255,
                green: 255,
                blue: 85,
            },
            Self::White => TextColorRgb {
                red: 255,
                green: 255,
                blue: 255,
            },
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Black => BLACK,
            Self::DarkBlue => DARK_BLUE,
            Self::DarkGreen => DARK_GREEN,
            Self::DarkAqua => DARK_AQUA,
            Self::DarkRed => DARK_RED,
            Self::DarkPurple => DARK_PURPLE,
            Self::Gold => GOLD,
            Self::Gray => GRAY,
            Self::DarkGray => DARK_GRAY,
            Self::Blue => BLUE,
            Self::Green => GREEN,
            Self::Aqua => AQUA,
            Self::Red => RED,
            Self::LightPurple => LIGHT_PURPLE,
            Self::Yellow => YELLOW,
            Self::White => WHITE,
        }
    }
}

const BLACK: &str = "black";
const DARK_BLUE: &str = "dark_blue";
const DARK_GREEN: &str = "dark_green";
const DARK_AQUA: &str = "dark_aqua";
const DARK_RED: &str = "dark_red";
const DARK_PURPLE: &str = "dark_purple";
const GOLD: &str = "gold";
const GRAY: &str = "gray";
const DARK_GRAY: &str = "dark_gray";
const BLUE: &str = "blue";
const GREEN: &str = "green";
const AQUA: &str = "aqua";
const RED: &str = "red";
const LIGHT_PURPLE: &str = "light_purple";
const YELLOW: &str = "yellow";
const WHITE: &str = "white";
const BLACK_K: StringTag = key(BLACK);
const DARK_BLUE_K: StringTag = key(DARK_BLUE);
const DARK_GREEN_K: StringTag = key(DARK_GREEN);
const DARK_AQUA_K: StringTag = key(DARK_AQUA);
const DARK_RED_K: StringTag = key(DARK_RED);
const DARK_PURPLE_K: StringTag = key(DARK_PURPLE);
const GOLD_K: StringTag = key(GOLD);
const GRAY_K: StringTag = key(GRAY);
const DARK_GRAY_K: StringTag = key(DARK_GRAY);
const BLUE_K: StringTag = key(BLUE);
const GREEN_K: StringTag = key(GREEN);
const AQUA_K: StringTag = key(AQUA);
const RED_K: StringTag = key(RED);
const LIGHT_PURPLE_K: StringTag = key(LIGHT_PURPLE);
const YELLOW_K: StringTag = key(YELLOW);
const WHITE_K: StringTag = key(WHITE);

impl FromStr for TextColorNamed {
    type Err = Error;

    fn from_str(n: &str) -> Result<Self, Self::Err> {
        Ok(match n {
            BLACK => Self::Black,
            DARK_BLUE => Self::DarkBlue,
            DARK_GREEN => Self::DarkGreen,
            DARK_AQUA => Self::DarkAqua,
            DARK_RED => Self::DarkRed,
            DARK_PURPLE => Self::DarkPurple,
            GOLD => Self::Gold,
            GRAY => Self::Gray,
            DARK_GRAY => Self::DarkGray,
            BLUE => Self::Blue,
            GREEN => Self::Green,
            AQUA => Self::Aqua,
            RED => Self::Red,
            LIGHT_PURPLE => Self::LightPurple,
            YELLOW => Self::Yellow,
            WHITE => Self::White,
            _ => return Err(Error),
        })
    }
}

impl Serialize for TextColor {
    fn serialize(&self) -> Tag {
        match self {
            Self::Named(text_color_named) => Tag::String(match text_color_named {
                TextColorNamed::Black => BLACK_K,
                TextColorNamed::DarkBlue => DARK_BLUE_K,
                TextColorNamed::DarkGreen => DARK_GREEN_K,
                TextColorNamed::DarkAqua => DARK_AQUA_K,
                TextColorNamed::DarkRed => DARK_RED_K,
                TextColorNamed::DarkPurple => DARK_PURPLE_K,
                TextColorNamed::Gold => GOLD_K,
                TextColorNamed::Gray => GRAY_K,
                TextColorNamed::DarkGray => DARK_GRAY_K,
                TextColorNamed::Blue => BLUE_K,
                TextColorNamed::Green => GREEN_K,
                TextColorNamed::Aqua => AQUA_K,
                TextColorNamed::Red => RED_K,
                TextColorNamed::LightPurple => LIGHT_PURPLE_K,
                TextColorNamed::Yellow => YELLOW_K,
                TextColorNamed::White => WHITE_K,
            }),
            Self::Rgb(text_color_rgb) => unsafe {
                let mut a = [0; 7];
                a[0] = b'#';
                let (r0, r1) = u8_to_hex(text_color_rgb.red);
                a[1] = r0;
                a[2] = r1;
                let (g0, g1) = u8_to_hex(text_color_rgb.green);
                a[3] = g0;
                a[4] = g1;
                let (b0, b1) = u8_to_hex(text_color_rgb.blue);
                a[5] = b0;
                a[6] = b1;
                let s = core::str::from_utf8_unchecked(&a);
                Tag::String(match StringTag::from_ascii_nunzero_unchecked(s) {
                    Some(x) => x,
                    None => StringTag::from_utf8(s),
                })
            },
        }
    }
}

impl Deserialize for TextColor {
    fn deserialize(nbt: Tag) -> Result<Self, Error> {
        let s = match nbt {
            Tag::String(s) => s,
            _ => return Err(Error),
        };
        if let Some(bytes) = s.strip_prefix('#').map(|x| x.as_bytes()) {
            let [c0, c1, c2, c3, c4, c5] = bytes[..] else {
                return Err(Error);
            };
            let r1 = hex_to_u8(c0);
            let r2 = hex_to_u8(c1);
            let g1 = hex_to_u8(c2);
            let g2 = hex_to_u8(c3);
            let b1 = hex_to_u8(c4);
            let b2 = hex_to_u8(c5);
            let (Some(a), Some(b), Some(c), Some(d), Some(e), Some(f)) = (r1, r2, g1, g2, b1, b2)
            else {
                return Err(Error);
            };
            Ok(Self::Rgb(TextColorRgb {
                red: (a << 4) | b,
                green: (c << 4) | d,
                blue: (e << 4) | f,
            }))
        } else {
            Ok(Self::Named(TextColorNamed::from_str(&s)?))
        }
    }
}
