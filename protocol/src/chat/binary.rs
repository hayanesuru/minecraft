use super::{
    COLOR, Component, Content, EXTRA, SCORE, SCORE_NAME, SCORE_OBJECTIVE, SELECTOR, SEPARATOR,
    Style, TEXT, TRANSLATE, TRANSLATE_FALLBACK, TRANSLATE_WITH, TYPE, TextColor,
};
use crate::nbt::{ListInfo, StringTag, StringTagRaw, StringTagWriter, TagType};
use crate::str::BoxStr;
use alloc::alloc::Allocator;
use alloc::boxed::Box;
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

unsafe fn write_rec<A: Allocator>(
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
                    write_rec(content, style, children, w);
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
        Content::Selector { pattern, separator } => unsafe {
            TagType::String.write(w);
            StringTagRaw::new_unchecked(SELECTOR).write(w);
            StringTagWriter(pattern).write(w);
            if let Some(Component {
                content,
                style,
                children,
            }) = separator.as_deref()
            {
                TagType::Compound.write(w);
                StringTagRaw::new_unchecked(SEPARATOR).write(w);
                write_rec(content, style, children, w);
            }
        },
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
                write_rec(content, style, children, w);
            }
        }

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
        Content::Selector { pattern, separator } => {
            w += TagType::String.sz();
            w += StringTagRaw::new_unchecked(SELECTOR).sz();
            w += StringTagWriter(pattern).sz();
            if let Some(Component {
                content,
                style,
                children,
            }) = separator.as_deref()
            {
                w += TagType::Compound.sz();
                w += StringTagRaw::new_unchecked(SEPARATOR).sz();
                w += write_raw_len(content, style, children);
            }
        }
        _ => {}
    };

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

    if let Some(color) = style.color {
        w += TagType::String.sz();
        w += StringTagRaw::new_unchecked(COLOR).sz();
        let mut buf = [0; 7];
        w += StringTagRaw::new_unchecked(color.name(&mut buf).as_bytes()).sz();
    }
    w += TagType::End.sz();

    w
}

const TEXT_H: u128 = cast2(TEXT);
const TRANSLATE_H: u128 = cast2(TRANSLATE);
const TRANSLATE_FALLBACK_H: u128 = cast2(TRANSLATE_FALLBACK);
const TRANSLATE_WITH_H: u128 = cast2(TRANSLATE_WITH);
const SCORE_H: u128 = cast2(SCORE);
const TYPE_H: u128 = cast2(TYPE);
const EXTRA_H: u128 = cast2(EXTRA);
const COLOR_H: u128 = cast2(COLOR);
const SELECTOR_H: u128 = cast2(SELECTOR);
const SEPARATOR_H: u128 = cast2(SEPARATOR);

const fn cast128(n: &[u8]) -> Result<u128, Error> {
    if n.len() <= 16 {
        Ok(cast2(n))
    } else {
        Err(Error)
    }
}

const fn cast2(n: &[u8]) -> u128 {
    debug_assert!(n.len() <= 16);
    let len = n.len();
    let mut dest = [0u8; 16];
    if len > 16 {
        unsafe { core::hint::unreachable_unchecked() }
    }
    unsafe {
        core::ptr::copy_nonoverlapping(n.as_ptr(), dest.as_mut_ptr(), len);
    }
    u128::from_le_bytes(dest)
}

fn read_rec_compound(buf: &mut &[u8]) -> Result<Component, Error> {
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
                    content: BoxStr::default(),
                },
            };
            return Ok(Component {
                content,
                style,
                children,
            });
        }

        match cast128(StringTagRaw::read(buf)?.inner())? {
            TEXT_H => match content.as_ref() {
                None => {
                    content = Some(Content::Literal {
                        content: expect_str!(t1),
                    })
                }
                _ => return Err(Error),
            },
            TRANSLATE_H => match content.as_mut() {
                None => {
                    content = Some(Content::Translatable {
                        key: expect_str!(t1),
                        fallback: None,
                        args: Vec::new(),
                    })
                }
                Some(Content::Translatable {
                    key,
                    fallback: _,
                    args: _,
                }) => {
                    *key = expect_str!(t1);
                }
                _ => return Err(Error),
            },
            TRANSLATE_FALLBACK_H => match content.as_mut() {
                None => {
                    content = Some(Content::Translatable {
                        key: BoxStr::default(),
                        fallback: Some(expect_str!(t1)),
                        args: Vec::new(),
                    })
                }
                Some(Content::Translatable {
                    key: _,
                    fallback,
                    args: _,
                }) => {
                    *fallback = Some(expect_str!(t1));
                }
                _ => return Err(Error),
            },
            TRANSLATE_WITH_H => {
                let mut args = Vec::new();
                match t1 {
                    TagType::List => match ListInfo::read(buf)? {
                        ListInfo(TagType::Compound, len) => {
                            for _ in 0..len {
                                args.push(read_rec_compound(buf)?);
                            }
                        }
                        _ => return Err(Error),
                    },
                    _ => return Err(Error),
                };
                match content.as_mut() {
                    None => {
                        content = Some(Content::Translatable {
                            key: BoxStr::default(),
                            fallback: None,
                            args,
                        })
                    }
                    Some(Content::Translatable {
                        key: _,
                        fallback: _,
                        args: x,
                    }) => *x = args,
                    _ => return Err(Error),
                }
            }
            SCORE_H => {
                let mut name: Option<BoxStr> = None;
                let mut objective: Option<BoxStr> = None;
                match t1 {
                    TagType::Compound => loop {
                        let t2 = TagType::read(buf)?;
                        if t2 == TagType::End {
                            break;
                        }
                        match StringTag::read(buf)?.0.as_bytes() {
                            SCORE_NAME => {
                                name = Some(expect_str!(t2));
                            }
                            SCORE_OBJECTIVE => {
                                objective = Some(expect_str!(t2));
                            }
                            _ => return Err(Error),
                        }
                    },
                    _ => return Err(Error),
                }
                match content.as_ref() {
                    None => {
                        content = Some(Content::Score {
                            name: match name {
                                Some(name) => name,
                                None => return Err(Error),
                            },
                            objective: match objective {
                                Some(objective) => objective,
                                None => return Err(Error),
                            },
                        })
                    }
                    _ => return Err(Error),
                }
            }
            SELECTOR_H => match content.as_mut() {
                None => {
                    content = Some(Content::Selector {
                        pattern: expect_str!(t1),
                        separator: None,
                    })
                }
                Some(Content::Selector {
                    pattern,
                    separator: _,
                }) => {
                    *pattern = expect_str!(t1);
                }
                _ => return Err(Error),
            },
            SEPARATOR_H => {
                let separator = Some(Box::new(read_rec_ty(buf, t1)?));
                match content.as_mut() {
                    None => {
                        content = Some(Content::Selector {
                            pattern: BoxStr::default(),
                            separator,
                        })
                    }
                    Some(Content::Selector {
                        pattern: _,
                        separator: x,
                    }) => {
                        *x = separator;
                    }
                    _ => return Err(Error),
                }
            }
            TYPE_H => {
                expect_str!(t1);
            }
            EXTRA_H => match t1 {
                TagType::List => match ListInfo::read(buf)? {
                    ListInfo(TagType::Compound, len) => {
                        for _ in 0..len {
                            children.push(read_rec_compound(buf)?);
                        }
                    }
                    _ => return Err(Error),
                },
                _ => return Err(Error),
            },
            COLOR_H => {
                let color = expect_str!(t1);
                style.color = match TextColor::parse(color.as_bytes()) {
                    Some(x) => Some(x),
                    None => return Err(Error),
                };
            }
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
            write_rec(content, style, children, w);
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

fn read_rec_ty(buf: &mut &[u8], ty: TagType) -> Result<Component, Error> {
    match ty {
        TagType::String => Ok(Component {
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
                    children.push(read_rec_compound(buf)?);
                }
                Ok(Component {
                    children,
                    style: Style::new(),
                    content: Content::Literal {
                        content: BoxStr::default(),
                    },
                })
            } else {
                Err(Error)
            }
        }
        TagType::Compound => read_rec_compound(buf),
        _ => Err(Error),
    }
}

impl<'a> Read<'a> for Component {
    fn read(buf: &mut &'a [u8]) -> Result<Self, Error> {
        let ty = TagType::read(buf)?;
        read_rec_ty(buf, ty)
    }
}
