#![no_std]

use minecraft_data::command_argument_type;
use mser::{Utf8, V21, Write, Writer};

#[derive(Clone, Copy)]
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
        arg_type: command_argument_type,
        arg_prop: &'a [u8],
        suggestions: Option<Suggestions>,
    },
}

#[derive(Copy, Clone)]
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

const FLAG_ROOT: u8 = 0x00;
const FLAG_LITERAL: u8 = 0x01;
const FLAG_ARGUMENT: u8 = 0x02;
const FLAG_EXECUTABLE: u8 = 0x04;
const FLAG_REDIRECT: u8 = 0x08;
const FLAG_SUGGESTION: u8 = 0x10;
const FLAG_RESTRICTED: u8 = 0x20;

impl Write for CommandNode<'_> {
    unsafe fn write(&self, w: &mut Writer) {
        unsafe {
            let (flags, children, redirect) = match *self {
                Self::Root { children } => (FLAG_ROOT, children, None),
                Self::Literal {
                    children,
                    redirect,
                    name: _,
                    executable,
                    restricted,
                } => {
                    let mut flags = FLAG_LITERAL;
                    if executable {
                        flags |= FLAG_EXECUTABLE;
                    }
                    if redirect.is_some() {
                        flags |= FLAG_REDIRECT;
                    }
                    if restricted {
                        flags |= FLAG_RESTRICTED;
                    }
                    (flags, children, redirect)
                }
                Self::Argument {
                    children,
                    redirect,
                    name: _,
                    executable,
                    restricted,
                    arg_type: _,
                    arg_prop: _,
                    suggestions,
                } => {
                    let mut flags = FLAG_ARGUMENT;
                    if executable {
                        flags |= FLAG_EXECUTABLE;
                    }
                    if redirect.is_some() {
                        flags |= FLAG_REDIRECT;
                    }
                    if restricted {
                        flags |= FLAG_RESTRICTED;
                    }
                    if suggestions.is_some() {
                        flags |= FLAG_SUGGESTION;
                    }
                    (flags, children, redirect)
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
            match *self {
                Self::Root { .. } => (),
                Self::Literal { name, .. } => {
                    name.write(w);
                }
                Self::Argument {
                    name,
                    arg_type,
                    arg_prop,
                    suggestions,
                    ..
                } => {
                    name.write(w);
                    arg_type.write(w);
                    w.write(arg_prop);
                    if let Some(s) = suggestions {
                        s.write(w);
                    }
                }
            };
        }
    }

    fn len_s(&self) -> usize {
        let (children, redirect) = match *self {
            Self::Root { children } => (children, None),
            Self::Literal {
                children, redirect, ..
            } => (children, redirect),
            Self::Argument {
                children, redirect, ..
            } => (children, redirect),
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
                arg_prop,
                suggestions,
                ..
            } => {
                l += name.len_s();
                l += arg_type.len_s();
                l += arg_prop.len_s();
                if let Some(s) = suggestions {
                    l += s.len_s();
                }
                l
            }
        }
    }
}
