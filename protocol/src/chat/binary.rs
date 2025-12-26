use super::*;
use crate::nbt::{Kv, ListInfo, RefStringTag, StringTag, TagType};
use mser::{Error, Read, UnsafeWriter, Write};

const STRING: TagType = TagType::String;
const LIST: TagType = TagType::List;
const COMPOUND: TagType = TagType::Compound;
const END: TagType = TagType::End;

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

const fn nbt_type<A: Allocator>(content: &NbtContent<A>) -> &'static [u8] {
    match content {
        NbtContent::Block { .. } => b"block",
        NbtContent::Entity { .. } => b"entity",
        NbtContent::Storage { .. } => b"storage",
    }
}

const fn object_type<A: Allocator>(content: &ObjectContents<A>) -> &'static [u8] {
    match content {
        ObjectContents::Atlas {
            atlas: _,
            sprite: _,
        } => b"atlas",
        ObjectContents::Player { player: _, hat: _ } => b"player",
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
        Kv(TYPE, mutf8(content_type(content))).write(w);
    }
    match content {
        Content::Literal { content } => unsafe {
            Kv(TEXT, content).write(w);
        },
        Content::Translatable {
            key,
            fallback,
            args,
        } => unsafe {
            Kv(TRANSLATE, key).write(w);
            if let Some(fallback) = fallback {
                Kv(TRANSLATE_FALLBACK, fallback).write(w);
            }
            let len = args.len();
            if len != 0 {
                Kv(TRANSLATE_WITH, ListInfo(COMPOUND, len as _)).write(w);
                for arg in args {
                    arg.write_ty(w);
                }
            }
        },
        Content::Score { name, objective } => unsafe {
            COMPOUND.write(w);
            mutf8(SCORE).write(w);
            Kv(SCORE_NAME, name).write(w);
            Kv(SCORE_OBJECTIVE, objective).write(w);
            END.write(w);
        },
        Content::Selector { pattern, separator } => unsafe {
            Kv(SELECTOR, pattern).write(w);
            if let Some(separator) = separator.as_deref() {
                Kv(SEPARATOR, separator).write(w);
            }
        },
        Content::Keybind { keybind } => unsafe {
            Kv(KEYBIND, keybind).write(w);
        },
        Content::Nbt {
            nbt_path,
            interpret,
            content,
            separator,
        } => unsafe {
            Kv(NBT_PATH, nbt_path).write(w);
            if *interpret {
                Kv(NBT_INTERPRET, *interpret).write(w);
            }
            if let Some(separator) = separator.as_deref() {
                Kv(SEPARATOR, separator).write(w);
            }
            Kv(NBT_SOURCE, mutf8(nbt_type(content))).write(w);
            match content {
                NbtContent::Block { pos } => {
                    Kv(NBT_BLOCK, pos).write(w);
                }
                NbtContent::Entity { selector } => {
                    Kv(NBT_ENTITY, selector).write(w);
                }
                NbtContent::Storage { storage } => {
                    Kv(NBT_STORAGE, storage).write(w);
                }
            }
        },
        Content::Object { content } => unsafe {
            Kv(OBJECT_TYPE, mutf8(object_type(content))).write(w);
            match content {
                ObjectContents::Atlas { atlas, sprite } => {
                    let atlas = atlas.as_ident();
                    if atlas != DEFAULT_ATLAS {
                        Kv(OBJECT_ATLAS, atlas).write(w);
                    }
                    Kv(OBJECT_SPRITE, sprite.as_ident()).write(w);
                }
                ObjectContents::Player { player, hat } => {
                    Kv(OBJECT_PLAYER, &**player).write(w);
                    if !hat {
                        Kv(OBJECT_HAT, *hat).write(w);
                    }
                }
            }
        },
    }
    unsafe {
        let len = children.len();
        if len != 0 {
            Kv(EXTRA, ListInfo(COMPOUND, len as _)).write(w);
            for child in children {
                child.write_ty(w);
            }
        }

        if let Some(color) = style.color {
            let mut buf = [0; 7];
            Kv(COLOR, mutf8(color.name(&mut buf).as_bytes())).write(w);
        }

        END.write(w);
    }
}

fn write_rec_len<A: Allocator>(
    content: &Content<A>,
    style: &Style<A>,
    children: &[Component<A>],
) -> usize {
    let mut w = 0usize;
    w += Kv(TYPE, mutf8(content_type(content))).sz();
    match content {
        Content::Literal { content } => {
            w += Kv(TEXT, content).sz();
        }
        Content::Translatable {
            key,
            fallback,
            args,
        } => {
            w += Kv(TRANSLATE, key).sz();
            if let Some(fallback) = fallback {
                w += Kv(TRANSLATE_FALLBACK, fallback).sz();
            }
            let len = args.len();
            if len != 0 {
                w += Kv(TRANSLATE_WITH, ListInfo(COMPOUND, len as _)).sz();
                for arg in args {
                    w += arg.ty_sz();
                }
            }
        }
        Content::Score { name, objective } => {
            w += COMPOUND.sz();
            w += mutf8(SCORE).sz();
            w += Kv(SCORE_NAME, name).sz();
            w += Kv(SCORE_OBJECTIVE, objective).sz();
            w += END.sz();
        }
        Content::Selector { pattern, separator } => {
            w += Kv(SELECTOR, pattern).sz();
            if let Some(separator) = separator.as_deref() {
                w += Kv(SEPARATOR, separator).sz();
            }
        }
        Content::Keybind { keybind } => {
            w += Kv(KEYBIND, keybind).sz();
        }
        Content::Nbt {
            nbt_path,
            interpret,
            separator,
            content,
        } => {
            w += Kv(NBT_PATH, nbt_path).sz();
            if *interpret {
                w += Kv(NBT_INTERPRET, *interpret).sz();
            }
            if let Some(separator) = separator.as_deref() {
                w += Kv(SEPARATOR, separator).sz();
            }
            w += Kv(NBT_SOURCE, mutf8(nbt_type(content))).sz();
            match content {
                NbtContent::Block { pos } => {
                    w += Kv(NBT_BLOCK, pos).sz();
                }
                NbtContent::Entity { selector } => {
                    w += Kv(NBT_ENTITY, selector).sz();
                }
                NbtContent::Storage { storage } => {
                    w += Kv(NBT_STORAGE, storage).sz();
                }
            }
        }
        Content::Object { content } => {
            w += Kv(OBJECT_TYPE, mutf8(object_type(content))).sz();
            match content {
                ObjectContents::Atlas { atlas, sprite } => {
                    let atlas = atlas.as_ident();
                    if atlas != DEFAULT_ATLAS {
                        w += Kv(OBJECT_ATLAS, atlas).sz();
                    }
                    w += Kv(OBJECT_SPRITE, sprite.as_ident()).sz();
                }
                ObjectContents::Player { player, hat } => {
                    w += Kv(OBJECT_PLAYER, &**player).sz();
                    if !hat {
                        w += Kv(OBJECT_HAT, *hat).sz();
                    }
                }
            }
        }
    };

    let len = children.len();
    if len != 0 {
        w += Kv(EXTRA, ListInfo(COMPOUND, len as _)).sz();
        for child in children {
            w += child.ty_sz();
        }
    }

    if let Some(color) = style.color {
        let mut buf = [0; 7];
        w += Kv(COLOR, mutf8(color.name(&mut buf).as_bytes())).sz();
    }
    w += END.sz();

    w
}

fn read_rec_compound(buf: &mut &[u8]) -> Result<Component, Error> {
    let mut content: Option<ContentB> = None;
    let mut style = Style::new();
    let mut children = Vec::new();
    let mut separator: Option<Box<Component>> = None;
    loop {
        let t1 = TagType::read(buf)?;
        if t1 == END {
            let content = match content {
                Some(x) => match x.into_content(separator) {
                    Some(x) => x,
                    None => return Err(Error),
                },
                None => Content::Literal {
                    content: BoxStr::empty(),
                },
            };
            return Ok(Component {
                content,
                style,
                children,
            });
        }
        let name = match cast128(StringTagRaw::read(buf)?.inner()) {
            Some(x) => x,
            None => return Err(Error),
        };
        match name {
            TEXT_H => {
                let x = t1.expect_str(buf)?;
                match content.as_mut() {
                    None => content = Some(ContentB::Literal { content: x }),
                    Some(ContentB::Literal { content }) => *content = x,
                    _ => return Err(Error),
                }
            }
            TRANSLATE_H => {
                let x = t1.expect_str(buf)?;
                match content.as_mut() {
                    None => {
                        content = Some(ContentB::Translatable {
                            key: Some(x),
                            fallback: None,
                            args: Vec::new(),
                        })
                    }
                    Some(ContentB::Translatable {
                        key,
                        fallback: _,
                        args: _,
                    }) => *key = Some(x),
                    _ => return Err(Error),
                }
            }
            TRANSLATE_FALLBACK_H => {
                let x = t1.expect_str(buf)?;
                match content.as_mut() {
                    None => {
                        content = Some(ContentB::Translatable {
                            key: None,
                            fallback: Some(x),
                            args: Vec::new(),
                        })
                    }
                    Some(ContentB::Translatable {
                        key: _,
                        fallback,
                        args: _,
                    }) => *fallback = Some(x),
                    _ => return Err(Error),
                }
            }
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
                        content = Some(ContentB::Translatable {
                            key: None,
                            fallback: None,
                            args,
                        })
                    }
                    Some(ContentB::Translatable {
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
                                name = Some(t2.expect_str(buf)?);
                            }
                            SCORE_OBJECTIVE => {
                                objective = Some(t2.expect_str(buf)?);
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
                    None => content = Some(ContentB::Score { name, objective }),
                    Some(ContentB::Score {
                        name: x,
                        objective: y,
                    }) => {
                        *x = name;
                        *y = objective;
                    }
                    _ => return Err(Error),
                }
            }
            SELECTOR_H => {
                let x = t1.expect_str(buf)?;
                match content.as_mut() {
                    None => content = Some(ContentB::Selector { pattern: x }),
                    Some(ContentB::Selector { pattern }) => *pattern = x,
                    _ => return Err(Error),
                }
            }
            SEPARATOR_H => {
                separator = Some(Box::new(Component::read_ty(buf, t1)?));
            }
            KEYBIND_H => {
                let x = t1.expect_str(buf)?;
                match content.as_mut() {
                    None => content = Some(ContentB::Keybind { keybind: x }),
                    Some(ContentB::Keybind { keybind }) => *keybind = x,
                    _ => return Err(Error),
                }
            }
            NBT_PATH_H => {
                let x = t1.expect_str(buf)?;
                match content.as_mut() {
                    None => {
                        content = Some(ContentB::Nbt {
                            nbt_path: Some(x),
                            interpret: false,
                            content: None,
                        })
                    }
                    Some(ContentB::Nbt {
                        nbt_path,
                        interpret: _,
                        content: _,
                    }) => *nbt_path = Some(x),
                    _ => return Err(Error),
                }
            }
            NBT_INTERPRET_H => {
                let x = TagType::expect_bool(t1, buf)?;
                match content.as_mut() {
                    None => {
                        content = Some(ContentB::Nbt {
                            nbt_path: None,
                            interpret: x,
                            content: None,
                        })
                    }
                    Some(ContentB::Nbt {
                        nbt_path: _,
                        interpret,
                        content: _,
                    }) => *interpret = x,
                    _ => return Err(Error),
                }
            }
            NBT_SOURCE_H => match t1 {
                TagType::String => {
                    let _ = StringTagRaw::read(buf)?;
                }
                _ => return Err(Error),
            },
            NBT_BLOCK_H => {
                let x = t1.expect_str(buf)?;
                match content.as_mut() {
                    None => {
                        content = Some(ContentB::Nbt {
                            nbt_path: None,
                            interpret: false,
                            content: Some(NbtContent::Block { pos: x }),
                        })
                    }
                    Some(ContentB::Nbt {
                        nbt_path: _,
                        interpret: _,
                        content,
                    }) => *content = Some(NbtContent::Block { pos: x }),
                    _ => return Err(Error),
                }
            }
            NBT_ENTITY_H => {
                let x = t1.expect_str(buf)?;
                match content.as_mut() {
                    None => {
                        content = Some(ContentB::Nbt {
                            nbt_path: None,
                            interpret: false,
                            content: Some(NbtContent::Entity { selector: x }),
                        })
                    }
                    Some(ContentB::Nbt {
                        nbt_path: _,
                        interpret: _,
                        content,
                    }) => *content = Some(NbtContent::Entity { selector: x }),
                    _ => return Err(Error),
                }
            }
            NBT_STORAGE_H => {
                let x = t1.expect_ident(buf)?;
                match content.as_mut() {
                    None => {
                        content = Some(ContentB::Nbt {
                            nbt_path: None,
                            interpret: false,
                            content: Some(NbtContent::Storage { storage: x }),
                        })
                    }
                    Some(ContentB::Nbt {
                        nbt_path: _,
                        interpret: _,
                        content,
                    }) => *content = Some(NbtContent::Storage { storage: x }),
                    _ => return Err(Error),
                }
            }
            OBJECT_TYPE_H => match t1 {
                TagType::String => {
                    let _ = StringTagRaw::read(buf)?;
                }
                _ => return Err(Error),
            },
            OBJECT_ATLAS_H => {
                let x = t1.expect_ident(buf)?;
                match content.as_mut() {
                    None => {
                        content = Some(ContentB::Object {
                            content: ObjectContentB::Atlas {
                                atlas: Some(x),
                                sprite: None,
                            },
                        })
                    }
                    Some(ContentB::Object {
                        content: ObjectContentB::Atlas { atlas, sprite: _ },
                    }) => *atlas = Some(x),
                    _ => return Err(Error),
                }
            }
            OBJECT_SPRITE_H => {
                let x = t1.expect_ident(buf)?;
                match content.as_mut() {
                    None => {
                        content = Some(ContentB::Object {
                            content: ObjectContentB::Atlas {
                                atlas: None,
                                sprite: Some(x),
                            },
                        })
                    }
                    Some(ContentB::Object {
                        content: ObjectContentB::Atlas { atlas: _, sprite },
                    }) => *sprite = Some(x),
                    _ => return Err(Error),
                }
            }
            OBJECT_PLAYER_H => {
                let x = match t1 {
                    TagType::Compound => Box::new(ResolvableProfile::read_ty(buf)?),
                    _ => return Err(Error),
                };
                match content.as_mut() {
                    None => {
                        content = Some(ContentB::Object {
                            content: ObjectContentB::Player {
                                player: Some(x),
                                hat: false,
                            },
                        })
                    }
                    Some(ContentB::Object {
                        content: ObjectContentB::Player { player, hat: _ },
                    }) => *player = Some(x),
                    _ => return Err(Error),
                }
            }
            OBJECT_HAT_H => {
                let x = t1.expect_bool(buf)?;
                match content.as_mut() {
                    None => {
                        content = Some(ContentB::Object {
                            content: ObjectContentB::Player {
                                player: None,
                                hat: x,
                            },
                        })
                    }
                    Some(ContentB::Object {
                        content: ObjectContentB::Player { player: _, hat },
                    }) => *hat = x,
                    _ => return Err(Error),
                }
            }
            TYPE_H => match t1 {
                TagType::String => {
                    let _ = StringTagRaw::read(buf)?;
                }
                _ => return Err(Error),
            },
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
                let color = t1.expect_str(buf)?;
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
        w += write_rec_len(content, style, children);
        w
    }
}

impl Component {
    pub fn read_ty(buf: &mut &[u8], ty: TagType) -> Result<Component, Error> {
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
                            content: BoxStr::empty(),
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
}

impl<A: Allocator> Component<A> {
    /// # Safety
    pub unsafe fn write_ty(&self, w: &mut UnsafeWriter) {
        unsafe {
            write_rec(&self.content, &self.style, &self.children, w);
        }
    }

    pub fn ty_sz(&self) -> usize {
        write_rec_len(&self.content, &self.style, &self.children)
    }
}

impl<'a> Read<'a> for Component {
    fn read(buf: &mut &'a [u8]) -> Result<Self, Error> {
        let ty = TagType::read(buf)?;
        Self::read_ty(buf, ty)
    }
}
