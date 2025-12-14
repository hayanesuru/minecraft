use crate::dialog::Dialog;
use crate::item::ItemStack;
use crate::nbt::{StringTag, StringTagRaw, StringTagWriter, TagType};
use crate::profile::Profile;
use crate::str::SmolStr;
use crate::{Holder, Identifier};
use alloc::alloc::{Allocator, Global};
use alloc::boxed::Box;
use alloc::vec::Vec;
use minecraft_data::entity_type;
use mser::{Error, Read, UnsafeWriter, Write};
use uuid::Uuid;

pub const TEXT: StringTagRaw = StringTagRaw::new_unchecked(b"text");
pub const TRANSLATE: StringTagRaw = StringTagRaw::new_unchecked(b"translate");
pub const TRANSLATE_FALLBACK: StringTagRaw = StringTagRaw::new_unchecked(b"fallback");
pub const TRANSLATE_WITH: StringTagRaw = StringTagRaw::new_unchecked(b"with");
pub const SCORE: StringTagRaw = StringTagRaw::new_unchecked(b"score");
pub const SCORE_NAME: StringTagRaw = StringTagRaw::new_unchecked(b"name");
pub const SCORE_OBJECTIVE: StringTagRaw = StringTagRaw::new_unchecked(b"objective");
pub const SELECTOR: StringTagRaw = StringTagRaw::new_unchecked(b"selector");
pub const KEYBIND: StringTagRaw = StringTagRaw::new_unchecked(b"keybind");
pub const EXTRA: StringTagRaw = StringTagRaw::new_unchecked(b"extra");
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
pub const COLOR: StringTagRaw = StringTagRaw::new_unchecked(b"color");
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

pub const HEX_PREFIX: u8 = b'#';

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
        fallback: SmolStr<A>,
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
    Objects {
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

impl<A: Allocator> Write for Component<A> {
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            let Self {
                content,
                style,
                children,
            } = self;
            if let Content::Literal { content } = content
                && style.is_empty()
                && children.is_empty()
            {
                StringTagWriter(content).write(w);
                return;
            }
            match content {
                Content::Literal { content } => {
                    TagType::Compound.write(w);
                    TagType::String.write(w);
                    TEXT.write(w);
                    StringTagWriter(content).write(w);
                    style.write(w);
                    TagType::End.write(w);
                }
                _ => StringTagWriter("").write(w),
            }
        }
    }

    fn sz(&self) -> usize {
        let mut w = 0usize;
        let Self {
            content,
            style,
            children,
        } = self;
        if let Content::Literal { content } = content
            && style.is_empty()
            && children.is_empty()
        {
            w += StringTagWriter(content).sz();
            return w;
        }
        match content {
            Content::Literal { content } => {
                w += TagType::Compound.sz();
                w += TagType::String.sz();
                w += TEXT.sz();
                w += StringTagWriter(content).sz();
                w += style.sz();
                w += TagType::End.sz();
            }
            _ => {
                w += StringTagWriter("").sz();
            }
        }
        w
    }
}

impl<'a> Read<'a> for Component {
    fn read(buf: &mut &'a [u8]) -> Result<Self, Error> {
        match TagType::read(buf)? {
            TagType::String => Ok(Self {
                children: Vec::new(),
                style: Style::new(),
                content: Content::Literal {
                    content: StringTag::read(buf)?.0,
                },
            }),
            TagType::List => Err(Error),
            TagType::Compound => {
                let mut content = Content::Literal {
                    content: SmolStr::EMPTY,
                };
                let style = Style::new();
                let children = Vec::new();
                loop {
                    let tag_type = TagType::read(buf)?;
                    if tag_type == TagType::End {
                        return Ok(Self {
                            content,
                            style,
                            children,
                        });
                    }
                    let name = StringTag::read(buf)?;
                    let name = name.0.as_str();
                    match name {
                        "text" => {
                            content = Content::Literal {
                                content: StringTag::read(buf)?.0,
                            }
                        }
                        _ => return Err(Error),
                    }
                }
            }
            _ => Err(Error),
        }
    }
}

impl<A: Allocator> Style<A> {
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            if let Some(color) = self.color {
                TagType::String.write(w);
                COLOR.write(w);
                match color {
                    TextColor::Named(named) => {
                        StringTagRaw::new_unchecked(named.name().as_bytes()).write(w);
                    }
                    TextColor::Rgb(rgb) => {
                        let (a, b) = mser::u8_to_hex(rgb.red);
                        let (c, d) = mser::u8_to_hex(rgb.green);
                        let (e, f) = mser::u8_to_hex(rgb.blue);
                        let s = [b'#', a, b, c, d, e, f];
                        StringTagRaw::new_unchecked(&s).write(w);
                    }
                }
            }
        }
    }

    fn sz(&self) -> usize {
        let mut w = 0;
        if let Some(color) = self.color {
            w += TagType::String.sz();
            w += COLOR.sz();
            match color {
                TextColor::Named(named) => {
                    w += StringTagRaw::new_unchecked(named.name().as_bytes()).sz();
                }
                TextColor::Rgb(rgb) => {
                    let (a, b) = mser::u8_to_hex(rgb.red);
                    let (c, d) = mser::u8_to_hex(rgb.green);
                    let (e, f) = mser::u8_to_hex(rgb.blue);
                    let s = [b'#', a, b, c, d, e, f];
                    w += StringTagRaw::new_unchecked(&s).sz();
                }
            }
        }
        w
    }
}
