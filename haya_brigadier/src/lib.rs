#![no_std]

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
        children: &'a [u32],
    },
    Literal {
        children: &'a [u32],
        redirect: Option<u32>,
        name: Utf8<'a>,
        executable: bool,
        restricted: bool,
    },
    Argument {
        children: &'a [u32],
        redirect: Option<u32>,
        name: Utf8<'a>,
        executable: bool,
        restricted: bool,
        arg_type: CommandArgumentType<'a>,
        suggestions: Option<Suggestions>,
    },
}

#[derive(Clone, Debug)]
pub enum CommandArgumentType<'a> {
    Bool,
    Float { min: f32, max: f32 },
    Double { min: f64, max: f64 },
    Integer { min: i32, max: i32 },
    Long { min: i64, max: i64 },
    String { ty: StringType },
    Entity { single: bool, players_only: bool },
    GameProfile,
    BlockPos,
    ColumnPos,
    Vec3,
    Vec2,
    BlockState,
    BlockPredicate,
    ItemStack,
    ItemPredicate,
    Color,
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
    ScoreHolder { multiple: bool },
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
    Time { min: i32 },
    ResourceOrTag { registry_key: ResourceKey<'a> },
    ResourceOrTagKey { registry_key: ResourceKey<'a> },
    Resource { registry_key: ResourceKey<'a> },
    ResourceKey { registry_key: ResourceKey<'a> },
    ResourceSelector { registry_key: ResourceKey<'a> },
    TemplateMirror,
    TemplateRotation,
    Heightmap,
    LootTable,
    LootPredicate,
    LootModifier,
    Dialog,
    Uuid,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[repr(u8)]
pub enum StringType {
    SingleWord,
    QuotablePhrase,
    GreedyPhrase,
}

impl<'a> CommandArgumentType<'a> {
    pub const fn id(&self) -> command_argument_type {
        use command_argument_type::*;
        match self {
            Self::Bool => bool,
            Self::Float { .. } => float,
            Self::Double { .. } => double,
            Self::Integer { .. } => integer,
            Self::Long { .. } => long,
            Self::String { .. } => string,
            Self::Entity { .. } => entity,
            Self::GameProfile => game_profile,
            Self::BlockPos => block_pos,
            Self::ColumnPos => column_pos,
            Self::Vec3 => vec3,
            Self::Vec2 => vec2,
            Self::BlockState => block_state,
            Self::BlockPredicate => block_predicate,
            Self::ItemStack => item_stack,
            Self::ItemPredicate => item_predicate,
            Self::Color => color,
            Self::HexColor => hex_color,
            Self::Component => component,
            Self::Style => style,
            Self::Message => message,
            Self::NbtCompoundTag => nbt_compound_tag,
            Self::NbtTag => nbt_tag,
            Self::NbtPath => nbt_path,
            Self::Objective => objective,
            Self::ObjectiveCriteria => objective_criteria,
            Self::Operation => operation,
            Self::Particle => particle,
            Self::Angle => angle,
            Self::Rotation => rotation,
            Self::ScoreboardSlot => scoreboard_slot,
            Self::ScoreHolder { .. } => score_holder,
            Self::Swizzle => swizzle,
            Self::Team => team,
            Self::ItemSlot => item_slot,
            Self::ItemSlots => item_slots,
            Self::ResourceLocation => resource_location,
            Self::Function => function,
            Self::EntityAnchor => entity_anchor,
            Self::IntRange => int_range,
            Self::FloatRange => float_range,
            Self::Dimension => dimension,
            Self::Gamemode => gamemode,
            Self::Time { .. } => time,
            Self::ResourceOrTag { .. } => resource_or_tag,
            Self::ResourceOrTagKey { .. } => resource_or_tag_key,
            Self::Resource { .. } => resource,
            Self::ResourceKey { .. } => resource_key,
            Self::ResourceSelector { .. } => resource_selector,
            Self::TemplateMirror => template_mirror,
            Self::TemplateRotation => template_rotation,
            Self::Heightmap => heightmap,
            Self::LootTable => loot_table,
            Self::LootPredicate => loot_predicate,
            Self::LootModifier => loot_modifier,
            Self::Dialog => dialog,
            Self::Uuid => uuid,
        }
    }
}

impl Write for CommandArgumentType<'_> {
    unsafe fn write(&self, w: &mut Writer) {
        unsafe {
            self.id().write(w);
            match self {
                Self::Float { min, max } => {
                    let mut flags = 0u8;
                    if *min == f32::MIN {
                        flags |= NUMBER_FLAG_MIN;
                    }
                    if *max == f32::MAX {
                        flags |= NUMBER_FLAG_MAX;
                    }
                    flags.write(w);
                    if *min != f32::MIN {
                        min.write(w);
                    }
                    if *max != f32::MAX {
                        max.write(w);
                    }
                }
                Self::Double { min, max } => {
                    let mut flags = 0u8;
                    if *min == f64::MIN {
                        flags |= NUMBER_FLAG_MIN;
                    }
                    if *max == f64::MAX {
                        flags |= NUMBER_FLAG_MAX;
                    }
                    flags.write(w);
                    if *min != f64::MIN {
                        min.write(w);
                    }
                    if *max != f64::MAX {
                        max.write(w);
                    }
                }
                Self::Integer { min, max } => {
                    let mut flags = 0u8;
                    if *min == i32::MIN {
                        flags |= NUMBER_FLAG_MIN;
                    }
                    if *max == i32::MAX {
                        flags |= NUMBER_FLAG_MAX;
                    }
                    flags.write(w);
                    if *min != i32::MIN {
                        min.write(w);
                    }
                    if *max != i32::MAX {
                        max.write(w);
                    }
                }
                Self::Long { min, max } => {
                    let mut flags = 0u8;
                    if *min == i64::MIN {
                        flags |= NUMBER_FLAG_MIN;
                    }
                    if *max == i64::MAX {
                        flags |= NUMBER_FLAG_MAX;
                    }
                    flags.write(w);
                    if *min != i64::MIN {
                        min.write(w);
                    }
                    if *max != i64::MAX {
                        max.write(w);
                    }
                }
                Self::String { ty } => {
                    ty.write(w);
                }
                Self::Entity {
                    single,
                    players_only,
                } => {
                    let mut flags = 0u8;
                    if *single {
                        flags |= FLAG_SINGLE;
                    }
                    if *players_only {
                        flags |= FLAG_PLAYERS_ONLY;
                    }
                    flags.write(w);
                }
                Self::ScoreHolder { multiple } => {
                    let mut flags = 0u8;
                    if *multiple {
                        flags |= FLAG_MULTIPLE;
                    }
                    flags.write(w);
                }
                Self::Time { min } => {
                    min.write(w);
                }
                Self::ResourceOrTag { registry_key } => {
                    registry_key.write(w);
                }
                Self::ResourceOrTagKey { registry_key } => {
                    registry_key.write(w);
                }
                Self::Resource { registry_key } => {
                    registry_key.write(w);
                }
                Self::ResourceKey { registry_key } => {
                    registry_key.write(w);
                }
                Self::ResourceSelector { registry_key } => {
                    registry_key.write(w);
                }
                _ => {}
            }
        }
    }
    fn len_s(&self) -> usize {
        let mut w = self.id().len_s();
        match self {
            Self::Float { min, max } => {
                w += 1;
                if *min != f32::MIN {
                    w += min.len_s();
                }
                if *max != f32::MAX {
                    w += max.len_s();
                }
            }
            Self::Double { min, max } => {
                w += 1;
                if *min != f64::MIN {
                    w += min.len_s();
                }
                if *max != f64::MAX {
                    w += max.len_s();
                }
            }
            Self::Integer { min, max } => {
                w += 1;
                if *min != i32::MIN {
                    w += min.len_s();
                }
                if *max != i32::MAX {
                    w += max.len_s();
                }
            }
            Self::Long { min, max } => {
                w += 1;
                if *min != i64::MIN {
                    w += min.len_s();
                }
                if *max != i64::MAX {
                    w += max.len_s();
                }
            }
            Self::String { ty } => {
                w += ty.len_s();
            }
            Self::Entity {
                single: _,
                players_only: _,
            } => {
                w += 1;
            }
            Self::ScoreHolder { multiple: _ } => {
                w += 1;
            }
            Self::Time { min } => {
                w += min.len_s();
            }
            Self::ResourceOrTag { registry_key } => {
                w += registry_key.len_s();
            }
            Self::ResourceOrTagKey { registry_key } => {
                w += registry_key.len_s();
            }
            Self::Resource { registry_key } => {
                w += registry_key.len_s();
            }
            Self::ResourceKey { registry_key } => {
                w += registry_key.len_s();
            }
            Self::ResourceSelector { registry_key } => {
                w += registry_key.len_s();
            }
            _ => {}
        }
        w
    }
}

impl<'a> Read<'a> for CommandArgumentType<'a> {
    fn read(buf: &mut Reader<'a>) -> Result<Self, Error> {
        use command_argument_type::*;

        Ok(match command_argument_type::read(buf)? {
            bool => Self::Bool,
            float => {
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
                Self::Float { min, max }
            }
            double => {
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
                Self::Double { min, max }
            }
            integer => {
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
                Self::Integer { min, max }
            }
            long => {
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
                Self::Long { min, max }
            }
            string => Self::String {
                ty: StringType::read(buf)?,
            },
            entity => {
                let flags = u8::read(buf)?;
                let single = flags & FLAG_SINGLE != 0;
                let players_only = flags & FLAG_PLAYERS_ONLY != 0;
                Self::Entity {
                    single,
                    players_only,
                }
            }
            game_profile => Self::GameProfile,
            block_pos => Self::BlockPos,
            column_pos => Self::ColumnPos,
            vec3 => Self::Vec3,
            vec2 => Self::Vec2,
            block_state => Self::BlockState,
            block_predicate => Self::BlockPredicate,
            item_stack => Self::ItemStack,
            item_predicate => Self::ItemPredicate,
            color => Self::Color,
            hex_color => Self::HexColor,
            component => Self::Component,
            style => Self::Style,
            message => Self::Message,
            nbt_compound_tag => Self::NbtCompoundTag,
            nbt_tag => Self::NbtTag,
            nbt_path => Self::NbtPath,
            objective => Self::Objective,
            objective_criteria => Self::ObjectiveCriteria,
            operation => Self::Operation,
            particle => Self::Particle,
            angle => Self::Angle,
            rotation => Self::Rotation,
            scoreboard_slot => Self::ScoreboardSlot,
            score_holder => Self::ScoreHolder {
                multiple: (u8::read(buf)? & FLAG_MULTIPLE) != 0,
            },
            swizzle => Self::Swizzle,
            team => Self::Team,
            item_slot => Self::ItemSlot,
            item_slots => Self::ItemSlots,
            resource_location => Self::ResourceLocation,
            function => Self::Function,
            entity_anchor => Self::EntityAnchor,
            int_range => Self::IntRange,
            float_range => Self::FloatRange,
            dimension => Self::Dimension,
            gamemode => Self::Gamemode,
            time => Self::Time {
                min: i32::read(buf)?,
            },
            resource_or_tag => Self::ResourceOrTag {
                registry_key: ResourceKey::read(buf)?,
            },
            resource_or_tag_key => Self::ResourceOrTagKey {
                registry_key: ResourceKey::read(buf)?,
            },
            resource => Self::Resource {
                registry_key: ResourceKey::read(buf)?,
            },
            resource_key => Self::ResourceKey {
                registry_key: ResourceKey::read(buf)?,
            },
            resource_selector => Self::ResourceSelector {
                registry_key: ResourceKey::read(buf)?,
            },
            template_mirror => Self::TemplateMirror,
            template_rotation => Self::TemplateRotation,
            heightmap => Self::Heightmap,
            loot_table => Self::LootTable,
            loot_predicate => Self::LootPredicate,
            loot_modifier => Self::LootModifier,
            dialog => Self::Dialog,
            uuid => Self::Uuid,
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
                Self::Root { children } => (FLAG_ROOT, *children, None),
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
                    (flags, *children, *redirect)
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
                    (flags, *children, *redirect)
                }
            };
            w.write_byte(flags);
            V21(children.len() as u32).write(w);
            for &child in children {
                V21(child).write(w);
            }
            if let Some(r) = redirect {
                V21(r).write(w);
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
            Self::Root { children } => (*children, None),
            Self::Literal {
                children, redirect, ..
            } => (*children, *redirect),
            Self::Argument {
                children, redirect, ..
            } => (*children, *redirect),
        };
        let mut l = 1
            + V21(children.len() as u32).len_s()
            + children.iter().map(|&x| V21(x).len_s()).sum::<usize>();
        if let Some(r) = redirect {
            l += V21(r).len_s();
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
        match flags & MASK_NODE {
            FLAG_ROOT => {}
            FLAG_LITERAL => {}
            FLAG_ARGUMENT => {}
            _ => return Err(Error),
        }

        todo!()
    }
}
