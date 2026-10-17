use core::str::FromStr;
use haya_nbt::{Deserialize, Serialize, StringTag, Tag};
use haya_str::{HayaStr, hex_to_u8, u8_to_hex};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Name)]
#[repr(u8)]
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
}

impl Serialize for TextColor {
    fn serialize(&self) -> Tag {
        match self {
            Self::Named(text_color_named) => unsafe {
                Tag::String(StringTag::from_ascii_nunzero_unchecked(
                    HayaStr::copy_from(text_color_named.name()).unwrap_unchecked(),
                ))
            },
            Self::Rgb(text_color_rgb) => unsafe {
                let mut a = HayaStr::new();
                let _ = a.try_push('#');
                let (r0, r1) = u8_to_hex(text_color_rgb.red);
                let _ = a.try_push(r0 as char);
                let _ = a.try_push(r1 as char);
                let (g0, g1) = u8_to_hex(text_color_rgb.green);
                let _ = a.try_push(g0 as char);
                let _ = a.try_push(g1 as char);
                let (b0, b1) = u8_to_hex(text_color_rgb.blue);
                let _ = a.try_push(b0 as char);
                let _ = a.try_push(b1 as char);
                Tag::String(StringTag::from_ascii_nunzero_unchecked(a))
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
