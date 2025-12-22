use super::{
    COLOR, Component, Content, EXTRA, KEYBIND, NBT, NBT_BLOCK, NBT_ENTITY, NBT_INTERPRET,
    NBT_SOURCE, NBT_STORAGE, NbtContents, SCORE, SCORE_NAME, SCORE_OBJECTIVE, SELECTOR, SEPARATOR,
    Style, TEXT, TRANSLATE, TRANSLATE_FALLBACK, TRANSLATE_WITH, TYPE, TextColor,
};
use crate::nbt::{IdentifierTag, ListInfo, RefStringTag, StringTag, StringTagRaw, TagType};
use crate::str::BoxStr;
use crate::{Ident, Identifier};
use alloc::alloc::Allocator;
use alloc::boxed::Box;
use alloc::vec::Vec;
use mser::{Bytes, Error, Read, UnsafeWriter, Write};

const STRING: TagType = TagType::String;
const LIST: TagType = TagType::List;
const COMPOUND: TagType = TagType::Compound;
const BOOL: TagType = TagType::Byte;
const END: TagType = TagType::End;

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
const KEYBIND_H: u128 = cast2(KEYBIND);
const NBT_H: u128 = cast2(NBT);
const NBT_INTERPRET_H: u128 = cast2(NBT_INTERPRET);
const NBT_SOURCE_H: u128 = cast2(NBT_SOURCE);
const NBT_BLOCK_H: u128 = cast2(NBT_BLOCK);
const NBT_ENTITY_H: u128 = cast2(NBT_ENTITY);
const NBT_STORAGE_H: u128 = cast2(NBT_STORAGE);

const fn content_type<A: Allocator>(content: &Content<A>) -> &'static [u8] {
    match content {
        Content::Literal { .. } => b"text",
        Content::Translatable { .. } => b"translatable",
        Content::Score { .. } => b"score",
        Content::Selector { .. } => b"selector",
        Content::Keybind { .. } => b"keybind",
        Content::Nbt { .. } => b"nbt",
        Content::Object { .. } => b"object",
    }
}

const fn nbt_source<A: Allocator>(content: &NbtContents<A>) -> &'static [u8] {
    match content {
        NbtContents::Block { .. } => b"block",
        NbtContents::Entity { .. } => b"entity",
        NbtContents::Storage { .. } => b"storage",
    }
}

fn mutf8(n: &[u8]) -> StringTagRaw<'_> {
    StringTagRaw::new_unchecked(n)
}

unsafe fn write_rec<A: Allocator>(
    content: &Content<A>,
    style: &Style<A>,
    children: &[Component<A>],
    w: &mut UnsafeWriter,
) {
    unsafe {
        STRING.write(w);
        mutf8(TYPE).write(w);
        mutf8(content_type(content)).write(w);
    }
    match content {
        Content::Literal { content } => unsafe {
            STRING.write(w);
            mutf8(TEXT).write(w);
            RefStringTag(content).write(w);
        },
        Content::Translatable {
            key,
            fallback,
            args,
        } => unsafe {
            STRING.write(w);
            mutf8(TRANSLATE).write(w);
            RefStringTag(key).write(w);
            if let Some(fallback) = fallback {
                STRING.write(w);
                mutf8(TRANSLATE_FALLBACK).write(w);
                RefStringTag(fallback).write(w);
            }
            let len = args.len();
            if len != 0 {
                LIST.write(w);
                mutf8(TRANSLATE_WITH).write(w);
                ListInfo(COMPOUND, len as _).write(w);
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
            COMPOUND.write(w);
            mutf8(SCORE).write(w);
            STRING.write(w);
            mutf8(SCORE_NAME).write(w);
            RefStringTag(name).write(w);
            STRING.write(w);
            mutf8(SCORE_OBJECTIVE).write(w);
            RefStringTag(objective).write(w);
            END.write(w);
        },
        Content::Selector { pattern, separator } => unsafe {
            STRING.write(w);
            mutf8(SELECTOR).write(w);
            RefStringTag(pattern).write(w);
            if let Some(Component {
                content,
                style,
                children,
            }) = separator.as_deref()
            {
                COMPOUND.write(w);
                mutf8(SEPARATOR).write(w);
                write_rec(content, style, children, w);
            }
        },
        Content::Keybind { keybind } => unsafe {
            STRING.write(w);
            mutf8(KEYBIND).write(w);
            RefStringTag(keybind).write(w);
        },
        Content::Nbt {
            nbt_path,
            interpret,
            content,
            separator,
        } => unsafe {
            STRING.write(w);
            mutf8(NBT).write(w);
            RefStringTag(nbt_path).write(w);
            if *interpret {
                BOOL.write(w);
                mutf8(NBT_INTERPRET).write(w);
                interpret.write(w);
            }
            if let Some(Component {
                content,
                style,
                children,
            }) = separator.as_deref()
            {
                COMPOUND.write(w);
                mutf8(SEPARATOR).write(w);
                write_rec(content, style, children, w);
            }
            STRING.write(w);
            mutf8(NBT_SOURCE).write(w);
            mutf8(nbt_source(content)).write(w);
            match content {
                NbtContents::Block { pos } => {
                    STRING.write(w);
                    mutf8(NBT_BLOCK).write(w);
                    RefStringTag(pos).write(w);
                }
                NbtContents::Entity { selector } => {
                    STRING.write(w);
                    mutf8(NBT_ENTITY).write(w);
                    RefStringTag(selector).write(w);
                }
                NbtContents::Storage { storage } => {
                    STRING.write(w);
                    mutf8(NBT_STORAGE).write(w);
                    IdentifierTag(Ident {
                        namespace: &storage.namespace,
                        path: &storage.path,
                    })
                    .write(w);
                }
            }
        },
        Content::Object { content } => {}
    }
    unsafe {
        let len = children.len();
        if len != 0 {
            LIST.write(w);
            mutf8(EXTRA).write(w);
            ListInfo(COMPOUND, len as _).write(w);
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
            STRING.write(w);
            mutf8(COLOR).write(w);
            let mut buf = [0; 7];
            mutf8(color.name(&mut buf).as_bytes()).write(w);
        }

        END.write(w);
    }
}

fn write_raw_len<A: Allocator>(
    content: &Content<A>,
    style: &Style<A>,
    children: &[Component<A>],
) -> usize {
    let mut w = 0usize;
    w += STRING.sz();
    w += mutf8(TYPE).sz();
    w += mutf8(content_type(content)).sz();
    match content {
        Content::Literal { content } => {
            w += STRING.sz();
            w += mutf8(TEXT).sz();
            w += RefStringTag(content).sz();
        }
        Content::Translatable {
            key,
            fallback,
            args,
        } => {
            w += STRING.sz();
            w += mutf8(TRANSLATE).sz();
            w += RefStringTag(key).sz();
            if let Some(fallback) = fallback {
                w += STRING.sz();
                w += mutf8(TRANSLATE_FALLBACK).sz();
                w += RefStringTag(fallback).sz();
            }
            let len = args.len();
            if len != 0 {
                w += LIST.sz();
                w += mutf8(TRANSLATE_WITH).sz();
                w += ListInfo(COMPOUND, len as _).sz();
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
            w += COMPOUND.sz();
            w += mutf8(SCORE).sz();
            w += STRING.sz();
            w += mutf8(SCORE_NAME).sz();
            w += RefStringTag(name).sz();
            w += STRING.sz();
            w += mutf8(SCORE_OBJECTIVE).sz();
            w += RefStringTag(objective).sz();
            w += END.sz();
        }
        Content::Selector { pattern, separator } => {
            w += STRING.sz();
            w += mutf8(SELECTOR).sz();
            w += RefStringTag(pattern).sz();
            if let Some(Component {
                content,
                style,
                children,
            }) = separator.as_deref()
            {
                w += COMPOUND.sz();
                w += mutf8(SEPARATOR).sz();
                w += write_raw_len(content, style, children);
            }
        }
        Content::Keybind { keybind } => {
            w += STRING.sz();
            w += mutf8(KEYBIND).sz();
            w += RefStringTag(keybind).sz();
        }
        Content::Nbt {
            nbt_path,
            interpret,
            separator,
            content,
        } => {
            w += STRING.sz();
            w += mutf8(NBT).sz();
            w += RefStringTag(nbt_path).sz();
            if *interpret {
                w += BOOL.sz();
                w += mutf8(NBT_INTERPRET).sz();
                w += interpret.sz();
            }
            if let Some(Component {
                content,
                style,
                children,
            }) = separator.as_deref()
            {
                w += COMPOUND.sz();
                w += mutf8(SEPARATOR).sz();
                w += write_raw_len(content, style, children);
            }
            w += STRING.sz();
            w += mutf8(NBT_SOURCE).sz();
            w += mutf8(nbt_source(content)).sz();
            match content {
                NbtContents::Block { pos } => {
                    w += STRING.sz();
                    w += mutf8(NBT_BLOCK).sz();
                    w += RefStringTag(pos).sz();
                }
                NbtContents::Entity { selector } => {
                    w += STRING.sz();
                    w += mutf8(NBT_ENTITY).sz();
                    w += RefStringTag(selector).sz();
                }
                NbtContents::Storage { storage } => {
                    w += STRING.sz();
                    w += mutf8(NBT_STORAGE).sz();
                    w += IdentifierTag(Ident {
                        namespace: &storage.namespace,
                        path: &storage.path,
                    })
                    .sz();
                }
            }
        }
        _ => {}
    };

    let len = children.len();
    if len != 0 {
        w += LIST.sz();
        w += mutf8(EXTRA).sz();
        w += ListInfo(COMPOUND, len as _).sz();
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
        w += STRING.sz();
        w += mutf8(COLOR).sz();
        let mut buf = [0; 7];
        w += mutf8(color.name(&mut buf).as_bytes()).sz();
    }
    w += END.sz();

    w
}

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
    let mut separator: Option<Box<Component>> = None;
    loop {
        let t1 = TagType::read(buf)?;

        if t1 == END {
            let content = match content {
                Some(Content::Selector {
                    pattern,
                    separator: _,
                }) => Content::Selector { pattern, separator },
                Some(Content::Nbt {
                    nbt_path,
                    interpret,
                    separator: _,
                    content,
                }) => Content::Nbt {
                    nbt_path,
                    interpret,
                    separator,
                    content,
                },
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
            TEXT_H => match content.as_mut() {
                None => {
                    content = Some(Content::Literal {
                        content: expect_str(t1, buf)?,
                    })
                }
                Some(Content::Literal { content }) => {
                    *content = expect_str(t1, buf)?;
                }
                _ => return Err(Error),
            },
            TRANSLATE_H => match content.as_mut() {
                None => {
                    content = Some(Content::Translatable {
                        key: expect_str(t1, buf)?,
                        fallback: None,
                        args: Vec::new(),
                    })
                }
                Some(Content::Translatable {
                    key,
                    fallback: _,
                    args: _,
                }) => {
                    *key = expect_str(t1, buf)?;
                }
                _ => return Err(Error),
            },
            TRANSLATE_FALLBACK_H => match content.as_mut() {
                None => {
                    content = Some(Content::Translatable {
                        key: BoxStr::default(),
                        fallback: Some(expect_str(t1, buf)?),
                        args: Vec::new(),
                    })
                }
                Some(Content::Translatable {
                    key: _,
                    fallback,
                    args: _,
                }) => {
                    *fallback = Some(expect_str(t1, buf)?);
                }
                _ => return Err(Error),
            },
            TRANSLATE_WITH_H => {
                let mut args = Vec::new();
                match t1 {
                    LIST => match ListInfo::read(buf)? {
                        ListInfo(COMPOUND, len) => {
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
                    COMPOUND => loop {
                        let t2 = TagType::read(buf)?;
                        if t2 == END {
                            break;
                        }
                        match StringTag::read(buf)?.0.as_bytes() {
                            SCORE_NAME => {
                                name = Some(expect_str(t2, buf)?);
                            }
                            SCORE_OBJECTIVE => {
                                objective = Some(expect_str(t2, buf)?);
                            }
                            _ => return Err(Error),
                        }
                    },
                    _ => return Err(Error),
                }
                let name = match name {
                    Some(name) => name,
                    None => return Err(Error),
                };
                let objective = match objective {
                    Some(objective) => objective,
                    None => return Err(Error),
                };
                match content.as_mut() {
                    None => content = Some(Content::Score { name, objective }),
                    Some(Content::Score {
                        name: x,
                        objective: y,
                    }) => {
                        *x = name;
                        *y = objective;
                    }
                    _ => return Err(Error),
                }
            }
            SELECTOR_H => match content.as_mut() {
                None => {
                    content = Some(Content::Selector {
                        pattern: expect_str(t1, buf)?,
                        separator: None,
                    })
                }
                Some(Content::Selector {
                    pattern,
                    separator: _,
                }) => {
                    *pattern = expect_str(t1, buf)?;
                }
                _ => return Err(Error),
            },
            SEPARATOR_H => {
                separator = Some(Box::new(read_rec_ty(buf, t1)?));
            }
            KEYBIND_H => match content.as_ref() {
                None => {
                    content = Some(Content::Keybind {
                        keybind: expect_str(t1, buf)?,
                    })
                }
                _ => return Err(Error),
            },
            NBT_H => match content.as_mut() {
                None => {
                    content = Some(Content::Nbt {
                        nbt_path: expect_str(t1, buf)?,
                        interpret: false,
                        separator: None,
                        content: NbtContents::Block {
                            pos: BoxStr::default(),
                        },
                    })
                }
                Some(Content::Nbt {
                    nbt_path,
                    interpret: _,
                    separator: _,
                    content: _,
                }) => {
                    *nbt_path = expect_str(t1, buf)?;
                }
                _ => return Err(Error),
            },
            NBT_INTERPRET_H => {
                let x = expect_bool(t1, buf)?;
                match content.as_mut() {
                    None => {
                        content = Some(Content::Nbt {
                            nbt_path: BoxStr::default(),
                            interpret: x,
                            separator: None,
                            content: NbtContents::Block {
                                pos: BoxStr::default(),
                            },
                        })
                    }
                    Some(Content::Nbt {
                        nbt_path: _,
                        interpret,
                        separator: _,
                        content: _,
                    }) => {
                        *interpret = x;
                    }
                    _ => return Err(Error),
                }
            }
            NBT_SOURCE_H => {
                expect_str(t1, buf)?;
            }
            NBT_BLOCK_H => {
                let x = expect_str(t1, buf)?;
                match content.as_mut() {
                    None => {
                        content = Some(Content::Nbt {
                            nbt_path: BoxStr::default(),
                            interpret: false,
                            separator: None,
                            content: NbtContents::Block { pos: x },
                        })
                    }
                    Some(Content::Nbt {
                        nbt_path: _,
                        interpret: _,
                        separator: _,
                        content,
                    }) => {
                        *content = NbtContents::Block { pos: x };
                    }
                    _ => return Err(Error),
                }
            }
            NBT_ENTITY_H => {
                let x = expect_str(t1, buf)?;
                match content.as_mut() {
                    None => {
                        content = Some(Content::Nbt {
                            nbt_path: BoxStr::default(),
                            interpret: false,
                            separator: None,
                            content: NbtContents::Entity { selector: x },
                        })
                    }
                    Some(Content::Nbt {
                        nbt_path: _,
                        interpret: _,
                        separator: _,
                        content,
                    }) => {
                        *content = NbtContents::Entity { selector: x };
                    }
                    _ => return Err(Error),
                }
            }
            NBT_STORAGE_H => {
                let x = expect_ident(t1, buf)?;
                match content.as_mut() {
                    None => {
                        content = Some(Content::Nbt {
                            nbt_path: BoxStr::default(),
                            interpret: false,
                            separator: None,
                            content: NbtContents::Storage { storage: x },
                        })
                    }
                    Some(Content::Nbt {
                        nbt_path: _,
                        interpret: _,
                        separator: _,
                        content,
                    }) => {
                        *content = NbtContents::Storage { storage: x };
                    }
                    _ => return Err(Error),
                }
            }
            TYPE_H => {
                expect_str(t1, buf)?;
            }
            EXTRA_H => match t1 {
                LIST => match ListInfo::read(buf)? {
                    ListInfo(COMPOUND, len) => {
                        for _ in 0..len {
                            children.push(read_rec_compound(buf)?);
                        }
                    }
                    _ => return Err(Error),
                },
                _ => return Err(Error),
            },
            COLOR_H => {
                let color = expect_str(t1, buf)?;
                style.color = match TextColor::parse(color.as_bytes()) {
                    Some(x) => Some(x),
                    None => return Err(Error),
                };
            }
            _ => return Err(Error),
        }
    }
}

fn expect_ident(t: TagType, buf: &mut &[u8]) -> Result<Identifier, Error> {
    match t {
        TagType::String => match IdentifierTag::read(buf) {
            Ok(x) => unsafe {
                Ok(Identifier {
                    namespace: BoxStr::new_unchecked(Box::from(x.0.namespace.as_bytes())),
                    path: BoxStr::new_unchecked(Box::from(x.0.path.as_bytes())),
                })
            },
            Err(e) => Err(e),
        },
        _ => Err(Error),
    }
}

fn expect_str(t: TagType, buf: &mut &[u8]) -> Result<BoxStr, Error> {
    match t {
        TagType::String => match StringTag::read(buf) {
            Ok(x) => Ok(x.0),
            Err(e) => Err(e),
        },
        _ => Err(Error),
    }
}

fn expect_bool(t: TagType, buf: &mut &[u8]) -> Result<bool, Error> {
    match t {
        TagType::Byte => Ok(buf.i8()? != 0),
        TagType::Short => Ok(buf.i16()? != 0),
        TagType::Int => Ok(buf.i32()? != 0),
        TagType::Long => Ok(buf.i64()? != 0),
        TagType::Float => Ok(buf.f32()? != 0.0),
        TagType::Double => Ok(buf.f64()? != 0.0),
        _ => Err(Error),
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
                RefStringTag(content).write(w);
                return;
            }
            COMPOUND.write(w);
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
            w += RefStringTag(content).sz();
            return w;
        }
        w += COMPOUND.sz();
        w += write_raw_len(content, style, children);
        w
    }
}

fn read_rec_ty(buf: &mut &[u8], ty: TagType) -> Result<Component, Error> {
    match ty {
        STRING => Ok(Component {
            children: Vec::new(),
            style: Style::new(),
            content: Content::Literal {
                content: StringTag::read(buf)?.0,
            },
        }),
        LIST => {
            let ListInfo(ty, len) = ListInfo::read(buf)?;
            if ty == COMPOUND {
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
        COMPOUND => read_rec_compound(buf),
        _ => Err(Error),
    }
}

impl<'a> Read<'a> for Component {
    fn read(buf: &mut &'a [u8]) -> Result<Self, Error> {
        let ty = TagType::read(buf)?;
        read_rec_ty(buf, ty)
    }
}
