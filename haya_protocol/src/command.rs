use crate::chat::MessageSignature;
use haya_collection::List;
use haya_ident::{Ident, ResourceKey};
use minecraft_data::command_argument_type;
use mser::{Error, Read, Reader, Utf8, V21, Write, Writer};
use mser_macro::{Deserialize, Serialize};

const MASK_NODE: u8 = 0x03;
const FLAG_ROOT: u8 = 0x00;
const FLAG_LITERAL: u8 = 0x01;
const FLAG_ARGUMENT: u8 = 0x02;
const FLAG_EXECUTABLE: u8 = 0x04;
const FLAG_REDIRECT: u8 = 0x08;
const FLAG_SUGGESTION: u8 = 0x10;
const FLAG_RESTRICTED: u8 = 0x20;

const FLAG_SINGLE: u8 = 1;
const FLAG_PLAYERS_ONLY: u8 = 2;
const FLAG_MULTIPLE: u8 = 1;
const NUMBER_FLAG_MIN: u8 = 1;
const NUMBER_FLAG_MAX: u8 = 2;

#[derive(Clone, Debug)]
pub enum CommandNode<'a> {
    Root {
        children: List<'a, V21>,
        redirect: Option<V21>,
    },
    Literal {
        children: List<'a, V21>,
        redirect: Option<V21>,
        name: Utf8<'a>,
        executable: bool,
        restricted: bool,
    },
    Argument {
        children: List<'a, V21>,
        redirect: Option<V21>,
        name: Utf8<'a>,
        executable: bool,
        restricted: bool,
        arg_type: CommandArgument<'a>,
        suggestions: Option<Suggestions>,
    },
}

#[derive(Clone, Debug)]
pub struct Float {
    pub min: f32,
    pub max: f32,
}

#[derive(Clone, Debug)]
pub struct Double {
    pub min: f64,
    pub max: f64,
}

#[derive(Clone, Debug)]
pub struct Integer {
    pub min: i32,
    pub max: i32,
}

#[derive(Clone, Debug)]
pub struct Long {
    pub min: i64,
    pub max: i64,
}

#[derive(Clone, Debug)]
pub struct Entity {
    pub single: bool,
    pub players_only: bool,
}

#[derive(Clone, Debug)]
pub struct ScoreHolder {
    pub multiple: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Time {
    pub min: i32,
}

impl Write for Float {
    unsafe fn write(&self, w: &mut Writer) {
        unsafe {
            let Self { min, max } = *self;
            let mut flags = 0u8;
            if min != f32::MIN {
                flags |= NUMBER_FLAG_MIN;
            }
            if max != f32::MAX {
                flags |= NUMBER_FLAG_MAX;
            }
            flags.write(w);
            if min != f32::MIN {
                min.write(w);
            }
            if max != f32::MAX {
                max.write(w);
            }
        }
    }
    fn len_s(&self) -> usize {
        let Self { min, max } = *self;
        let mut w = 1;
        if min != f32::MIN {
            w += min.len_s();
        }
        if max != f32::MAX {
            w += max.len_s();
        }
        w
    }
}

impl Write for Double {
    unsafe fn write(&self, w: &mut Writer) {
        unsafe {
            let Self { min, max } = *self;
            let mut flags = 0u8;
            if min != f64::MIN {
                flags |= NUMBER_FLAG_MIN;
            }
            if max != f64::MAX {
                flags |= NUMBER_FLAG_MAX;
            }
            flags.write(w);
            if min != f64::MIN {
                min.write(w);
            }
            if max != f64::MAX {
                max.write(w);
            }
        }
    }

    fn len_s(&self) -> usize {
        let Self { min, max } = *self;
        let mut w = 1;
        if min != f64::MIN {
            w += min.len_s();
        }
        if max != f64::MAX {
            w += max.len_s();
        }
        w
    }
}

impl Write for Integer {
    unsafe fn write(&self, w: &mut Writer) {
        unsafe {
            let Self { min, max } = *self;
            let mut flags = 0u8;
            if min != i32::MIN {
                flags |= NUMBER_FLAG_MIN;
            }
            if max != i32::MAX {
                flags |= NUMBER_FLAG_MAX;
            }
            flags.write(w);
            if min != i32::MIN {
                min.write(w);
            }
            if max != i32::MAX {
                max.write(w);
            }
        }
    }

    fn len_s(&self) -> usize {
        let Self { min, max } = *self;
        let mut w = 1;
        if min != i32::MIN {
            w += min.len_s();
        }
        if max != i32::MAX {
            w += max.len_s();
        }
        w
    }
}

impl Write for Long {
    unsafe fn write(&self, w: &mut Writer) {
        unsafe {
            let Self { min, max } = *self;
            let mut flags = 0u8;
            if min != i64::MIN {
                flags |= NUMBER_FLAG_MIN;
            }
            if max != i64::MAX {
                flags |= NUMBER_FLAG_MAX;
            }
            flags.write(w);
            if min != i64::MIN {
                min.write(w);
            }
            if max != i64::MAX {
                max.write(w);
            }
        }
    }

    fn len_s(&self) -> usize {
        let Self { min, max } = *self;
        let mut w = 1;
        if min != i64::MIN {
            w += min.len_s();
        }
        if max != i64::MAX {
            w += max.len_s();
        }
        w
    }
}

impl Write for Entity {
    unsafe fn write(&self, w: &mut Writer) {
        unsafe {
            let Self {
                single,
                players_only,
            } = *self;
            let mut flags = 0u8;
            if single {
                flags |= FLAG_SINGLE;
            }
            if players_only {
                flags |= FLAG_PLAYERS_ONLY;
            }
            flags.write(w);
        }
    }

    fn len_s(&self) -> usize {
        1
    }
}

impl Write for ScoreHolder {
    unsafe fn write(&self, w: &mut Writer) {
        unsafe {
            let Self { multiple } = *self;
            let mut flags = 0u8;
            if multiple {
                flags |= FLAG_MULTIPLE;
            }
            flags.write(w);
        }
    }

    fn len_s(&self) -> usize {
        1
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[mser(header = command_argument_type)]
pub enum CommandArgument<'a> {
    Bool,
    Float(Float),
    Double(Double),
    Integer(Integer),
    Long(Long),
    String(StringType),
    Entity(Entity),
    GameProfile,
    BlockPos,
    ColumnPos,
    Vec3,
    Vec2,
    BlockState,
    BlockPredicate,
    ItemStack,
    ItemPredicate,
    TeamColor,
    HexColor,
    Component,
    Style,
    Message,
    NbtCompoundTag,
    NbtTag,
    NbtPath,
    Objective,
    ObjectiveCriteria,
    Operation,
    Particle,
    Angle,
    Rotation,
    ScoreboardSlot,
    ScoreHolder(ScoreHolder),
    Swizzle,
    Team,
    ItemSlot,
    ItemSlots,
    ResourceLocation,
    Function,
    EntityAnchor,
    IntRange,
    FloatRange,
    Dimension,
    Gamemode,
    Time(Time),
    ResourceOrTag(ResourceKey<'a>),
    ResourceOrTagKey(ResourceKey<'a>),
    Resource(ResourceKey<'a>),
    ResourceKey(ResourceKey<'a>),
    ResourceSelector(ResourceKey<'a>),
    TemplateMirror,
    TemplateRotation,
    Heightmap,
    LootTable,
    LootPredicate,
    LootModifier,
    ContextFloatProvider,
    ContextIntProvider,
    SlotSource,
    Dialog,
    Feature,
    SwingAnimation,
    Uuid,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum StringType {
    SingleWord,
    QuotablePhrase,
    GreedyPhrase,
}

impl<'a> Read<'a> for Float {
    fn read(buf: &mut Reader<'a>) -> Result<Self, Error> {
        let flags = u8::read(buf)?;
        let min = if flags & NUMBER_FLAG_MIN != 0 {
            f32::read(buf)?
        } else {
            f32::MIN
        };
        let max = if flags & NUMBER_FLAG_MAX != 0 {
            f32::read(buf)?
        } else {
            f32::MAX
        };
        Ok(Self { min, max })
    }
}

impl<'a> Read<'a> for Double {
    fn read(buf: &mut Reader<'a>) -> Result<Self, Error> {
        let flags = u8::read(buf)?;
        let min = if flags & NUMBER_FLAG_MIN != 0 {
            f64::read(buf)?
        } else {
            f64::MIN
        };
        let max = if flags & NUMBER_FLAG_MAX != 0 {
            f64::read(buf)?
        } else {
            f64::MAX
        };
        Ok(Self { min, max })
    }
}

impl<'a> Read<'a> for Integer {
    fn read(buf: &mut Reader<'a>) -> Result<Self, Error> {
        let flags = u8::read(buf)?;
        let min = if flags & NUMBER_FLAG_MIN != 0 {
            i32::read(buf)?
        } else {
            i32::MIN
        };
        let max = if flags & NUMBER_FLAG_MAX != 0 {
            i32::read(buf)?
        } else {
            i32::MAX
        };
        Ok(Self { min, max })
    }
}

impl<'a> Read<'a> for Long {
    fn read(buf: &mut Reader<'a>) -> Result<Self, Error> {
        let flags = u8::read(buf)?;
        let min = if flags & NUMBER_FLAG_MIN != 0 {
            i64::read(buf)?
        } else {
            i64::MIN
        };
        let max = if flags & NUMBER_FLAG_MAX != 0 {
            i64::read(buf)?
        } else {
            i64::MAX
        };
        Ok(Self { min, max })
    }
}

impl<'a> Read<'a> for Entity {
    fn read(buf: &mut Reader<'a>) -> Result<Self, Error> {
        let flags = u8::read(buf)?;
        let single = flags & FLAG_SINGLE != 0;
        let players_only = flags & FLAG_PLAYERS_ONLY != 0;
        Ok(Self {
            single,
            players_only,
        })
    }
}

impl<'a> Read<'a> for ScoreHolder {
    fn read(buf: &mut Reader<'a>) -> Result<Self, Error> {
        Ok(Self {
            multiple: (u8::read(buf)? & FLAG_MULTIPLE) != 0,
        })
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Suggestions {
    AskServer,
    AvailableSounds,
    SummonableEntities,
}

impl Write for Suggestions {
    #[inline]
    unsafe fn write(&self, w: &mut Writer) {
        unsafe {
            match self {
                Self::AskServer => {
                    w.write(b"\x20minecraft:ask_server");
                }
                Self::AvailableSounds => {
                    w.write(b"\x26minecraft:available_sounds");
                }
                Self::SummonableEntities => {
                    w.write(b"\x29minecraft:summonable_entities");
                }
            }
        }
    }

    #[inline]
    fn len_s(&self) -> usize {
        match self {
            Self::AskServer => 21,
            Self::AvailableSounds => 27,
            Self::SummonableEntities => 30,
        }
    }
}

impl<'a> Read<'a> for Suggestions {
    fn read(buf: &mut Reader<'a>) -> Result<Self, Error> {
        let ident = Ident::read(buf)?;
        if ident.namespace().is_none() {
            match ident.path() {
                "ask_server" => Ok(Self::AskServer),
                "available_sounds" => Ok(Self::AvailableSounds),
                "summonable_entities" => Ok(Self::SummonableEntities),
                _ => Ok(Self::AskServer),
            }
        } else {
            Ok(Self::AskServer)
        }
    }
}

impl Write for CommandNode<'_> {
    unsafe fn write(&self, w: &mut Writer) {
        unsafe {
            let (flags, children, redirect) = match self {
                Self::Root { children, redirect } => {
                    let mut flags = FLAG_ROOT;
                    if redirect.is_some() {
                        flags |= FLAG_REDIRECT;
                    }
                    (flags, children, *redirect)
                }
                Self::Literal {
                    children,
                    redirect,
                    name: _,
                    executable,
                    restricted,
                } => {
                    let mut flags = FLAG_LITERAL;
                    if *executable {
                        flags |= FLAG_EXECUTABLE;
                    }
                    if redirect.is_some() {
                        flags |= FLAG_REDIRECT;
                    }
                    if *restricted {
                        flags |= FLAG_RESTRICTED;
                    }
                    (flags, children, *redirect)
                }
                Self::Argument {
                    children,
                    redirect,
                    name: _,
                    executable,
                    restricted,
                    arg_type: _,
                    suggestions,
                } => {
                    let mut flags = FLAG_ARGUMENT;
                    if *executable {
                        flags |= FLAG_EXECUTABLE;
                    }
                    if redirect.is_some() {
                        flags |= FLAG_REDIRECT;
                    }
                    if *restricted {
                        flags |= FLAG_RESTRICTED;
                    }
                    if suggestions.is_some() {
                        flags |= FLAG_SUGGESTION;
                    }
                    (flags, children, *redirect)
                }
            };
            w.write_byte(flags);
            children.write(w);
            if let Some(r) = redirect {
                r.write(w);
            }
            match self {
                Self::Root { .. } => (),
                Self::Literal { name, .. } => {
                    name.write(w);
                }
                Self::Argument {
                    name,
                    arg_type,
                    suggestions,
                    ..
                } => {
                    name.write(w);
                    arg_type.write(w);
                    if let Some(s) = suggestions {
                        s.write(w);
                    }
                }
            };
        }
    }

    fn len_s(&self) -> usize {
        let (children, redirect) = match self {
            Self::Root { children, redirect } => (children, *redirect),
            Self::Literal {
                children, redirect, ..
            } => (children, *redirect),
            Self::Argument {
                children, redirect, ..
            } => (children, *redirect),
        };
        let mut l = 1 + children.len_s();
        if let Some(r) = redirect {
            l += r.len_s();
        }
        match self {
            Self::Root { .. } => l,
            Self::Literal { name, .. } => l + name.len_s(),
            Self::Argument {
                name,
                arg_type,
                suggestions,
                ..
            } => {
                l += name.len_s();
                l += arg_type.len_s();
                if let Some(s) = suggestions {
                    l += s.len_s();
                }
                l
            }
        }
    }
}

impl<'a> Read<'a> for CommandNode<'a> {
    fn read(buf: &mut Reader<'a>) -> Result<Self, Error> {
        let flags = u8::read(buf)?;
        let children = List::<'a, V21>::read(buf)?;
        let redirect = if (flags & FLAG_REDIRECT) != 0 {
            Some(V21::read(buf)?)
        } else {
            None
        };
        Ok(match flags & MASK_NODE {
            FLAG_LITERAL => Self::Literal {
                children,
                redirect,
                name: Utf8::read(buf)?,
                executable: (flags & FLAG_EXECUTABLE) != 0,
                restricted: (flags & FLAG_RESTRICTED) != 0,
            },
            FLAG_ARGUMENT => Self::Argument {
                children,
                redirect,
                name: Utf8::read(buf)?,
                executable: (flags & FLAG_EXECUTABLE) != 0,
                restricted: (flags & FLAG_RESTRICTED) != 0,
                arg_type: CommandArgument::read(buf)?,
                suggestions: if (flags & FLAG_SUGGESTION) != 0 {
                    Some(Suggestions::read(buf)?)
                } else {
                    None
                },
            },
            _ => Self::Root { children, redirect },
        })
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ArgumentSignatures<'a>(List<'a, ArgumentSignature<'a>, 8>);

#[derive(Clone, Serialize, Deserialize)]
pub struct ArgumentSignature<'a> {
    pub name: Utf8<'a, 16>,
    pub signature: MessageSignature<'a>,
}
