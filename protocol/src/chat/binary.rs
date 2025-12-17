use super::{COLOR, Component, Content, Style, TEXT, TRANSLATE};
use crate::chat::{TRANSLATE_FALLBACK, TRANSLATE_WITH, TextColor};
use crate::nbt::{ListInfo, StringTag, StringTagRaw, StringTagWriter, TagType};
use crate::str::SmolStr;
use alloc::alloc::Allocator;
use alloc::vec::Vec;
use mser::{Error, Read, UnsafeWriter, Write};

unsafe fn write_raw<A: Allocator>(
    content: &Content<A>,
    style: &Style<A>,
    children: &[Component<A>],
    w: &mut UnsafeWriter,
) {
    match content {
        Content::Literal { content } => unsafe {
            TagType::String.write(w);
            StringTagRaw::new_unchecked(TEXT).write(w);
            StringTagWriter(content).write(w);
        },
        Content::Translatable {
            key,
            fallback,
            args,
        } => unsafe {
            TagType::String.write(w);
            StringTagRaw::new_unchecked(TRANSLATE).write(w);
            StringTagWriter(key).write(w);
            if let Some(fallback) = fallback {
                TagType::String.write(w);
                StringTagRaw::new_unchecked(TRANSLATE_FALLBACK).write(w);
                StringTagWriter(fallback).write(w);
            }
            let len = args.len();
            if len != 0 {
                TagType::List.write(w);
                StringTagRaw::new_unchecked(TRANSLATE_WITH).write(w);
                ListInfo(TagType::Compound, len as _).write(w);
                for Component {
                    content,
                    style,
                    children,
                } in args
                {
                    write_raw(content, style, children, w);
                }
            }
        },
        _ => {}
    }
    unsafe {
        if let Some(color) = style.color {
            TagType::String.write(w);
            StringTagRaw::new_unchecked(COLOR).write(w);
            let mut buf = [0; 7];
            StringTagRaw::new_unchecked(color.name(&mut buf).as_bytes()).write(w);
        }
        TagType::End.write(w);
    }
}

fn write_raw_len<A: Allocator>(
    content: &Content<A>,
    style: &Style<A>,
    children: &[Component<A>],
) -> usize {
    let mut w = 0usize;
    match content {
        Content::Literal { content } => {
            w += TagType::String.sz();
            w += StringTagRaw::new_unchecked(TEXT).sz();
            w += StringTagWriter(content).sz();
        }
        Content::Translatable {
            key,
            fallback,
            args,
        } => {
            w += TagType::String.sz();
            w += StringTagRaw::new_unchecked(TRANSLATE).sz();
            w += StringTagWriter(key).sz();
            if let Some(fallback) = fallback {
                w += TagType::String.sz();
                w += StringTagRaw::new_unchecked(TRANSLATE_FALLBACK).sz();
                w += StringTagWriter(fallback).sz();
            }
            let len = args.len();
            if len != 0 {
                w += TagType::List.sz();
                w += StringTagRaw::new_unchecked(TRANSLATE_WITH).sz();
                w += ListInfo(TagType::Compound, len as _).sz();
                for Component {
                    content,
                    style,
                    children,
                } in args
                {
                    w += write_raw_len(content, style, children);
                }
            }
        }
        _ => {}
    };
    if let Some(color) = style.color {
        w += TagType::String.sz();
        w += StringTagRaw::new_unchecked(COLOR).sz();
        let mut buf = [0; 7];
        w += StringTagRaw::new_unchecked(color.name(&mut buf).as_bytes()).sz();
    }
    w += TagType::End.sz();

    w
}

fn read_raw(buf: &mut &[u8]) -> Result<Component, Error> {
    let mut content: Option<Content> = None;
    let mut style = Style::new();
    let children = Vec::new();
    loop {
        let tag_type = TagType::read(buf)?;
        if tag_type == TagType::End {
            return Ok(Component {
                content: match content {
                    Some(x) => x,
                    None => Content::Literal {
                        content: SmolStr::EMPTY,
                    },
                },
                style,
                children,
            });
        }
        let name = StringTag::read(buf)?;
        let name = name.0.as_str();
        match name.as_bytes() {
            TEXT => {
                match &content {
                    None => {
                        content = Some(Content::Literal {
                            content: SmolStr::EMPTY,
                        })
                    }
                    Some(Content::Literal { .. }) => {}
                    _ => return Err(Error),
                }
                match content.as_mut() {
                    Some(Content::Literal { content }) => match tag_type {
                        TagType::String => {
                            *content = StringTag::read(buf)?.0;
                        }
                        _ => return Err(Error),
                    },
                    _ => return Err(Error),
                }
            }
            TRANSLATE => {
                match &content {
                    None => {
                        content = Some(Content::Translatable {
                            key: SmolStr::EMPTY,
                            fallback: None,
                            args: Vec::new(),
                        })
                    }
                    Some(Content::Translatable { .. }) => {}
                    _ => return Err(Error),
                };
                match content.as_mut() {
                    Some(Content::Translatable {
                        key,
                        fallback: _,
                        args: _,
                    }) => match tag_type {
                        TagType::String => {
                            *key = StringTag::read(buf)?.0;
                        }
                        _ => return Err(Error),
                    },
                    _ => return Err(Error),
                }
            }
            TRANSLATE_FALLBACK => {
                match &content {
                    None => {
                        content = Some(Content::Translatable {
                            key: SmolStr::EMPTY,
                            fallback: None,
                            args: Vec::new(),
                        })
                    }
                    Some(Content::Translatable { .. }) => {}
                    _ => return Err(Error),
                };
                match content.as_mut() {
                    Some(Content::Translatable {
                        key: _,
                        fallback,
                        args: _,
                    }) => match tag_type {
                        TagType::String => {
                            *fallback = Some(StringTag::read(buf)?.0);
                        }
                        _ => return Err(Error),
                    },
                    _ => return Err(Error),
                }
            }
            TRANSLATE_WITH => {
                match &content {
                    None => {
                        content = Some(Content::Translatable {
                            key: SmolStr::EMPTY,
                            fallback: None,
                            args: Vec::new(),
                        })
                    }
                    Some(Content::Translatable { .. }) => {}
                    _ => return Err(Error),
                };
                match content.as_mut() {
                    Some(Content::Translatable {
                        key: _,
                        fallback: _,
                        args,
                    }) => match tag_type {
                        TagType::List => match ListInfo::read(buf)? {
                            ListInfo(TagType::Compound, len) => {
                                let mut vec = Vec::new();
                                for _ in 0..len {
                                    vec.push(read_raw(buf)?);
                                }
                                *args = vec;
                            }
                            _ => return Err(Error),
                        },
                        _ => return Err(Error),
                    },
                    _ => return Err(Error),
                }
            }
            COLOR => match tag_type {
                TagType::String => {
                    style.color =
                        Some(match TextColor::parse(StringTag::read(buf)?.0.as_bytes()) {
                            Some(x) => x,
                            None => return Err(Error),
                        });
                }
                _ => return Err(Error),
            },
            _ => return Err(Error),
        }
    }
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
            TagType::Compound.write(w);
            write_raw(content, style, children, w);
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
        w += TagType::Compound.sz();
        w += write_raw_len(content, style, children);
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
            TagType::List => {
                let ListInfo(ty, len) = ListInfo::read(buf)?;
                if ty == TagType::Compound {
                    let mut children = Vec::new();
                    for _ in 0..len {
                        children.push(read_raw(buf)?);
                    }
                    Ok(Self {
                        children,
                        style: Style::new(),
                        content: Content::Literal {
                            content: SmolStr::EMPTY,
                        },
                    })
                } else {
                    Err(Error)
                }
            }
            TagType::Compound => read_raw(buf),
            _ => Err(Error),
        }
    }
}
