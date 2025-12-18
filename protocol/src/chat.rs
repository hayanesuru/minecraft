mod binary;

use crate::dialog::Dialog;
use crate::item::ItemStack;
use crate::nbt::StringTagRaw;
use crate::profile::Profile;
use crate::str::SmolStr;
use crate::{Holder, Identifier};
use alloc::alloc::{Allocator, Global};
use alloc::boxed::Box;
use alloc::vec::Vec;
use minecraft_data::entity_type;
use uuid::Uuid;

const TYPE: &[u8] = b"type";
const EXTRA: &[u8] = b"extra";
const TEXT: &[u8] = b"text";
const TRANSLATE: &[u8] = b"translate";
const TRANSLATE_FALLBACK: &[u8] = b"fallback";
const TRANSLATE_WITH: &[u8] = b"with";
const SCORE: &[u8] = b"score";
const SCORE_NAME: &[u8] = b"name";
const SCORE_OBJECTIVE: &[u8] = b"objective";
pub const SELECTOR: StringTagRaw = StringTagRaw::new_unchecked(b"selector");
pub const KEYBIND: StringTagRaw = StringTagRaw::new_unchecked(b"keybind");
pub const NBT: StringTagRaw = StringTagRaw::new_unchecked(b"nbt");
pub const NBT_INTERPRET: StringTagRaw = StringTagRaw::new_unchecked(b"interpret");
pub const NBT_BLOCK: StringTagRaw = StringTagRaw::new_unchecked(b"block");
pub const NBT_ENTITY: StringTagRaw = StringTagRaw::new_unchecked(b"entity");
pub const NBT_STORAGE: StringTagRaw = StringTagRaw::new_unchecked(b"storage");
pub const SEPARATOR: StringTagRaw = StringTagRaw::new_unchecked(b"separator");
pub const OBJECT_ATLAS: StringTagRaw = StringTagRaw::new_unchecked(b"atlas");
pub const OBJECT_SPRITE: StringTagRaw = StringTagRaw::new_unchecked(b"sprite");
pub const OBJECT_HAT: StringTagRaw = StringTagRaw::new_unchecked(b"hat");
pub const OBJECT_PLAYER: StringTagRaw = StringTagRaw::new_unchecked(b"player");
pub const OBJECT_PLAYER_NAME: StringTagRaw = StringTagRaw::new_unchecked(b"name");
pub const OBJECT_PLAYER_ID: StringTagRaw = StringTagRaw::new_unchecked(b"id");
pub const OBJECT_PLAYER_PROPERTIES: StringTagRaw = StringTagRaw::new_unchecked(b"properties");
pub const OBJECT_PLAYER_TEXTURE: StringTagRaw = StringTagRaw::new_unchecked(b"texture");
pub const PROFILE_PROPERTY_NAME: StringTagRaw = StringTagRaw::new_unchecked(b"name");
pub const PROFILE_PROPERTY_VALUE: StringTagRaw = StringTagRaw::new_unchecked(b"value");
pub const PROFILE_PROPERTY_SIGNATURE: StringTagRaw = StringTagRaw::new_unchecked(b"signature");
pub const FONT: StringTagRaw = StringTagRaw::new_unchecked(b"font");
const COLOR: &[u8] = b"color";
pub const SHADOW_COLOR: StringTagRaw = StringTagRaw::new_unchecked(b"shadow_color");
pub const INSERTION: StringTagRaw = StringTagRaw::new_unchecked(b"insertion");
pub const CLICK_EVENT_SNAKE: StringTagRaw = StringTagRaw::new_unchecked(b"click_event");
pub const CLICK_EVENT_ACTION: StringTagRaw = StringTagRaw::new_unchecked(b"action");
pub const CLICK_EVENT_VALUE: StringTagRaw = StringTagRaw::new_unchecked(b"value");
pub const CLICK_EVENT_URL: StringTagRaw = StringTagRaw::new_unchecked(b"url");
pub const CLICK_EVENT_PATH: StringTagRaw = StringTagRaw::new_unchecked(b"path");
pub const CLICK_EVENT_COMMAND: StringTagRaw = StringTagRaw::new_unchecked(b"command");
pub const CLICK_EVENT_PAGE: StringTagRaw = StringTagRaw::new_unchecked(b"page");
pub const CLICK_EVENT_ID: StringTagRaw = StringTagRaw::new_unchecked(b"id");
pub const CLICK_EVENT_PAYLOAD: StringTagRaw = StringTagRaw::new_unchecked(b"payload");
pub const HOVER_EVENT_SNAKE: StringTagRaw = StringTagRaw::new_unchecked(b"hover_event");
pub const HOVER_EVENT_ACTION: StringTagRaw = StringTagRaw::new_unchecked(b"action");
pub const SHOW_ENTITY_ID: StringTagRaw = StringTagRaw::new_unchecked(b"id");
pub const SHOW_ENTITY_UUID: StringTagRaw = StringTagRaw::new_unchecked(b"uuid");
pub const SHOW_ENTITY_NAME: StringTagRaw = StringTagRaw::new_unchecked(b"name");
pub const SHOW_ITEM_ID: StringTagRaw = StringTagRaw::new_unchecked(b"id");
pub const SHOW_ITEM_COUNT: StringTagRaw = StringTagRaw::new_unchecked(b"count");
pub const SHOW_ITEM_COMPONENTS: StringTagRaw = StringTagRaw::new_unchecked(b"components");

const HEX_PREFIX: u8 = b'#';

#[derive(Clone)]
pub struct Component<A: Allocator = Global> {
    pub content: Content<A>,
    pub style: Style<A>,
    pub children: Vec<Component<A>, A>,
}

#[derive(Clone)]
pub enum Content<A: Allocator = Global> {
    Literal {
        content: SmolStr<A>,
    },
    Translatable {
        key: SmolStr<A>,
        fallback: Option<SmolStr<A>>,
        args: Vec<Component<A>, A>,
    },
    Score {
        name: SmolStr<A>,
        objective: SmolStr<A>,
    },
    Selector {
        pattern: SmolStr<A>,
        separator: Option<Box<Component<A>, A>>,
    },
    Keybind {
        keybind: SmolStr<A>,
    },
    BlockNbt {
        nbt_path: SmolStr<A>,
        interpret: Option<bool>,
        separator: Option<Box<Component<A>, A>>,
        pos: SmolStr<A>,
    },
    EntityNbt {
        nbt_path: SmolStr<A>,
        interpret: Option<bool>,
        separator: Option<Box<Component<A>, A>>,
        selector: SmolStr<A>,
    },
    StorageNbt {
        nbt_path: SmolStr<A>,
        interpret: Option<bool>,
        separator: Option<Box<Component<A>, A>>,
        storage: Identifier<A>,
    },
    Object {
        contents: ObjectContents<A>,
    },
}

impl Component {
    pub const fn empty() -> Self {
        Self {
            content: Content::Literal {
                content: SmolStr::EMPTY,
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
pub struct Style<A: Allocator = Global> {
    pub font: Option<SmolStr<A>>,
    pub color: Option<TextColor>,
    pub shadow_color: Option<ShadowColor>,
    pub decorations: DecorationMap,
    pub click_event: Option<ClickEvent<A>>,
    pub hover_event: Option<HoverEvent<A>>,
    pub insertion: Option<SmolStr<A>>,
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

impl<A: Allocator> Style<A> {
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
                let (a, b) = mser::u8_to_hex(rgb.red);
                let (c, d) = mser::u8_to_hex(rgb.green);
                let (e, f) = mser::u8_to_hex(rgb.blue);
                *buf = [HEX_PREFIX, a, b, c, d, e, f];
                unsafe { core::str::from_utf8_unchecked(buf) }
            }
        }
    }

    pub fn parse(n: &[u8]) -> Option<Self> {
        match TextColorNamed::parse(n) {
            Some(x) => Some(Self::Named(x)),
            None => {
                let hex = match n {
                    [HEX_PREFIX, rest @ ..] => rest,
                    _ => return None,
                };
                let (a, b) = mser::parse_hex::<u32>(hex);
                if b == hex.len() && a <= 0xffffff {
                    Some(Self::Rgb(TextColorRgb {
                        red: (a >> 16) as u8,
                        green: ((a >> 8) & 0xff) as u8,
                        blue: (a & 0xff) as u8,
                    }))
                } else {
                    None
                }
            }
        }
    }
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

#[derive(Clone, Copy, PartialEq, Eq)]
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
pub enum ClickEvent<A: Allocator = Global> {
    OpenUrl(SmolStr<A>),
    OpenFile(SmolStr<A>),
    RunCommand(SmolStr<A>),
    SuggestCommand(SmolStr<A>),
    ChangePage(u32),
    CopyToClipboard(SmolStr<A>),
    ShowDialog(Holder<Box<Dialog<A>, A>, A>),
    Custom(Identifier<A>, SmolStr<A>),
}

#[derive(Clone)]
pub enum HoverEvent<A: Allocator = Global> {
    ShowEntity {
        id: entity_type,
        uuid: Uuid,
        name: Option<Box<Component<A>, A>>,
    },
    ShowItem {
        item: ItemStack<A>,
    },
    ShowText {
        value: Box<Component<A>, A>,
    },
}

#[derive(Clone)]
pub enum ObjectContents<A: Allocator = Global> {
    Atlas {
        atlas: Option<Identifier<A>>,
        sprite: Identifier<A>,
    },
    Player {
        player: Box<Profile<A>, A>,
        hat: Option<bool>,
    },
}
