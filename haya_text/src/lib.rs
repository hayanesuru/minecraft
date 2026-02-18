#![no_std]

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
            Self::Black => "black",
            Self::DarkBlue => "dark_blue",
            Self::DarkGreen => "dark_green",
            Self::DarkAqua => "dark_aqua",
            Self::DarkRed => "dark_red",
            Self::DarkPurple => "dark_purple",
            Self::Gold => "gold",
            Self::Gray => "gray",
            Self::DarkGray => "dark_gray",
            Self::Blue => "blue",
            Self::Green => "green",
            Self::Aqua => "aqua",
            Self::Red => "red",
            Self::LightPurple => "light_purple",
            Self::Yellow => "yellow",
            Self::White => "white",
        }
    }

    pub const fn parse(n: &[u8]) -> Option<Self> {
        Some(match n {
            b"black" => Self::Black,
            b"dark_blue" => Self::DarkBlue,
            b"dark_green" => Self::DarkGreen,
            b"dark_aqua" => Self::DarkAqua,
            b"dark_red" => Self::DarkRed,
            b"dark_purple" => Self::DarkPurple,
            b"gold" => Self::Gold,
            b"gray" => Self::Gray,
            b"dark_gray" => Self::DarkGray,
            b"blue" => Self::Blue,
            b"green" => Self::Green,
            b"aqua" => Self::Aqua,
            b"red" => Self::Red,
            b"light_purple" => Self::LightPurple,
            b"yellow" => Self::Yellow,
            b"white" => Self::White,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DecorationMap {
    pub value: u16,
}

impl Default for DecorationMap {
    fn default() -> Self {
        Self::new()
    }
}

impl DecorationMap {
    pub const fn new() -> Self {
        Self { value: 0 }
    }

    pub const fn is_empty(self) -> bool {
        self.value == 0
    }

    pub const fn obfuscated(self) -> Option<bool> {
        match self.value & 0x0003 {
            0x0001 => Some(true),
            0x0002 => Some(false),
            _ => None,
        }
    }

    pub const fn with_obfuscated(self, obfuscated: Option<bool>) -> Self {
        let n = match obfuscated {
            Some(true) => 0x0001,
            Some(false) => 0x0002,
            None => 0x0003,
        };
        Self {
            value: (self.value & !0x0003) | n,
        }
    }

    pub const fn bold(self) -> Option<bool> {
        match self.value & 0x000C {
            0x0004 => Some(true),
            0x0008 => Some(false),
            _ => None,
        }
    }

    pub const fn with_bold(self, bold: Option<bool>) -> Self {
        let n = match bold {
            Some(true) => 0x0004,
            Some(false) => 0x0008,
            None => 0x000C,
        };
        Self {
            value: (self.value & !0x000C) | n,
        }
    }

    pub const fn strikethrough(self) -> Option<bool> {
        match self.value & 0x0030 {
            0x0010 => Some(true),
            0x0020 => Some(false),
            _ => None,
        }
    }

    pub const fn with_strikethrough(self, strikethrough: Option<bool>) -> Self {
        let n = match strikethrough {
            Some(true) => 0x0010,
            Some(false) => 0x0020,
            None => 0x0030,
        };
        Self {
            value: (self.value & !0x0030) | n,
        }
    }

    pub const fn underlined(self) -> Option<bool> {
        match self.value & 0x00C0 {
            0x0040 => Some(true),
            0x0080 => Some(false),
            _ => None,
        }
    }

    pub const fn with_underlined(self, underlined: Option<bool>) -> Self {
        let n = match underlined {
            Some(true) => 0x0040,
            Some(false) => 0x0080,
            None => 0x00C0,
        };
        Self {
            value: (self.value & !0x00C0) | n,
        }
    }

    pub const fn italic(self) -> Option<bool> {
        match self.value & 0x0300 {
            0x0100 => Some(true),
            0x0200 => Some(false),
            _ => None,
        }
    }

    pub const fn with_italic(self, italic: Option<bool>) -> Self {
        let n = match italic {
            Some(true) => 0x0100,
            Some(false) => 0x0200,
            None => 0x0300,
        };
        Self {
            value: (self.value & !0x0300) | n,
        }
    }
}
