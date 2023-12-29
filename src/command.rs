use crate::{command_argument_type, UnsafeWriter, Write, V21};

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
    fn write(&self, w: &mut UnsafeWriter) {
        match self {
            Self::AskServer => {
                w.write(b"\x20minecraft:ask_server");
            }
        }
    }
    fn len(&self) -> usize {
        match self {
            Self::AskServer => 21,
        }
    }
}

impl Write for CommandNode<'_> {
    fn write(&self, w: &mut UnsafeWriter) {
        match *self {
            Self::Root { children } => {
                w.write_byte(0);
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
                let mut flags = 1;
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
                let mut flags = 2;
                if executable {
                    flags |= 0x04;
                }
                if redirect.is_some() {
                    flags |= 0x08;
                }
                if suggestions.is_some() {
                    flags |= 0x10;
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

    fn len(&self) -> usize {
        match self {
            Self::Root { children } => {
                1 + V21(children.len() as u32).len()
                    + children.iter().map(|&x| V21(x).len()).sum::<usize>()
            }
            Self::Literal {
                children,
                redirect,
                name,
                executable: _,
            } => {
                let mut l = 1
                    + V21(children.len() as u32).len()
                    + children.iter().map(|&x| V21(x).len()).sum::<usize>();
                if let Some(r) = redirect {
                    l += V21(*r).len();
                }
                l += V21(name.len() as u32).len();
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
                    + V21(children.len() as u32).len()
                    + children.iter().map(|&x| V21(x).len()).sum::<usize>();
                if let Some(r) = redirect {
                    l += V21(*r).len();
                }
                l += V21(name.len() as u32).len();
                l += name.len();
                l += arg_type.len();
                l += arg_prop.len();
                if let Some(s) = suggestions {
                    l += s.len();
                }
                l
            }
        }
    }
}
