use crate::click_event::ClickEvent;
use crate::color::{TextColor, TextColorNamed, TextColorRgb};
use crate::decoration::DecorationMap;
use crate::hover_event::HoverEvent;
use crate::profile::{PlayerSkinPatch, PropertyMap, ResolvableProfile};
use crate::{capacity_fix, key};
use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use haya_ident::Identifier;
use haya_math::{f32_to_u8, f64_to_u8};
use haya_nbt::{CompoundTag, Deserialize, ListTag, Serialize, StringTag, Tag};
use mser::Error;

const TYPE: &str = "type";
const EXTRA: &str = "extra";
const TEXT: &str = "text";
const TRANSLATE: &str = "translate";
const TRANSLATE_FALLBACK: &str = "fallback";
const TRANSLATE_WITH: &str = "with";
const SCORE: &str = "score";
const SCORE_NAME: &str = "name";
const SCORE_OBJECTIVE: &str = "objective";
const SELECTOR: &str = "selector";
const SEPARATOR: &str = "separator";
const KEYBIND: &str = "keybind";
const NBT_PATH: &str = "nbt";
const NBT_INTERPRET: &str = "interpret";
const NBT_SOURCE: &str = "source";
const NBT_BLOCK: &str = "block";
const NBT_ENTITY: &str = "entity";
const NBT_STORAGE: &str = "storage";
const OBJECT_TYPE: &str = "object";
const OBJECT_ATLAS: &str = "atlas";
const OBJECT_SPRITE: &str = "sprite";
const OBJECT_PLAYER: &str = "player";
const OBJECT_HAT: &str = "hat";

const TYPE_K: StringTag = key(TYPE);
const TEXT_K: StringTag = key(TEXT);
const TRANSLATE_K: StringTag = key(TRANSLATE);
const TRANSLATE_FALLBACK_K: StringTag = key(TRANSLATE_FALLBACK);
const TRANSLATE_WITH_K: StringTag = key(TRANSLATE_WITH);
const SCORE_K: StringTag = key(SCORE);
const SCORE_NAME_K: StringTag = key(SCORE_NAME);
const SCORE_OBJECTIVE_K: StringTag = key(SCORE_OBJECTIVE);
const SELECTOR_K: StringTag = key(SELECTOR);
const KEYBIND_K: StringTag = key(KEYBIND);
const NBT_PATH_K: StringTag = key(NBT_PATH);
const OBJECT_TYPE_K: StringTag = key(OBJECT_TYPE);
const SEPARATOR_K: StringTag = key(SEPARATOR);
const NBT_INTERPRET_K: StringTag = key(NBT_INTERPRET);
const NBT_SOURCE_K: StringTag = key(NBT_SOURCE);
const NBT_BLOCK_K: StringTag = key(NBT_BLOCK);
const NBT_ENTITY_K: StringTag = key(NBT_ENTITY);
const NBT_STORAGE_K: StringTag = key(NBT_STORAGE);
const OBJECT_ATLAS_K: StringTag = key(OBJECT_ATLAS);
const OBJECT_PLAYER_K: StringTag = key(OBJECT_PLAYER);
const OBJECT_SPRITE_K: StringTag = key(OBJECT_SPRITE);
const OBJECT_HAT_K: StringTag = key(OBJECT_HAT);

const COLOR: &str = "color";
const SHADOW_COLOR: &str = "shadow_color";
const BOLD: &str = "bold";
const ITALIC: &str = "italic";
const UNDERLINED: &str = "underlined";
const STRIKETHROUGH: &str = "strikethrough";
const OBFUSCATED: &str = "obfuscated";
const CLICK_EVENT: &str = "click_event";
const HOVER_EVENT: &str = "hover_event";
const INSERTION: &str = "insertion";
const FONT: &str = "font";

const COLOR_K: StringTag = key(COLOR);
const SHADOW_COLOR_K: StringTag = key(SHADOW_COLOR);
const BOLD_K: StringTag = key(BOLD);
const ITALIC_K: StringTag = key(ITALIC);
const UNDERLINED_K: StringTag = key(UNDERLINED);
const STRIKETHROUGH_K: StringTag = key(STRIKETHROUGH);
const OBFUSCATED_K: StringTag = key(OBFUSCATED);
const CLICK_EVENT_K: StringTag = key(CLICK_EVENT);
const HOVER_EVENT_K: StringTag = key(HOVER_EVENT);
const INSERTION_K: StringTag = key(INSERTION);
const FONT_K: StringTag = key(FONT);

#[derive(Clone)]
pub struct TextComponent {
    pub content: Content,
    pub style: Style,
    pub siblings: Vec<Self>,
}

#[derive(Clone)]
pub enum Content {
    Literal {
        content: StringTag,
    },
    Translatable {
        key: StringTag,
        fallback: Option<StringTag>,
        args: Vec<TextComponent>,
    },
    Score {
        name: StringTag,
        objective: StringTag,
    },
    Selector {
        pattern: StringTag,
        separator: Option<Box<TextComponent>>,
    },
    Keybind {
        keybind: StringTag,
    },
    Nbt {
        nbt_path: StringTag,
        interpret: bool,
        separator: Option<Box<TextComponent>>,
        content: NbtContent,
    },
    Object {
        content: ObjectInfo,
    },
}

#[derive(Clone, Copy)]
pub enum Color {
    NS(TextColorNamed, ShadowColor),
    RS(TextColorRgb, ShadowColor),
    NV(TextColorNamed),
    RV(TextColorRgb),
    VS(ShadowColor),
    VV,
}

impl Color {
    pub const fn with_shadow(self, shadow_color: ShadowColor) -> Self {
        match self {
            Self::NS(color, _) => Self::NS(color, shadow_color),
            Self::RS(color, _) => Self::RS(color, shadow_color),
            Self::NV(color) => Self::NS(color, shadow_color),
            Self::RV(color) => Self::RS(color, shadow_color),
            Self::VS(_) => Self::VS(shadow_color),
            Self::VV => Self::VS(shadow_color),
        }
    }

    pub const fn with_rgb(self, color: TextColorRgb) -> Self {
        match self {
            Self::NS(_, shadow_color) => Self::RS(color, shadow_color),
            Self::RS(_, shadow_color) => Self::RS(color, shadow_color),
            Self::NV(_) => Self::RV(color),
            Self::RV(_) => Self::RV(color),
            Self::VS(shadow_color) => Self::RS(color, shadow_color),
            Self::VV => Self::RV(color),
        }
    }

    pub const fn with_named(self, color: TextColorNamed) -> Self {
        match self {
            Self::NS(_, shadow_color) => Self::NS(color, shadow_color),
            Self::RS(_, shadow_color) => Self::NS(color, shadow_color),
            Self::NV(_) => Self::NV(color),
            Self::RV(_) => Self::NV(color),
            Self::VS(shadow_color) => Self::NS(color, shadow_color),
            Self::VV => Self::NV(color),
        }
    }

    pub const fn with_color(self, color: TextColor) -> Self {
        match color {
            TextColor::Named(color_named) => self.with_named(color_named),
            TextColor::Rgb(color_rgb) => self.with_rgb(color_rgb),
        }
    }

    pub const fn shadow_color(self) -> Option<ShadowColor> {
        match self {
            Self::NS(_, shadow_color) => Some(shadow_color),
            Self::RS(_, shadow_color) => Some(shadow_color),
            Self::NV(_) => None,
            Self::RV(_) => None,
            Self::VS(shadow_color) => Some(shadow_color),
            Self::VV => None,
        }
    }

    pub const fn rgb(self) -> Option<TextColorRgb> {
        match self {
            Self::NS(_, _) => None,
            Self::RS(text_color_rgb, _) => Some(text_color_rgb),
            Self::NV(_) => None,
            Self::RV(text_color_rgb) => Some(text_color_rgb),
            Self::VS(_) => None,
            Self::VV => None,
        }
    }

    pub const fn named(self) -> Option<TextColorNamed> {
        match self {
            Self::NS(text_color_named, _) => Some(text_color_named),
            Self::RS(_, _) => None,
            Self::NV(text_color_named) => Some(text_color_named),
            Self::RV(_) => None,
            Self::VS(_) => None,
            Self::VV => None,
        }
    }

    pub const fn color(self) -> Option<TextColor> {
        match self {
            Self::NS(text_color_named, _) => Some(TextColor::Named(text_color_named)),
            Self::RS(text_color_rgb, _) => Some(TextColor::Rgb(text_color_rgb)),
            Self::NV(text_color_named) => Some(TextColor::Named(text_color_named)),
            Self::RV(text_color_rgb) => Some(TextColor::Rgb(text_color_rgb)),
            Self::VS(_) => None,
            Self::VV => None,
        }
    }

    pub const fn clear_color(self) -> Self {
        match self {
            Self::NS(_, shadow_color) => Self::VS(shadow_color),
            Self::RS(_, shadow_color) => Self::VS(shadow_color),
            Self::NV(_) => Self::VV,
            Self::RV(_) => Self::VV,
            Self::VS(shadow_color) => Self::VS(shadow_color),
            Self::VV => Self::VV,
        }
    }

    pub const fn clear_shadow(self) -> Self {
        match self {
            Self::NS(text_color_named, _) => Self::NV(text_color_named),
            Self::RS(text_color_rgb, _) => Self::RV(text_color_rgb),
            Self::NV(text_color_named) => Self::NV(text_color_named),
            Self::RV(text_color_rgb) => Self::RV(text_color_rgb),
            Self::VS(_) => Self::VV,
            Self::VV => Self::VV,
        }
    }

    pub const fn clear_rgb(self) -> Self {
        match self {
            Self::NS(text_color_named, shadow_color) => Self::NS(text_color_named, shadow_color),
            Self::RS(_, shadow_color) => Self::VS(shadow_color),
            Self::NV(text_color_named) => Self::NV(text_color_named),
            Self::RV(_) => Self::VV,
            Self::VS(shadow_color) => Self::VS(shadow_color),
            Self::VV => Self::VV,
        }
    }

    pub const fn clear_named(self) -> Self {
        match self {
            Self::NS(_, shadow_color) => Self::VS(shadow_color),
            Self::RS(text_color_rgb, shadow_color) => Self::RS(text_color_rgb, shadow_color),
            Self::NV(_) => Self::VV,
            Self::RV(text_color_rgb) => Self::RV(text_color_rgb),
            Self::VS(shadow_color) => Self::VS(shadow_color),
            Self::VV => Self::VV,
        }
    }

    pub const fn is_none(self) -> bool {
        matches!(self, Self::VV)
    }

    pub const fn is_some(self) -> bool {
        !self.is_none()
    }
}

#[derive(Clone)]
pub struct Style {
    pub color: Color,
    pub decorations: DecorationMap,
    pub click_event: Option<ClickEvent>,
    pub hover_event: Option<HoverEvent>,
    pub insertion: Option<StringTag>,
    pub font: Option<StringTag>,
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
            color: Color::VV,
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
            && self.decorations.is_empty()
            && self.click_event.is_none()
            && self.hover_event.is_none()
            && self.insertion.is_none()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ShadowColor {
    pub value: u32,
}

impl ShadowColor {
    pub const NONE: Self = Self { value: 0 };
}

impl Default for ShadowColor {
    fn default() -> Self {
        Self::NONE
    }
}

impl Serialize for ShadowColor {
    fn serialize(&self) -> Tag {
        Tag::Int(self.value as i32)
    }
}

impl Deserialize for ShadowColor {
    fn deserialize(nbt: Tag) -> Result<Self, Error> {
        match nbt {
            Tag::Int(value) => Ok(Self {
                value: value as u32,
            }),
            Tag::Byte(value) => Ok(Self {
                value: value as u32,
            }),
            Tag::Long(value) => Ok(Self {
                value: value as u32,
            }),
            Tag::Float(value) => Ok(Self {
                value: value as u32,
            }),
            Tag::Double(value) => Ok(Self {
                value: value as u32,
            }),
            Tag::List(ListTag::Float(value)) => match value[..] {
                [r, g, b, a] => Ok(Self {
                    value: (f32_to_u8(r) as u32)
                        | (f32_to_u8(g) as u32) << 8
                        | (f32_to_u8(b) as u32) << 16
                        | (f32_to_u8(a) as u32) << 24,
                }),
                _ => Err(Error),
            },
            Tag::List(ListTag::Double(value)) => match value[..] {
                [r, g, b, a] => Ok(Self {
                    value: (f64_to_u8(r) as u32)
                        | (f64_to_u8(g) as u32) << 8
                        | (f64_to_u8(b) as u32) << 16
                        | (f64_to_u8(a) as u32) << 24,
                }),
                _ => Err(Error),
            },
            _ => Err(Error),
        }
    }
}

const DEFAULT_ATLAS: Identifier = Identifier::new("blocks").unwrap();

#[derive(Clone)]
pub enum ObjectInfo {
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
    Block { pos: StringTag },
    Entity { selector: StringTag },
    Storage { storage: Identifier },
}

impl Serialize for TextComponent {
    fn serialize(&self) -> Tag {
        let mut nbt = CompoundTag::new();
        let ty = match &self.content {
            Content::Literal { .. } => TEXT_K,
            Content::Translatable { .. } => TRANSLATE_K,
            Content::Score { .. } => SCORE_K,
            Content::Selector { .. } => SELECTOR_K,
            Content::Keybind { .. } => KEYBIND_K,
            Content::Nbt { .. } => NBT_PATH_K,
            Content::Object { .. } => OBJECT_TYPE_K,
        };
        nbt.push(TYPE_K, Tag::String(ty));
        match &self.content {
            Content::Literal { content } => {
                nbt.push(TEXT_K, content.serialize());
            }
            Content::Translatable {
                key,
                fallback,
                args,
            } => {
                nbt.push(TRANSLATE_K, key.serialize());
                if let Some(f) = fallback {
                    nbt.push(TRANSLATE_FALLBACK_K, f.serialize());
                }
                if !args.is_empty() {
                    let mut vec = Vec::with_capacity(args.len());
                    for arg in args {
                        vec.push(match arg.serialize() {
                            Tag::Compound(x) => x,
                            _ => unsafe { core::hint::unreachable_unchecked() },
                        });
                    }
                    nbt.push(TRANSLATE_WITH_K, Tag::List(ListTag::Compound(vec)));
                }
            }
            Content::Score { name, objective } => {
                let score = vec![
                    (SCORE_NAME_K, name.serialize()),
                    (SCORE_OBJECTIVE_K, objective.serialize()),
                ];
                nbt.push(SCORE_K, Tag::Compound(CompoundTag::from(score)));
            }
            Content::Selector { pattern, separator } => {
                nbt.push(SELECTOR_K, pattern.serialize());
                if let Some(s) = separator {
                    nbt.push(SEPARATOR_K, s.serialize());
                }
            }
            Content::Keybind { keybind } => {
                nbt.push(KEYBIND_K, keybind.serialize());
            }
            Content::Nbt {
                nbt_path,
                interpret,
                separator,
                content,
            } => {
                nbt.push(NBT_PATH_K, nbt_path.serialize());
                if *interpret {
                    nbt.push(NBT_INTERPRET_K, interpret.serialize());
                }
                if let Some(s) = separator {
                    nbt.push(SEPARATOR_K, s.serialize());
                }
                let source = match content {
                    NbtContent::Block { .. } => NBT_BLOCK_K,
                    NbtContent::Entity { .. } => NBT_ENTITY_K,
                    NbtContent::Storage { .. } => NBT_STORAGE_K,
                };
                nbt.push(NBT_SOURCE_K, Tag::String(source));
                match content {
                    NbtContent::Block { pos } => nbt.push(NBT_BLOCK_K, pos.serialize()),
                    NbtContent::Entity { selector } => nbt.push(NBT_ENTITY_K, selector.serialize()),
                    NbtContent::Storage { storage } => nbt.push(NBT_STORAGE_K, storage.serialize()),
                }
            }
            Content::Object { content } => match content {
                ObjectInfo::Atlas { atlas, sprite } => {
                    nbt.push(OBJECT_TYPE_K, Tag::String(OBJECT_ATLAS_K));
                    nbt.push(OBJECT_ATLAS_K, atlas.serialize());
                    nbt.push(OBJECT_SPRITE_K, sprite.serialize());
                }
                ObjectInfo::Player { player, hat } => {
                    nbt.push(OBJECT_TYPE_K, Tag::String(OBJECT_PLAYER_K));
                    nbt.push(OBJECT_PLAYER_K, player.serialize());
                    if !hat {
                        nbt.push(OBJECT_HAT_K, hat.serialize());
                    }
                }
            },
        }
        if let Some(text_color) = self.style.color.color() {
            nbt.push(COLOR_K, text_color.serialize());
        }
        if let Some(shadow_color) = self.style.color.shadow_color() {
            nbt.push(SHADOW_COLOR_K, shadow_color.serialize());
        }
        if let Some(bold) = self.style.decorations.bold() {
            nbt.push(BOLD_K, bold.serialize());
        }
        if let Some(italic) = self.style.decorations.italic() {
            nbt.push(ITALIC_K, italic.serialize());
        }
        if let Some(underlined) = self.style.decorations.underlined() {
            nbt.push(UNDERLINED_K, underlined.serialize());
        }
        if let Some(strikethrough) = self.style.decorations.strikethrough() {
            nbt.push(STRIKETHROUGH_K, strikethrough.serialize());
        }
        if let Some(obfuscated) = self.style.decorations.obfuscated() {
            nbt.push(OBFUSCATED_K, obfuscated.serialize());
        }
        if let Some(click_event) = self.style.click_event.as_ref() {
            nbt.push(CLICK_EVENT_K, click_event.serialize());
        }
        if let Some(hover_event) = self.style.hover_event.as_ref() {
            nbt.push(HOVER_EVENT_K, hover_event.serialize());
        }
        if let Some(insertion) = self.style.insertion.as_ref() {
            nbt.push(INSERTION_K, insertion.serialize());
        }
        if let Some(font) = self.style.font.as_ref() {
            nbt.push(FONT_K, font.serialize());
        }
        Tag::Compound(nbt)
    }
}

impl Deserialize for TextComponent {
    fn deserialize(nbt: Tag) -> Result<Self, Error> {
        let c = match nbt {
            Tag::String(literal) => {
                return Ok(Self {
                    content: Content::Literal { content: literal },
                    style: Style::new(),
                    siblings: Vec::new(),
                });
            }
            Tag::Compound(c) => c,
            Tag::List(ListTag::Compound(c)) => {
                let mut iter = c.into_iter();
                let first = match iter.next() {
                    Some(x) => x,
                    None => {
                        return Ok(Self {
                            content: Content::Literal {
                                content: StringTag::new(),
                            },
                            style: Style::new(),
                            siblings: Vec::new(),
                        });
                    }
                };
                let mut c = Self::deserialize(Tag::Compound(first))?;
                c.siblings.reserve_exact(capacity_fix(iter.len()));
                for sibling in iter {
                    c.siblings.push(Self::deserialize(Tag::Compound(sibling))?);
                }
                return Ok(c);
            }
            Tag::List(ListTag::None) => {
                return Ok(Self {
                    content: Content::Literal {
                        content: StringTag::new(),
                    },
                    style: Style::new(),
                    siblings: Vec::new(),
                });
            }
            _ => return Err(Error),
        };
        let mut content = Content::Literal {
            content: StringTag::new(),
        };
        let mut style = Style::new();
        let mut siblings = Vec::new();
        let mut separator: Option<Box<Self>> = None;
        for (k, v) in c {
            match &*k {
                TYPE => {
                    let _ty = StringTag::deserialize(v)?;
                }
                TEXT => {
                    let text = StringTag::deserialize(v)?;
                    content = Content::Literal { content: text };
                }
                TRANSLATE => {
                    let t = StringTag::deserialize(v)?;
                    match &mut content {
                        Content::Translatable { key, .. } => *key = t,
                        _ => {
                            content = Content::Translatable {
                                key: t,
                                fallback: None,
                                args: Vec::new(),
                            };
                        }
                    }
                }
                TRANSLATE_FALLBACK => {
                    let fb = StringTag::deserialize(v)?;
                    match &mut content {
                        Content::Translatable { fallback, .. } => *fallback = Some(fb),
                        _ => {
                            content = Content::Translatable {
                                key: StringTag::new(),
                                fallback: Some(fb),
                                args: Vec::new(),
                            };
                        }
                    }
                }
                TRANSLATE_WITH => {
                    let w = match v {
                        Tag::List(ListTag::Compound(l)) => {
                            let mut vec = Vec::with_capacity(capacity_fix(l.len()));
                            for ele in l {
                                vec.push(Self::deserialize(Tag::Compound(ele))?);
                            }
                            vec
                        }
                        Tag::List(ListTag::None) => Vec::new(),
                        _ => return Err(Error),
                    };
                    match &mut content {
                        Content::Translatable { args, .. } => *args = w,
                        _ => {
                            content = Content::Translatable {
                                key: StringTag::new(),
                                fallback: None,
                                args: w,
                            };
                        }
                    }
                }
                SCORE => {
                    let score = match v {
                        Tag::Compound(c) => c,
                        _ => return Err(Error),
                    };
                    let mut name = StringTag::new();
                    let mut objective = StringTag::new();
                    for (sk, sv) in score {
                        match &*sk {
                            SCORE_NAME => {
                                name = StringTag::deserialize(sv)?;
                            }
                            SCORE_OBJECTIVE => {
                                objective = StringTag::deserialize(sv)?;
                            }
                            _ => return Err(Error),
                        }
                    }
                    content = Content::Score { name, objective };
                }
                SELECTOR => {
                    let pattern = StringTag::deserialize(v)?;
                    content = Content::Selector {
                        pattern,
                        separator: None,
                    };
                }
                SEPARATOR => {
                    separator = Some(Box::new(Self::deserialize(v)?));
                }
                KEYBIND => {
                    let keybind = StringTag::deserialize(v)?;
                    content = Content::Keybind { keybind };
                }
                NBT_PATH => {
                    let path = StringTag::deserialize(v)?;
                    match &mut content {
                        Content::Nbt { nbt_path, .. } => *nbt_path = path,
                        _ => {
                            content = Content::Nbt {
                                nbt_path: path,
                                interpret: false,
                                separator: None,
                                content: NbtContent::Block {
                                    pos: StringTag::new(),
                                },
                            }
                        }
                    }
                }
                NBT_INTERPRET => {
                    let i = bool::deserialize(v)?;
                    match &mut content {
                        Content::Nbt { interpret, .. } => *interpret = i,
                        _ => {
                            content = Content::Nbt {
                                nbt_path: StringTag::new(),
                                interpret: i,
                                separator: None,
                                content: NbtContent::Block {
                                    pos: StringTag::new(),
                                },
                            }
                        }
                    }
                }
                NBT_SOURCE => {
                    let _source = StringTag::deserialize(v)?;
                }
                NBT_BLOCK => {
                    let block = NbtContent::Block {
                        pos: StringTag::deserialize(v)?,
                    };
                    match &mut content {
                        Content::Nbt { content, .. } => *content = block,
                        _ => {
                            content = Content::Nbt {
                                nbt_path: StringTag::new(),
                                interpret: false,
                                separator: None,
                                content: block,
                            }
                        }
                    }
                }
                NBT_ENTITY => {
                    let entity = NbtContent::Entity {
                        selector: StringTag::deserialize(v)?,
                    };
                    match &mut content {
                        Content::Nbt { content, .. } => *content = entity,
                        _ => {
                            content = Content::Nbt {
                                nbt_path: StringTag::new(),
                                interpret: false,
                                separator: None,
                                content: entity,
                            }
                        }
                    }
                }
                NBT_STORAGE => {
                    let storage = NbtContent::Storage {
                        storage: Identifier::deserialize(v)?,
                    };
                    match &mut content {
                        Content::Nbt { content, .. } => *content = storage,
                        _ => {
                            content = Content::Nbt {
                                nbt_path: StringTag::new(),
                                interpret: false,
                                separator: None,
                                content: storage,
                            }
                        }
                    }
                }
                OBJECT_TYPE => {
                    let _object = StringTag::deserialize(v)?;
                }
                OBJECT_ATLAS => {
                    let atlas1 = Identifier::deserialize(v)?;
                    match &mut content {
                        Content::Object {
                            content: ObjectInfo::Atlas { atlas, .. },
                        } => *atlas = atlas1,
                        _ => {
                            content = Content::Object {
                                content: ObjectInfo::Atlas {
                                    atlas: atlas1,
                                    sprite: Identifier::new("").unwrap(),
                                },
                            };
                        }
                    }
                }
                OBJECT_SPRITE => {
                    let sprite1 = Identifier::deserialize(v)?;
                    match &mut content {
                        Content::Object {
                            content: ObjectInfo::Atlas { sprite, .. },
                        } => *sprite = sprite1,
                        _ => {
                            content = Content::Object {
                                content: ObjectInfo::Atlas {
                                    atlas: DEFAULT_ATLAS,
                                    sprite: sprite1,
                                },
                            };
                        }
                    }
                }
                OBJECT_PLAYER => {
                    let pl = ResolvableProfile::deserialize(v)?;
                    match &mut content {
                        Content::Object {
                            content: ObjectInfo::Player { player, .. },
                        } => **player = pl,
                        _ => {
                            content = Content::Object {
                                content: ObjectInfo::Player {
                                    player: Box::new(pl),
                                    hat: true,
                                },
                            }
                        }
                    }
                }
                OBJECT_HAT => {
                    let hat1 = bool::deserialize(v)?;
                    match &mut content {
                        Content::Object {
                            content: ObjectInfo::Player { hat, .. },
                        } => *hat = hat1,
                        _ => {
                            content = Content::Object {
                                content: ObjectInfo::Player {
                                    hat: hat1,
                                    player: Box::new(ResolvableProfile {
                                        name: None,
                                        id: None,
                                        properties: PropertyMap(Vec::new()),
                                        skin_patch: PlayerSkinPatch {
                                            texture: None,
                                            cape: None,
                                            elytra: None,
                                            model: None,
                                        },
                                    }),
                                },
                            };
                        }
                    }
                }
                EXTRA => match v {
                    Tag::List(ListTag::Compound(l)) => {
                        siblings.reserve_exact(capacity_fix(l.len()));
                        for nbt in l {
                            let c = Self::deserialize(Tag::Compound(nbt))?;
                            siblings.push(c);
                        }
                    }
                    Tag::List(ListTag::None) => {}
                    _ => return Err(Error),
                },
                COLOR => {
                    let color = TextColor::deserialize(v)?;
                    style.color = style.color.with_color(color);
                }
                SHADOW_COLOR => {
                    let shadow_color = ShadowColor::deserialize(v)?;
                    style.color = style.color.with_shadow(shadow_color);
                }
                BOLD => {
                    style.decorations.with_bold(Some(bool::deserialize(v)?));
                }
                ITALIC => {
                    style.decorations.with_italic(Some(bool::deserialize(v)?));
                }
                UNDERLINED => {
                    style
                        .decorations
                        .with_underlined(Some(bool::deserialize(v)?));
                }
                STRIKETHROUGH => {
                    style
                        .decorations
                        .with_strikethrough(Some(bool::deserialize(v)?));
                }
                OBFUSCATED => {
                    style
                        .decorations
                        .with_obfuscated(Some(bool::deserialize(v)?));
                }
                CLICK_EVENT => {
                    let click_event = ClickEvent::deserialize(v)?;
                    style.click_event = Some(click_event);
                }
                HOVER_EVENT => {
                    let hover_event = HoverEvent::deserialize(v)?;
                    style.hover_event = Some(hover_event);
                }
                INSERTION => {
                    let insertion = StringTag::deserialize(v)?;
                    style.insertion = Some(insertion);
                }
                FONT => {
                    let font = StringTag::deserialize(v)?;
                    style.font = Some(font);
                }
                _ => return Err(Error),
            }
        }
        if let Some(sep) = separator {
            match &mut content {
                Content::Selector { separator: s, .. } => {
                    *s = Some(sep);
                }
                Content::Nbt { separator: s, .. } => {
                    *s = Some(sep);
                }
                _ => return Err(Error),
            }
        }
        Ok(Self {
            content,
            style,
            siblings,
        })
    }
}
