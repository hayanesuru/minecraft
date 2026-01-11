use super::*;
use crate::nbt::{Kv, ListInfo, MapCodec, MapReader, RefStringTag, StringTag, TagType, read_map};
use mser::{Error, Read, UnsafeWriter, Write};

const STRING: TagType = TagType::String;
const LIST: TagType = TagType::List;
const COMPOUND: TagType = TagType::Compound;
const END: TagType = TagType::End;

const fn content_type(content: &Content) -> &'static [u8] {
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

const fn nbt_type(content: &NbtContent) -> &'static [u8] {
    match content {
        NbtContent::Block { .. } => b"block",
        NbtContent::Entity { .. } => b"entity",
        NbtContent::Storage { .. } => b"storage",
    }
}

const fn object_type(content: &ObjectContents) -> &'static [u8] {
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

unsafe fn write_rec(
    content: &Content,
    style: &Style,
    children: &[Component],
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
                    arg.write_kv(w);
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
                child.write_kv(w);
            }
        }

        if let Some(color) = style.color {
            let mut buf = [0; 7];
            Kv(COLOR, mutf8(color.name(&mut buf).as_bytes())).write(w);
        }

        END.write(w);
    }
}

fn write_rec_len(content: &Content, style: &Style, children: &[Component]) -> usize {
    let mut w = 0usize;
    w += Kv(TYPE, mutf8(content_type(content))).len_s();
    match content {
        Content::Literal { content } => {
            w += Kv(TEXT, content).len_s();
        }
        Content::Translatable {
            key,
            fallback,
            args,
        } => {
            w += Kv(TRANSLATE, key).len_s();
            if let Some(fallback) = fallback {
                w += Kv(TRANSLATE_FALLBACK, fallback).len_s();
            }
            let len = args.len();
            if len != 0 {
                w += Kv(TRANSLATE_WITH, ListInfo(COMPOUND, len as _)).len_s();
                for arg in args {
                    w += arg.len_kv();
                }
            }
        }
        Content::Score { name, objective } => {
            w += COMPOUND.len_s();
            w += mutf8(SCORE).len_s();
            w += Kv(SCORE_NAME, name).len_s();
            w += Kv(SCORE_OBJECTIVE, objective).len_s();
            w += END.len_s();
        }
        Content::Selector { pattern, separator } => {
            w += Kv(SELECTOR, pattern).len_s();
            if let Some(separator) = separator.as_deref() {
                w += Kv(SEPARATOR, separator).len_s();
            }
        }
        Content::Keybind { keybind } => {
            w += Kv(KEYBIND, keybind).len_s();
        }
        Content::Nbt {
            nbt_path,
            interpret,
            separator,
            content,
        } => {
            w += Kv(NBT_PATH, nbt_path).len_s();
            if *interpret {
                w += Kv(NBT_INTERPRET, *interpret).len_s();
            }
            if let Some(separator) = separator.as_deref() {
                w += Kv(SEPARATOR, separator).len_s();
            }
            w += Kv(NBT_SOURCE, mutf8(nbt_type(content))).len_s();
            match content {
                NbtContent::Block { pos } => {
                    w += Kv(NBT_BLOCK, pos).len_s();
                }
                NbtContent::Entity { selector } => {
                    w += Kv(NBT_ENTITY, selector).len_s();
                }
                NbtContent::Storage { storage } => {
                    w += Kv(NBT_STORAGE, storage).len_s();
                }
            }
        }
        Content::Object { content } => {
            w += Kv(OBJECT_TYPE, mutf8(object_type(content))).len_s();
            match content {
                ObjectContents::Atlas { atlas, sprite } => {
                    let atlas = atlas.as_ident();
                    if atlas != DEFAULT_ATLAS {
                        w += Kv(OBJECT_ATLAS, atlas).len_s();
                    }
                    w += Kv(OBJECT_SPRITE, sprite.as_ident()).len_s();
                }
                ObjectContents::Player { player, hat } => {
                    w += Kv(OBJECT_PLAYER, &**player).len_s();
                    if !hat {
                        w += Kv(OBJECT_HAT, *hat).len_s();
                    }
                }
            }
        }
    };

    let len = children.len();
    if len != 0 {
        w += Kv(EXTRA, ListInfo(COMPOUND, len as _)).len_s();
        for child in children {
            w += child.len_kv();
        }
    }

    if let Some(color) = style.color {
        let mut buf = [0; 7];
        w += Kv(COLOR, mutf8(color.name(&mut buf).as_bytes())).len_s();
    }
    w += END.len_s();

    w
}

impl Write for Component {
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

    fn len_s(&self) -> usize {
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
            w += RefStringTag(content).len_s();
            return w;
        }
        w += COMPOUND.len_s();
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
                        children.push(Self::read_kv(buf)?);
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
            COMPOUND => Self::read_kv(buf),
            _ => Err(Error),
        }
    }
}

struct Reader {
    content: Option<ContentB>,
    style: Style,
    children: Vec<Component>,
    separator: Option<Box<Component>>,
}

impl MapReader<Component> for Reader {
    fn visit(&mut self, ty: TagType, k: &str, buf: &mut &[u8]) -> Result<(), Error> {
        match cast128(k.as_bytes())? {
            TEXT_H => {
                let x = ty.string(buf)?;
                match self.content.as_mut() {
                    None => self.content = Some(ContentB::Literal { content: x }),
                    Some(ContentB::Literal { content }) => *content = x,
                    _ => return Err(Error),
                }
            }
            TRANSLATE_H => {
                let x = ty.string(buf)?;
                match self.content.as_mut() {
                    None => {
                        self.content = Some(ContentB::Translatable {
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
                let x = ty.string(buf)?;
                match self.content.as_mut() {
                    None => {
                        self.content = Some(ContentB::Translatable {
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
                match ty {
                    LIST => match ListInfo::read(buf)? {
                        ListInfo(COMPOUND, len) => {
                            for _ in 0..len {
                                args.push(Component::read_kv(buf)?);
                            }
                        }
                        _ => return Err(Error),
                    },
                    _ => return Err(Error),
                };
                match self.content.as_mut() {
                    None => {
                        self.content = Some(ContentB::Translatable {
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
                match ty {
                    COMPOUND => loop {
                        let t2 = TagType::read(buf)?;
                        if t2 == END {
                            break;
                        }
                        match StringTag::read(buf)?.0.as_bytes() {
                            SCORE_NAME => {
                                name = Some(t2.string(buf)?);
                            }
                            SCORE_OBJECTIVE => {
                                objective = Some(t2.string(buf)?);
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
                match self.content.as_mut() {
                    None => self.content = Some(ContentB::Score { name, objective }),
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
                let x = ty.string(buf)?;
                match self.content.as_mut() {
                    None => self.content = Some(ContentB::Selector { pattern: x }),
                    Some(ContentB::Selector { pattern }) => *pattern = x,
                    _ => return Err(Error),
                }
            }
            SEPARATOR_H => {
                self.separator = Some(Box::new(Component::read_ty(buf, ty)?));
            }
            KEYBIND_H => {
                let x = ty.string(buf)?;
                match self.content.as_mut() {
                    None => self.content = Some(ContentB::Keybind { keybind: x }),
                    Some(ContentB::Keybind { keybind }) => *keybind = x,
                    _ => return Err(Error),
                }
            }
            NBT_PATH_H => {
                let x = ty.string(buf)?;
                match self.content.as_mut() {
                    None => {
                        self.content = Some(ContentB::Nbt {
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
                let x = ty.bool(buf)?;
                match self.content.as_mut() {
                    None => {
                        self.content = Some(ContentB::Nbt {
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
            NBT_SOURCE_H => match ty {
                TagType::String => {
                    let _ = StringTagRaw::read(buf)?;
                }
                _ => return Err(Error),
            },
            NBT_BLOCK_H => {
                let x = ty.string(buf)?;
                match self.content.as_mut() {
                    None => {
                        self.content = Some(ContentB::Nbt {
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
                let x = ty.string(buf)?;
                match self.content.as_mut() {
                    None => {
                        self.content = Some(ContentB::Nbt {
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
                let x = ty.ident(buf)?;
                match self.content.as_mut() {
                    None => {
                        self.content = Some(ContentB::Nbt {
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
            OBJECT_TYPE_H => match ty {
                TagType::String => {
                    let _ = StringTagRaw::read(buf)?;
                }
                _ => return Err(Error),
            },
            OBJECT_ATLAS_H => {
                let x = ty.ident(buf)?;
                match self.content.as_mut() {
                    None => {
                        self.content = Some(ContentB::Object {
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
                let x = ty.ident(buf)?;
                match self.content.as_mut() {
                    None => {
                        self.content = Some(ContentB::Object {
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
                let x = match ty {
                    TagType::Compound => Box::new(ResolvableProfile::read_kv(buf)?),
                    _ => return Err(Error),
                };
                match self.content.as_mut() {
                    None => {
                        self.content = Some(ContentB::Object {
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
                let x = ty.bool(buf)?;
                match self.content.as_mut() {
                    None => {
                        self.content = Some(ContentB::Object {
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
            TYPE_H => match ty {
                TagType::String => {
                    let _ = StringTagRaw::read(buf)?;
                }
                _ => return Err(Error),
            },
            EXTRA_H => match ty {
                LIST => match ListInfo::read(buf)? {
                    ListInfo(COMPOUND, len) => {
                        for _ in 0..len {
                            self.children.push(Component::read_kv(buf)?);
                        }
                    }
                    _ => return Err(Error),
                },
                _ => return Err(Error),
            },
            COLOR_H => {
                let color = ty.string(buf)?;
                self.style.color = match TextColor::parse(color.as_bytes()) {
                    Some(x) => Some(x),
                    None => return Err(Error),
                };
            }
            _ => return Err(Error),
        }
        Ok(())
    }

    fn end(self) -> Result<Component, Error> {
        Ok(Component {
            content: match self.content {
                Some(x) => match x.into_content(self.separator) {
                    Some(x) => x,
                    None => return Err(Error),
                },
                None => Content::Literal {
                    content: BoxStr::empty(),
                },
            },
            style: self.style,
            children: self.children,
        })
    }
}

impl MapCodec for Component {
    fn read_kv(buf: &mut &[u8]) -> Result<Self, Error> {
        read_map(
            Reader {
                content: None,
                style: Style::new(),
                children: Vec::new(),
                separator: None,
            },
            buf,
        )
    }

    unsafe fn write_kv(&self, w: &mut UnsafeWriter) {
        unsafe {
            write_rec(&self.content, &self.style, &self.children, w);
        }
    }

    fn len_kv(&self) -> usize {
        write_rec_len(&self.content, &self.style, &self.children)
    }
}

impl<'a> Read<'a> for Component {
    fn read(buf: &mut &'a [u8]) -> Result<Self, Error> {
        let ty = TagType::read(buf)?;
        Self::read_ty(buf, ty)
    }
}
