#![no_std]

use minecraft_data::command_argument_type;
use mser::{V21, Write, Writer};

#[derive(Clone, Copy)]
pub enum CommandNode<'a> {
    Root {
        children: &'a [u32],
    },
    Literal {
        children: &'a [u32],
        redirect: Option<u32>,
        name: &'a str,
        executable: bool,
    },
    Argument {
        children: &'a [u32],
        redirect: Option<u32>,
        name: &'a str,
        executable: bool,
        arg_type: command_argument_type,
        arg_prop: &'a [u8],
        suggestions: Option<Suggestions>,
    },
}

#[derive(Copy, Clone)]
pub enum Suggestions {
    AskServer,
}

impl Write for Suggestions {
    #[inline]
    unsafe fn write(&self, w: &mut Writer) {
        unsafe {
            match self {
                Self::AskServer => {
                    w.write(b"\x20minecraft:ask_server");
                }
            }
        }
    }

    #[inline]
    fn len_s(&self) -> usize {
        match self {
            Self::AskServer => 21,
        }
    }
}

const FLAG_ROOT: u8 = 0x00;
const FLAG_LITERAL: u8 = 0x01;
const FLAG_ARGUMENT: u8 = 0x02;
const FLAG_EXECUTABLE: u8 = 0x04;
const FLAG_REDIRECT: u8 = 0x08;
const FLAG_SUGGESTION: u8 = 0x10;

impl Write for CommandNode<'_> {
    unsafe fn write(&self, w: &mut Writer) {
        unsafe {
            match *self {
                Self::Root { children } => {
                    w.write_byte(FLAG_ROOT);
                    V21(children.len() as u32).write(w);
                    for &child in children {
                        V21(child).write(w);
                    }
                }
                Self::Literal {
                    children,
                    redirect,
                    name,
                    executable,
                } => {
                    let mut flags = FLAG_LITERAL;
                    if executable {
                        flags |= 0x04;
                    }
                    if redirect.is_some() {
                        flags |= 0x08;
                    }
                    w.write_byte(flags);
                    V21(children.len() as u32).write(w);
                    for &child in children {
                        V21(child).write(w);
                    }
                    if let Some(r) = redirect {
                        V21(r).write(w);
                    }
                    V21(name.len() as u32).write(w);
                    w.write(name.as_bytes());
                }
                Self::Argument {
                    children,
                    redirect,
                    name,
                    executable,
                    arg_type,
                    arg_prop,
                    suggestions,
                } => {
                    let mut flags = FLAG_ARGUMENT;
                    if executable {
                        flags |= FLAG_EXECUTABLE;
                    }
                    if redirect.is_some() {
                        flags |= FLAG_REDIRECT;
                    }
                    if suggestions.is_some() {
                        flags |= FLAG_SUGGESTION;
                    }
                    w.write_byte(flags);
                    V21(children.len() as u32).write(w);
                    for &child in children {
                        V21(child).write(w);
                    }
                    if let Some(r) = redirect {
                        V21(r).write(w);
                    }
                    V21(name.len() as u32).write(w);
                    w.write(name.as_bytes());
                    arg_type.write(w);
                    w.write(arg_prop);
                    if let Some(s) = suggestions {
                        s.write(w);
                    }
                }
            }
        }
    }

    fn len_s(&self) -> usize {
        match self {
            Self::Root { children } => {
                1 + V21(children.len() as u32).len_s()
                    + children.iter().map(|&x| V21(x).len_s()).sum::<usize>()
            }
            Self::Literal {
                children,
                redirect,
                name,
                executable: _,
            } => {
                let mut l = 1
                    + V21(children.len() as u32).len_s()
                    + children.iter().map(|&x| V21(x).len_s()).sum::<usize>();
                if let Some(r) = redirect {
                    l += V21(*r).len_s();
                }
                l += V21(name.len() as u32).len_s();
                l += name.len();
                l
            }
            Self::Argument {
                children,
                redirect,
                name,
                executable: _,
                arg_type,
                arg_prop,
                suggestions,
            } => {
                let mut l = 1
                    + V21(children.len() as u32).len_s()
                    + children.iter().map(|&x| V21(x).len_s()).sum::<usize>();
                if let Some(r) = redirect {
                    l += V21(*r).len_s();
                }
                l += V21(name.len() as u32).len_s();
                l += name.len();
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
