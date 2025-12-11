#![no_std]
#![cfg_attr(not(feature = "allocator-api2"), feature(allocator_api))]

use crate::str::SmolStr;
use alloc::alloc::{Allocator, Global};
use alloc::boxed::Box;
use alloc::vec::Vec;
use mser::{Bytes, Error, Read, UnsafeWriter, Write, V21, V32};
use uuid::Uuid;

pub mod chat;
pub mod clientbound;
pub mod dialog;
pub mod item;
pub mod nbt;
pub mod profile;
pub mod serverbound;
pub mod str;
pub mod types;

#[macro_use]
extern crate mser_macro;

#[cfg(not(feature = "allocator-api2"))]
extern crate alloc;
#[cfg(feature = "allocator-api2")]
extern crate allocator_api2 as alloc;

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

    fn sz(&self) -> usize {
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

    fn sz(&self) -> usize {
        V21(self.0.len() as u32).sz() + self.0.len()
    }
}

impl<'a, const MAX: usize> Read<'a> for Utf8<'a, MAX> {
    fn read(buf: &mut &'a [u8]) -> Result<Self, Error> {
        let len = V21::read(buf)?.0 as usize;
        if len > MAX * 3 {
            return Err(Error);
        }
        let bytes = buf.slice(len)?;
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

    fn sz(&self) -> usize {
        V21(self.0.len() as u32).sz() + self.0.len()
    }
}

impl<'a, const MAX: usize> Read<'a> for ByteArray<'a, MAX> {
    fn read(buf: &mut &'a [u8]) -> Result<Self, Error> {
        let len = V21::read(buf)?.0 as usize;
        if len > MAX {
            return Err(Error);
        }
        let bytes = buf.slice(len)?;
        Ok(ByteArray(bytes))
    }
}

#[derive(Clone, Debug)]
pub enum List<'a, T: 'a, A: Allocator = Global, const MAX: usize = { usize::MAX }>
where
    A: 'a,
{
    Borrowed(&'a [T]),
    Ref(Box<[T], A>),
}

impl<'a, T: Write + 'a, A: Allocator, const MAX: usize> Write for List<'a, T, A, MAX> {
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

    fn sz(&self) -> usize {
        let x = match self {
            Self::Borrowed(x) => x,
            Self::Ref(x) => &x[..],
        };
        let mut len = V21(x.len() as u32).sz();
        for y in x {
            len += y.sz();
        }
        len
    }
}

impl<'a, T: Read<'a> + 'a, const MAX: usize> Read<'a> for List<'a, T, Global, MAX> {
    fn read(buf: &mut &'a [u8]) -> Result<Self, Error> {
        let len = V21::read(buf)?.0 as usize;
        if len > MAX {
            return Err(Error);
        }
        let mut vec = Vec::with_capacity_in(usize::min(len, 65536), Global);
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
        Ok(Rest(buf.slice(buf.len())?))
    }
}

impl<'a, const MAX: usize> Write for Rest<'a, MAX> {
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        w.write(self.0)
    }

    fn sz(&self) -> usize {
        self.0.len()
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct GameProfile<'a, A: Allocator = Global> {
    pub id: Uuid,
    pub name: Utf8<'a, 16>,
    pub peoperties: List<'a, PropertyMap<'a>, A, 16>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PropertyMap<'a> {
    pub name: Utf8<'a, 64>,
    pub value: Utf8<'a, 32767>,
    pub signature: Option<Utf8<'a, 1024>>,
}

#[derive(Clone)]
pub struct Ident<'a> {
    pub namespace: &'a str,
    pub path: &'a str,
}

impl Ident<'_> {
    pub const MINECRAFT: &'static str = "minecraft";

    pub fn is_valid_path(c: char) -> bool {
        matches!(c, 'a'..='z' | '0'..='9' | '_' | '-' | '.' | '/')
    }

    pub fn is_valid_namespace(c: char) -> bool {
        matches!(c, 'a'..='z' | '0'..='9' | '_' | '-' | '.')
    }
}

impl<'a> Read<'a> for Ident<'a> {
    fn read(buf: &mut &'a [u8]) -> Result<Self, Error> {
        let identifier = Utf8::<32767>::read(buf)?.0;
        match identifier.strip_prefix("minecraft:") {
            Some(path) => {
                if path.chars().all(Self::is_valid_path) {
                    Ok(Self {
                        namespace: Self::MINECRAFT,
                        path,
                    })
                } else {
                    Err(Error)
                }
            }
            None => match identifier.split_once(':') {
                Some((namespace, path)) => {
                    if namespace.chars().all(Self::is_valid_namespace)
                        && path.chars().all(Self::is_valid_path)
                    {
                        Ok(Self {
                            namespace: if !namespace.is_empty() {
                                namespace
                            } else {
                                Self::MINECRAFT
                            },
                            path,
                        })
                    } else {
                        Err(Error)
                    }
                }
                None => {
                    if identifier.chars().all(Self::is_valid_path) {
                        Ok(Self {
                            namespace: Self::MINECRAFT,
                            path: identifier,
                        })
                    } else {
                        Err(Error)
                    }
                }
            },
        }
    }
}

impl Write for Ident<'_> {
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            w.write(self.namespace.as_bytes());
            w.write_byte(b':');
            w.write(self.path.as_bytes());
        }
    }

    fn sz(&self) -> usize {
        self.namespace.len() + 1 + self.path.len()
    }
}

#[derive(Clone)]
pub struct Identifier<A: Allocator = Global> {
    pub namespace: SmolStr<A>,
    pub path: SmolStr<A>,
}

#[derive(Clone)]
pub struct ResourceKey<A: Allocator = Global> {
    pub registry_name: Identifier<A>,
    pub identifier: Identifier<A>,
}

#[derive(Clone)]
pub struct TagKey<A: Allocator = Global> {
    pub registry: ResourceKey<A>,
    pub location: Identifier<A>,
}

#[derive(Clone)]
pub enum HolderSet<T, A: Allocator = Global> {
    Direct(Vec<Holder<T, A>, A>),
    Named(TagKey<A>),
}

#[derive(Clone)]
pub enum Holder<T, A: Allocator = Global> {
    Direct(T),
    Reference(ResourceKey<A>),
}

#[test]
fn test_write() {
    use crate::clientbound::login::LoginFinished;
    use crate::types::{Id, Packet};
    use minecraft_data::clientbound__login;

    let packet: LoginFinished<'_, Global> = LoginFinished {
        game_profile: GameProfile {
            id: Uuid::nil(),
            name: Utf8("abc"),
            peoperties: List::Borrowed(&[]),
        },
    };
    let packet = Packet::new(packet);
    let len = packet.sz();
    let data = unsafe {
        let mut data = alloc::vec::Vec::with_capacity(len);
        packet.write(&mut mser::UnsafeWriter::new(data.as_mut_ptr()));
        data.set_len(len);
        data.into_boxed_slice()
    };
    let mut data = &data[..];
    let id = data.v32().unwrap();
    assert_eq!(
        clientbound__login::new(id as _).unwrap(),
        LoginFinished::<'_, Global>::ID
    );
    assert_eq!(Uuid::read(&mut data).unwrap(), Uuid::nil());
    assert_eq!(Utf8::<16>::read(&mut data).unwrap().0, "abc");
    assert_eq!(data.v32().unwrap(), 0);
    assert!(data.is_empty());
}
