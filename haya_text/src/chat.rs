use crate::profile::ResolvableProfile;
use crate::{HolderDialog, ItemStack};
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::str::FromStr;
use haya_ident::{Ident, Identifier};
use haya_str::u8_to_hex;
use minecraft_data::entity_type;
use mser::Error;
use uuid::Uuid;

pub const TYPE: &str = "type";
pub const EXTRA: &str = "extra";
pub const TEXT: &str = "text";
pub const TRANSLATE: &str = "translate";
pub const TRANSLATE_FALLBACK: &str = "fallback";
pub const TRANSLATE_WITH: &str = "with";
pub const SCORE: &str = "score";
pub const SCORE_NAME: &str = "name";
pub const SCORE_OBJECTIVE: &str = "objective";
pub const SELECTOR: &str = "selector";
pub const SEPARATOR: &str = "separator";
pub const KEYBIND: &str = "keybind";
pub const NBT_PATH: &str = "nbt";
pub const NBT_INTERPRET: &str = "interpret";
pub const NBT_SOURCE: &str = "source";
pub const NBT_BLOCK: &str = "block";
pub const NBT_ENTITY: &str = "entity";
pub const NBT_STORAGE: &str = "storage";
pub const OBJECT_TYPE: &str = "object";
pub const OBJECT_ATLAS: &str = "atlas";
pub const OBJECT_SPRITE: &str = "sprite";
pub const OBJECT_PLAYER: &str = "player";
pub const OBJECT_HAT: &str = "hat";

pub const OBJECT_PLAYER_NAME: &str = "name";
pub const OBJECT_PLAYER_ID: &str = "id";
pub const OBJECT_PLAYER_PROPERTIES: &str = "properties";
pub const OBJECT_PLAYER_TEXTURE: &str = "texture";
pub const PROFILE_PROPERTY_NAME: &str = "name";
pub const PROFILE_PROPERTY_VALUE: &str = "value";
pub const PROFILE_PROPERTY_SIGNATURE: &str = "signature";
pub const FONT: &str = "font";
pub const COLOR: &str = "color";
pub const SHADOW_COLOR: &str = "shadow_color";
pub const INSERTION: &str = "insertion";
pub const CLICK_EVENT_SNAKE: &str = "click_event";
pub const CLICK_EVENT_ACTION: &str = "action";
pub const CLICK_EVENT_VALUE: &str = "value";
pub const CLICK_EVENT_URL: &str = "url";
pub const CLICK_EVENT_PATH: &str = "path";
pub const CLICK_EVENT_COMMAND: &str = "command";
pub const CLICK_EVENT_PAGE: &str = "page";
pub const CLICK_EVENT_ID: &str = "id";
pub const CLICK_EVENT_PAYLOAD: &str = "payload";
pub const HOVER_EVENT_SNAKE: &str = "hover_event";
pub const HOVER_EVENT_ACTION: &str = "action";
pub const SHOW_ENTITY_ID: &str = "id";
pub const SHOW_ENTITY_UUID: &str = "uuid";
pub const SHOW_ENTITY_NAME: &str = "name";
pub const SHOW_ITEM_ID: &str = "id";
pub const SHOW_ITEM_COUNT: &str = "count";
pub const SHOW_ITEM_COMPONENTS: &str = "components";

#[derive(Clone)]
pub struct Component {
    pub content: Content,
    pub style: Style,
    pub children: Vec<Component>,
}

#[derive(Clone)]
pub enum Content {
    Literal {
        content: Box<str>,
    },
    Translatable {
        key: Box<str>,
        fallback: Option<Box<str>>,
        args: Vec<Component>,
    },
    Score {
        name: Box<str>,
        objective: Box<str>,
    },
    Selector {
        pattern: Box<str>,
        separator: Option<Box<Component>>,
    },
    Keybind {
        keybind: Box<str>,
    },
    Nbt {
        nbt_path: Box<str>,
        interpret: bool,
        separator: Option<Box<Component>>,
        content: NbtContent,
    },
    Object {
        content: ObjectContents,
    },
}

pub enum ContentB {
    Literal {
        content: Box<str>,
    },
    Translatable {
        key: Option<Box<str>>,
        fallback: Option<Box<str>>,
        args: Vec<Component>,
    },
    Score {
        name: Box<str>,
        objective: Box<str>,
    },
    Selector {
        pattern: Box<str>,
    },
    Keybind {
        keybind: Box<str>,
    },
    Nbt {
        nbt_path: Option<Box<str>>,
        interpret: bool,
        content: Option<NbtContent>,
    },
    Object {
        content: ObjectContentB,
    },
}

pub enum ObjectContentB {
    Atlas {
        atlas: Option<Identifier>,
        sprite: Option<Identifier>,
    },
    Player {
        player: Option<Box<ResolvableProfile>>,
        hat: bool,
    },
}

impl ContentB {
    pub fn into_content(self, separator: Option<Box<Component>>) -> Option<Content> {
        Some(match self {
            ContentB::Literal { content } => Content::Literal { content },
            ContentB::Translatable {
                key,
                fallback,
                args,
            } => Content::Translatable {
                key: key?,
                fallback,
                args,
            },
            ContentB::Score { name, objective } => Content::Score { name, objective },
            ContentB::Selector { pattern } => Content::Selector { pattern, separator },
            ContentB::Keybind { keybind } => Content::Keybind { keybind },
            ContentB::Nbt {
                nbt_path,
                interpret,
                content,
            } => Content::Nbt {
                nbt_path: nbt_path?,
                interpret,
                separator,
                content: content?,
            },
            ContentB::Object { content } => match content {
                ObjectContentB::Atlas { atlas, sprite } => Content::Object {
                    content: ObjectContents::Atlas {
                        atlas: atlas?,
                        sprite: sprite?,
                    },
                },
                ObjectContentB::Player { player, hat } => Content::Object {
                    content: ObjectContents::Player {
                        player: player?,
                        hat,
                    },
                },
            },
        })
    }
}

impl Component {
    pub fn empty() -> Self {
        Self {
            content: Content::Literal {
                content: String::new().into_boxed_str(),
            },
            children: Vec::new(),
            style: Style {
                font: None,
                color: None,
                shadow_color: None,
                decorations: DecorationMap::new(),
                click_event: None,
                hover_event: None,
                insertion: None,
            },
        }
    }
}

#[derive(Clone)]
pub struct Style {
    pub font: Option<Box<str>>,
    pub color: Option<TextColor>,
    pub shadow_color: Option<ShadowColor>,
    pub decorations: DecorationMap,
    pub click_event: Option<ClickEvent>,
    pub hover_event: Option<HoverEvent>,
    pub insertion: Option<Box<str>>,
}

impl Default for Style {
    fn default() -> Self {
        Self::new()
    }
}

impl Style {
    pub const fn new() -> Self {
        Self {
            font: None,
            color: None,
            shadow_color: None,
            decorations: DecorationMap::new(),
            click_event: None,
            hover_event: None,
            insertion: None,
        }
    }
}

impl Style {
    pub const fn is_empty(&self) -> bool {
        self.font.is_none()
            && self.color.is_none()
            && self.shadow_color.is_none()
            && self.decorations.is_empty()
            && self.click_event.is_none()
            && self.hover_event.is_none()
            && self.insertion.is_none()
    }
}

#[derive(Clone, Copy)]
pub enum TextColor {
    Named(TextColorNamed),
    Rgb(TextColorRgb),
}

impl TextColor {
    pub const fn name(self, buf: &mut [u8; 7]) -> &str {
        match self {
            TextColor::Named(named) => named.name(),
            TextColor::Rgb(rgb) => {
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
}

impl FromStr for TextColorNamed {
    type Err = Error;

    fn from_str(n: &str) -> Result<Self, Self::Err> {
        Ok(match n {
            "black" => Self::Black,
            "dark_blue" => Self::DarkBlue,
            "dark_green" => Self::DarkGreen,
            "dark_aqua" => Self::DarkAqua,
            "dark_red" => Self::DarkRed,
            "dark_purple" => Self::DarkPurple,
            "gold" => Self::Gold,
            "gray" => Self::Gray,
            "dark_gray" => Self::DarkGray,
            "blue" => Self::Blue,
            "green" => Self::Green,
            "aqua" => Self::Aqua,
            "red" => Self::Red,
            "light_purple" => Self::LightPurple,
            "yellow" => Self::Yellow,
            "white" => Self::White,
            _ => return Err(Error),
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ShadowColor {
    pub value: u32,
}

impl ShadowColor {
    pub const NONE: Self = Self { value: 0 };
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

#[derive(Clone)]
pub enum ClickEvent {
    OpenUrl(Box<str>),
    OpenFile(Box<str>),
    RunCommand(Box<str>),
    SuggestCommand(Box<str>),
    ChangePage(u32),
    CopyToClipboard(Box<str>),
    ShowDialog(HolderDialog),
    Custom(Identifier, Box<str>),
}

#[derive(Clone)]
pub enum HoverEvent {
    ShowEntity {
        id: entity_type,
        uuid: Uuid,
        name: Option<Box<Component>>,
    },
    ShowItem {
        item: ItemStack,
    },
    ShowText {
        value: Box<Component>,
    },
}

pub const DEFAULT_ATLAS: Ident = unsafe { Ident::new_unchecked(None, "blocks") };

#[derive(Clone)]
pub enum ObjectContents {
    Atlas {
        atlas: Identifier,
        sprite: Identifier,
    },
    Player {
        player: Box<ResolvableProfile>,
        hat: bool,
    },
}

#[derive(Clone)]
pub enum NbtContent {
    Block { pos: Box<str> },
    Entity { selector: Box<str> },
    Storage { storage: Identifier },
}
