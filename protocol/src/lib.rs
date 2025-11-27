#![no_std]
#![cfg_attr(not(feature = "allocator-api2"), feature(allocator_api))]

use alloc::{
    alloc::{Allocator, Global},
    boxed::Box,
    vec::Vec,
};
use minecraft_data::{
    clientbound__configuration, clientbound__login, clientbound__play, clientbound__status,
    serverbound__configuration, serverbound__handshake, serverbound__login, serverbound__play,
    serverbound__status,
};
use mser::{Bytes, Error, Read, UnsafeWriter, Write, V21};
use uuid::Uuid;

pub mod clientbound;
pub mod serverbound;

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
        match u8::read(buf)? {
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
    Owned(Box<[T], A>),
}

impl<'a, T: Write + 'a, A: Allocator, const MAX: usize> Write for List<'a, T, A, MAX> {
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            let x = match self {
                Self::Borrowed(x) => x,
                Self::Owned(x) => &x[..],
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
            Self::Owned(x) => &x[..],
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
        Ok(List::Owned(vec.into_boxed_slice()))
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
pub struct Identifier<'a> {
    pub namespace: &'a str,
    pub path: &'a str,
}

impl Identifier<'_> {
    pub fn is_valid_path(c: char) -> bool {
        matches!(c, 'a'..='z' | '0'..='9' | '_' | '-' | '.' | '/')
    }

    pub fn is_valid_namespace(c: char) -> bool {
        matches!(c, 'a'..='z' | '0'..='9' | '_' | '-' | '.')
    }
}

impl<'a> Read<'a> for Identifier<'a> {
    fn read(buf: &mut &'a [u8]) -> Result<Self, Error> {
        let identifier = Utf8::<32767>::read(buf)?.0;
        match identifier.strip_prefix("minecraft:") {
            Some(path) => {
                if path.chars().all(Self::is_valid_path) {
                    Ok(Self {
                        namespace: "minecraft",
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
                                "minecraft"
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
                            namespace: "minecraft",
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

impl Write for Identifier<'_> {
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

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct Packet<'a, I: PacketId, T: Id<I> + 'a>(pub T, core::marker::PhantomData<&'a I>);

impl<'a, I: PacketId, T: Id<I> + 'a> Packet<'a, I, T> {
    pub const fn new(packet: T) -> Self {
        Packet(packet, core::marker::PhantomData)
    }
}

impl<'a, I: PacketId, T: Id<I> + 'a + Read<'a>> Read<'a> for Packet<'a, I, T> {
    fn read(buf: &mut &'a [u8]) -> Result<Self, Error> {
        let id = I::read(buf)?;
        if id == T::id() {
            Ok(Packet(T::read(buf)?, core::marker::PhantomData))
        } else {
            Err(Error)
        }
    }
}

impl<'a, I: PacketId, T: Id<I> + 'a + Write> Write for Packet<'a, I, T> {
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            T::id().write(w);
            self.0.write(w);
        }
    }

    fn sz(&self) -> usize {
        T::id().sz() + self.0.sz()
    }
}

pub trait PacketId: Write + for<'a> Read<'a> + Clone + Copy + Eq {}

pub trait Id<T: PacketId> {
    fn id() -> T;
}

impl PacketId for clientbound__status {}
impl PacketId for clientbound__configuration {}
impl PacketId for clientbound__login {}
impl PacketId for clientbound__play {}
impl PacketId for serverbound__handshake {}
impl PacketId for serverbound__status {}
impl PacketId for serverbound__configuration {}
impl PacketId for serverbound__login {}
impl PacketId for serverbound__play {}

#[test]
fn test_write() {
    use crate::clientbound::login::LoginFinished;
    let packet: LoginFinished<'_, Global> = LoginFinished {
        game_profile: GameProfile {
            id: Uuid::nil(),
            name: Utf8("abc"),
            peoperties: List::Borrowed(&[]),
        },
    };
    let packet = Packet::new(packet);
    let data = mser::boxed(&packet);
    let mut data = &data[..];
    let id = data.v32().unwrap();
    assert_eq!(
        clientbound__login::new(id as _).unwrap(),
        LoginFinished::<'_, Global>::id()
    );
    assert_eq!(Uuid::read(&mut data).unwrap(), Uuid::nil());
    assert_eq!(Utf8::<16>::read(&mut data).unwrap().0, "abc");
    assert_eq!(data.v32().unwrap(), 0);
    assert!(data.is_empty());
}
