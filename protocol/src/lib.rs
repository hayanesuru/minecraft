#![no_std]

use minecraft_data::{
    clientbound__configuration, clientbound__login, clientbound__play, clientbound__status,
    serverbound__configuration, serverbound__handshake, serverbound__login, serverbound__play,
    serverbound__status,
};
use mser::{Bytes, Error, Read, Write, V21};
use uuid::Uuid;

pub mod clientbound;
pub mod serverbound;

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
    unsafe fn write(&self, w: &mut mser::UnsafeWriter) {
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
    unsafe fn write(&self, w: &mut mser::UnsafeWriter) {
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
    unsafe fn write(&self, w: &mut mser::UnsafeWriter) {
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

#[derive(Clone, Serialize, Deserialize)]
pub struct GameProfile<'a> {
    pub id: Uuid,
    pub name: Utf8<'a, 16>,
}

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct Packet<'a, I: PacketId, T: Id<I> + 'a>(pub T, core::marker::PhantomData<&'a I>);

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
    unsafe fn write(&self, w: &mut mser::UnsafeWriter) {
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
