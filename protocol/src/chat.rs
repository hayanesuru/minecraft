use alloc::alloc::{Allocator, Global};
use alloc::vec::Vec;
use mser::SmolStr;

const TEXT: &str = "text";
const TRANSLATE: &str = "translate";
const TRANSLATE_FALLBACK: &str = "fallback";
const TRANSLATE_WITH: &str = "with";
const SCORE: &str = "score";
const SCORE_NAME: &str = "name";
const SCORE_OBJECTIVE: &str = "objective";
const SELECTOR: &str = "selector";
const KEYBIND: &str = "keybind";
const EXTRA: &str = "extra";
const NBT: &str = "nbt";
const NBT_INTERPRET: &str = "interpret";
const NBT_BLOCK: &str = "block";
const NBT_ENTITY: &str = "entity";
const NBT_STORAGE: &str = "storage";
const SEPARATOR: &str = "separator";
const OBJECT_ATLAS: &str = "atlas";
const OBJECT_SPRITE: &str = "sprite";
const OBJECT_HAT: &str = "hat";
const OBJECT_PLAYER: &str = "player";
const OBJECT_PLAYER_NAME: &str = "name";
const OBJECT_PLAYER_ID: &str = "id";
const OBJECT_PLAYER_PROPERTIES: &str = "properties";
const OBJECT_PLAYER_TEXTURE: &str = "texture";
const PROFILE_PROPERTY_NAME: &str = "name";
const PROFILE_PROPERTY_VALUE: &str = "value";
const PROFILE_PROPERTY_SIGNATURE: &str = "signature";
const FONT: &str = "font";
const COLOR: &str = "color";
const SHADOW_COLOR: &str = "shadow_color";
const INSERTION: &str = "insertion";
const CLICK_EVENT_SNAKE: &str = "click_event";
const CLICK_EVENT_ACTION: &str = "action";
const CLICK_EVENT_VALUE: &str = "value";
const CLICK_EVENT_URL: &str = "url";
const CLICK_EVENT_PATH: &str = "path";
const CLICK_EVENT_COMMAND: &str = "command";
const CLICK_EVENT_PAGE: &str = "page";
const CLICK_EVENT_ID: &str = "id";
const CLICK_EVENT_PAYLOAD: &str = "payload";
const HOVER_EVENT_SNAKE: &str = "hover_event";
const HOVER_EVENT_ACTION: &str = "action";
const SHOW_ENTITY_ID: &str = "id";
const SHOW_ENTITY_UUID: &str = "uuid";
const SHOW_ENTITY_NAME: &str = "name";
const SHOW_ITEM_ID: &str = "id";
const SHOW_ITEM_COUNT: &str = "count";
const SHOW_ITEM_COMPONENTS: &str = "components";

const HEX_PREFIX: u8 = b'#';

#[derive(Clone)]
pub enum Component<A: Allocator = Global> {
    Literal {
        content: SmolStr,
        children: Vec<Component<A>, A>,
        style: Style,
    },
}

impl Component {
    pub const EMPTY: Self = Self::Literal {
        content: SmolStr::new_static(""),
        children: Vec::new(),
        style: Style {
            font: None,
            color: None,
            shadow_color: None,
            decorations: DecorationMap::new(),
        },
    };
}

#[derive(Clone)]
pub struct Style {
    pub font: Option<SmolStr>,
    pub color: Option<TextColor>,
    pub shadow_color: Option<ShadowColor>,
    pub decorations: DecorationMap,
}

#[derive(Clone, Copy)]
pub enum TextColor {
    Named(TextColorNamed),
    Rgb(TextColorRgb),
}

#[derive(Clone, Copy)]
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct TextColorRgb {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
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

#[derive(Clone, Copy)]
pub struct ShadowColor {
    pub value: u32,
}

impl ShadowColor {
    pub const NONE: Self = Self { value: 0 };
}

#[derive(Clone, Copy)]
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
