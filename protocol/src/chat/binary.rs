use super::{
    COLOR, Component, Content, EXTRA, SCORE, SCORE_NAME, SCORE_OBJECTIVE, Style, TEXT, TRANSLATE,
    TRANSLATE_FALLBACK, TRANSLATE_WITH, TYPE, TextColor,
};
use crate::nbt::{ListInfo, StringTag, StringTagRaw, StringTagWriter, TagType};
use crate::str::SmolStr;
use alloc::alloc::Allocator;
use alloc::vec::Vec;
use mser::{Error, Read, UnsafeWriter, Write};

const fn content_type<A: Allocator>(content: &Content<A>) -> &'static [u8] {
    match content {
        Content::Literal { .. } => b"text",
        Content::Translatable { .. } => b"translatable",
        Content::Score { .. } => b"score",
        Content::Selector { .. } => b"selector",
        Content::Keybind { .. } => b"keybind",
        Content::BlockNbt { .. } | Content::EntityNbt { .. } | Content::StorageNbt { .. } => b"nbt",
        Content::Object { .. } => b"object",
    }
}

unsafe fn write_raw<A: Allocator>(
    content: &Content<A>,
    style: &Style<A>,
    children: &[Component<A>],
    w: &mut UnsafeWriter,
) {
    unsafe {
        TagType::String.write(w);
        StringTagRaw::new_unchecked(TYPE).write(w);
        StringTagRaw::new_unchecked(content_type(content)).write(w);
    }
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
        Content::Score { name, objective } => unsafe {
            TagType::Compound.write(w);
            StringTagRaw::new_unchecked(SCORE).write(w);
            TagType::String.write(w);
            StringTagRaw::new_unchecked(SCORE_NAME).write(w);
            StringTagWriter(name).write(w);
            TagType::String.write(w);
            StringTagRaw::new_unchecked(SCORE_OBJECTIVE).write(w);
            StringTagWriter(objective).write(w);
            TagType::End.write(w);
        },
        Content::Selector { pattern, separator } => {}
        Content::Keybind { keybind } => {}
        Content::BlockNbt {
            nbt_path,
            interpret,
            separator,
            pos,
        } => {}
        Content::EntityNbt {
            nbt_path,
            interpret,
            separator,
            selector,
        } => {}
        Content::StorageNbt {
            nbt_path,
            interpret,
            separator,
            storage,
        } => {}
        Content::Object { contents } => {}
    }
    unsafe {
        if let Some(color) = style.color {
            TagType::String.write(w);
            StringTagRaw::new_unchecked(COLOR).write(w);
            let mut buf = [0; 7];
            StringTagRaw::new_unchecked(color.name(&mut buf).as_bytes()).write(w);
        }

        let len = children.len();
        if len != 0 {
            TagType::List.write(w);
            StringTagRaw::new_unchecked(EXTRA).write(w);
            ListInfo(TagType::Compound, len as _).write(w);
            for Component {
                content,
                style,
                children,
            } in children
            {
                write_raw(content, style, children, w);
            }
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
    w += TagType::String.sz();
    w += StringTagRaw::new_unchecked(TYPE).sz();
    w += StringTagRaw::new_unchecked(content_type(content)).sz();
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
        Content::Score { name, objective } => {
            w += TagType::Compound.sz();
            w += StringTagRaw::new_unchecked(SCORE).sz();
            w += TagType::String.sz();
            w += StringTagRaw::new_unchecked(SCORE_NAME).sz();
            w += StringTagWriter(name).sz();
            w += TagType::String.sz();
            w += StringTagRaw::new_unchecked(SCORE_OBJECTIVE).sz();
            w += StringTagWriter(objective).sz();
            w += TagType::End.sz();
        }
        _ => {}
    };
    if let Some(color) = style.color {
        w += TagType::String.sz();
        w += StringTagRaw::new_unchecked(COLOR).sz();
        let mut buf = [0; 7];
        w += StringTagRaw::new_unchecked(color.name(&mut buf).as_bytes()).sz();
    }
    let len = children.len();
    if len != 0 {
        w += TagType::List.sz();
        w += StringTagRaw::new_unchecked(EXTRA).sz();
        w += ListInfo(TagType::Compound, len as _).sz();
        for Component {
            content,
            style,
            children,
        } in children
        {
            w += write_raw_len(content, style, children);
        }
    }
    w += TagType::End.sz();

    w
}

fn read_raw(buf: &mut &[u8]) -> Result<Component, Error> {
    let mut content: Option<Content> = None;
    let mut style = Style::new();
    let mut children = Vec::new();
    loop {
        let t1 = TagType::read(buf)?;
        macro_rules! expect_str {
            ($ty:expr) => {
                match $ty {
                    TagType::String => match StringTag::read(buf) {
                        Ok(x) => x.0,
                        Err(e) => return Err(e),
                    },
                    _ => return Err(Error),
                }
            };
        }
        if t1 == TagType::End {
            let content = match content {
                Some(x) => x,
                None => Content::Literal {
                    content: SmolStr::EMPTY,
                },
            };
            return Ok(Component {
                content,
                style,
                children,
            });
        }
        match StringTag::read(buf)?.0.as_bytes() {
            TEXT => {
                if content.is_none() {
                    content = Some(Content::Literal {
                        content: SmolStr::EMPTY,
                    })
                }
                if let Some(Content::Literal { content }) = content.as_mut() {
                    *content = expect_str!(t1);
                }
            }
            TRANSLATE => {
                if content.is_none() {
                    content = Some(Content::Translatable {
                        key: SmolStr::EMPTY,
                        fallback: None,
                        args: Vec::new(),
                    });
                }
                if let Some(Content::Translatable {
                    key,
                    fallback: _,
                    args: _,
                }) = content.as_mut()
                {
                    *key = expect_str!(t1);
                }
            }
            TRANSLATE_FALLBACK => {
                if content.is_none() {
                    content = Some(Content::Translatable {
                        key: SmolStr::EMPTY,
                        fallback: None,
                        args: Vec::new(),
                    });
                }
                if let Some(Content::Translatable {
                    key: _,
                    fallback,
                    args: _,
                }) = content.as_mut()
                {
                    *fallback = Some(expect_str!(t1));
                }
            }
            TRANSLATE_WITH => {
                if content.is_none() {
                    content = Some(Content::Translatable {
                        key: SmolStr::EMPTY,
                        fallback: None,
                        args: Vec::new(),
                    });
                }
                if let Some(Content::Translatable {
                    key: _,
                    fallback: _,
                    args,
                }) = content.as_mut()
                {
                    match t1 {
                        TagType::List => match ListInfo::read(buf)? {
                            ListInfo(TagType::Compound, len) => {
                                for _ in 0..len {
                                    args.push(read_raw(buf)?);
                                }
                            }
                            _ => return Err(Error),
                        },
                        _ => return Err(Error),
                    }
                }
            }
            SCORE => {
                if content.is_none() {
                    content = Some(Content::Score {
                        name: SmolStr::EMPTY,
                        objective: SmolStr::EMPTY,
                    });
                }
                if let Some(Content::Score { name, objective }) = content.as_mut() {
                    match t1 {
                        TagType::Compound => loop {
                            let t2 = TagType::read(buf)?;
                            if t2 == TagType::End {
                                break;
                            }
                            match StringTag::read(buf)?.0.as_bytes() {
                                SCORE_NAME => {
                                    *name = expect_str!(t2);
                                }
                                SCORE_OBJECTIVE => {
                                    *objective = expect_str!(t2);
                                }
                                _ => return Err(Error),
                            }
                        },
                        _ => return Err(Error),
                    }
                }
            }
            COLOR => {
                let color = expect_str!(t1);
                style.color = match TextColor::parse(color.as_bytes()) {
                    Some(x) => Some(x),
                    None => return Err(Error),
                };
            }
            TYPE => {
                expect_str!(t1);
            }
            EXTRA => match t1 {
                TagType::List => match ListInfo::read(buf)? {
                    ListInfo(TagType::Compound, len) => {
                        for _ in 0..len {
                            children.push(read_raw(buf)?);
                        }
                    }
                    _ => return Err(Error),
                },
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
