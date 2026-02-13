#![no_std]

use crate::nbt::Tag;
use crate::str::BoxStr;
use alloc::boxed::Box;
use alloc::vec::Vec;
use mser::{Error, Read, UnsafeWriter, V21, V32, Write};

// pub mod chat;
pub mod clientbound;
// pub mod dialog;
pub mod item;
pub mod nbt;
pub mod profile;
pub mod serverbound;
pub mod str;
pub mod types;

#[macro_use]
extern crate mser_macro;
extern crate alloc;

#[derive(Clone, Copy, Debug)]
pub enum ClientIntent {
    Status,
    Login,
    Transfer,
}

impl Write for ClientIntent {
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            w.write_byte(match self {
                Self::Status => 1,
                Self::Login => 2,
                Self::Transfer => 3,
            });
        }
    }

    fn len_s(&self) -> usize {
        1
    }
}

impl<'a> Read<'a> for ClientIntent {
    fn read(buf: &mut &'a [u8]) -> Result<Self, Error> {
        match V32::read(buf)?.0 {
            1 => Ok(Self::Status),
            2 => Ok(Self::Login),
            3 => Ok(Self::Transfer),
            _ => Err(Error),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Utf8<'a, const MAX: usize = 32767>(pub &'a str);

impl<'a, const MAX: usize> Write for Utf8<'a, MAX> {
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            V21(self.0.len() as u32).write(w);
            w.write(self.0.as_bytes());
        }
    }

    fn len_s(&self) -> usize {
        V21(self.0.len() as u32).len_s() + self.0.len()
    }
}

impl<'a, const MAX: usize> Read<'a> for Utf8<'a, MAX> {
    fn read(buf: &mut &'a [u8]) -> Result<Self, Error> {
        let len = V21::read(buf)?.0 as usize;
        if len > MAX * 3 {
            return Err(Error);
        }
        let bytes = match buf.split_at_checked(len) {
            Some((x, y)) => {
                *buf = y;
                x
            }
            None => return Err(Error),
        };
        let s = match core::str::from_utf8(bytes) {
            Ok(x) => x,
            Err(_) => return Err(Error),
        };
        if s.chars().map(|x| x.len_utf16()).sum::<usize>() <= MAX {
            Ok(Utf8(s))
        } else {
            Err(Error)
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ByteArray<'a, const MAX: usize = { usize::MAX }>(pub &'a [u8]);

impl<'a, const MAX: usize> Write for ByteArray<'a, MAX> {
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            V21(self.0.len() as u32).write(w);
            w.write(self.0);
        }
    }

    fn len_s(&self) -> usize {
        V21(self.0.len() as u32).len_s() + self.0.len()
    }
}

impl<'a, const MAX: usize> Read<'a> for ByteArray<'a, MAX> {
    fn read(buf: &mut &'a [u8]) -> Result<Self, Error> {
        let len = V21::read(buf)?.0 as usize;
        if len > MAX {
            return Err(Error);
        }
        match buf.split_at_checked(len) {
            Some((x, y)) => {
                *buf = y;
                Ok(Self(x))
            }
            None => Err(Error),
        }
    }
}

#[derive(Clone, Debug)]
pub enum List<'a, T: 'a, const MAX: usize = { usize::MAX }> {
    Borrowed(&'a [T]),
    Ref(Box<[T]>),
}

impl<'a, T: Write + 'a, const MAX: usize> Write for List<'a, T, MAX> {
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            let x = match self {
                Self::Borrowed(x) => x,
                Self::Ref(x) => &x[..],
            };
            V21(x.len() as u32).write(w);
            for y in x {
                y.write(w);
            }
        }
    }

    fn len_s(&self) -> usize {
        let x = match self {
            Self::Borrowed(x) => x,
            Self::Ref(x) => &x[..],
        };
        let mut len = V21(x.len() as u32).len_s();
        for y in x {
            len += y.len_s();
        }
        len
    }
}

impl<'a, T: Read<'a> + 'a, const MAX: usize> Read<'a> for List<'a, T, MAX> {
    fn read(buf: &mut &'a [u8]) -> Result<Self, Error> {
        let len = V21::read(buf)?.0 as usize;
        if len > MAX {
            return Err(Error);
        }
        let mut vec = Vec::with_capacity(usize::min(len, 65536));
        for _ in 0..len {
            vec.push(T::read(buf)?);
        }
        Ok(List::Ref(vec.into_boxed_slice()))
    }
}

#[derive(Clone, Copy)]
pub struct Rest<'a, const MAX: usize = { usize::MAX }>(pub &'a [u8]);

impl<'a, const MAX: usize> Read<'a> for Rest<'a, MAX> {
    fn read(buf: &mut &'a [u8]) -> Result<Self, Error> {
        let len = buf.len();
        if len > MAX {
            return Err(Error);
        }
        match buf.split_at_checked(len) {
            Some((x, y)) => {
                *buf = y;
                Ok(Self(x))
            }
            None => Err(Error),
        }
    }
}

impl<'a, const MAX: usize> Write for Rest<'a, MAX> {
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe { w.write(self.0) }
    }

    fn len_s(&self) -> usize {
        self.0.len()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Ident<'a> {
    pub namespace: &'a str,
    pub path: &'a str,
}

impl<'a> Ident<'a> {
    pub const MINECRAFT: &'static str = "minecraft";

    pub fn is_valid_path(c: char) -> bool {
        matches!(c, 'a'..='z' | '0'..='9' | '_' | '-' | '.' | '/')
    }

    pub fn is_valid_namespace(c: char) -> bool {
        matches!(c, 'a'..='z' | '0'..='9' | '_' | '-' | '.')
    }

    pub fn parse(identifier: &'a str) -> Option<Self> {
        match identifier.strip_prefix("minecraft:") {
            Some(path) => {
                if path.chars().all(Self::is_valid_path) {
                    Some(Self {
                        namespace: Self::MINECRAFT,
                        path,
                    })
                } else {
                    None
                }
            }
            None => match identifier.split_once(':') {
                Some((namespace, path)) => {
                    if namespace.chars().all(Self::is_valid_namespace)
                        && path.chars().all(Self::is_valid_path)
                    {
                        Some(Self {
                            namespace: if !namespace.is_empty() {
                                namespace
                            } else {
                                Self::MINECRAFT
                            },
                            path,
                        })
                    } else {
                        None
                    }
                }
                None => {
                    if identifier.chars().all(Self::is_valid_path) {
                        Some(Self {
                            namespace: Self::MINECRAFT,
                            path: identifier,
                        })
                    } else {
                        None
                    }
                }
            },
        }
    }
}

impl<'a> Read<'a> for Ident<'a> {
    fn read(buf: &mut &'a [u8]) -> Result<Self, Error> {
        let identifier = Utf8::<32767>::read(buf)?.0;
        match Self::parse(identifier) {
            Some(x) => Ok(x),
            None => Err(Error),
        }
    }
}

impl Write for Ident<'_> {
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            V21((self.namespace.len() + 1 + self.path.len()) as _).write(w);
            w.write(self.namespace.as_bytes());
            w.write_byte(b':');
            w.write(self.path.as_bytes());
        }
    }

    fn len_s(&self) -> usize {
        let a = self.namespace.len() + 1 + self.path.len();
        V21(a as u32).len_s() + a
    }
}

#[derive(Clone)]
pub struct Identifier {
    pub namespace: Option<BoxStr>,
    pub path: BoxStr,
}

impl Identifier {
    pub fn as_ident(&self) -> Ident<'_> {
        Ident {
            namespace: match self.namespace.as_deref() {
                Some(x) => x,
                None => Ident::MINECRAFT,
            },
            path: &self.path,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Component(pub Tag);

#[derive(Clone)]
pub struct ResourceKey {
    pub registry_name: Identifier,
    pub identifier: Identifier,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RegistryKey<'a> {
    pub identifier: Ident<'a>,
}

#[derive(Clone)]
pub struct TagKey {
    pub registry: ResourceKey,
    pub location: Identifier,
}

#[derive(Clone)]
pub enum HolderSet<T> {
    Direct(Vec<Holder<T>>),
    Named(TagKey),
}

#[derive(Clone)]
pub enum Holder<T> {
    Direct(T),
    Reference(ResourceKey),
}

pub fn json_escaped_string(s: &str, w: &mut Vec<u8>) {
    let mut start = 0;
    let mut cur = 0;
    let n = s.as_bytes();

    while let Some(&byte) = n.get(cur) {
        let esc = mser::json_char_width_escaped(byte);
        if esc <= 4 {
            cur += esc as usize;
            continue;
        }
        w.extend(unsafe { n.get_unchecked(start..cur) });
        if esc == 0xff {
            let (d1, d2) = mser::u8_to_hex(byte);
            w.extend(&[b'\\', b'u', b'0', b'0', d1, d2]);
        } else {
            w.extend(&[b'\\', esc]);
        }
        cur += 1;
        start = cur;
    }
    w.extend(unsafe { n.get_unchecked(start..) });
}

#[test]
fn test_write() {
    use crate::clientbound::login::LoginFinished;
    use crate::profile::GameProfileRef;
    use crate::types::Id as _;
    use minecraft_data::clientbound__login;
    use uuid::Uuid;

    let packet: LoginFinished = LoginFinished {
        game_profile: GameProfileRef {
            id: Uuid::nil(),
            name: Utf8("abc"),
            peoperties: List::Borrowed(&[]),
        },
    };

    let id = LoginFinished::ID;
    let len1 = id.len_s();
    let len2 = packet.len_s() + len1;
    let data = unsafe {
        let mut data = alloc::vec::Vec::with_capacity(len2);
        mser::write_unchecked(data.as_mut_ptr(), &id);
        mser::write_unchecked(data.as_mut_ptr().add(len1), &packet);
        data.set_len(len2);
        data.into_boxed_slice()
    };
    let mut data = &data[..];
    let id = V32::read(&mut data).unwrap().0;
    assert_eq!(clientbound__login::new(id as _).unwrap(), LoginFinished::ID);
    assert_eq!(Uuid::read(&mut data).unwrap(), Uuid::nil());
    assert_eq!(Utf8::<16>::read(&mut data).unwrap().0, "abc");
    assert_eq!(V32::read(&mut data).unwrap().0, 0);
    assert!(data.is_empty());
}
