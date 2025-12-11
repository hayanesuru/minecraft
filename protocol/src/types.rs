use minecraft_data::{
    clientbound__configuration, clientbound__login, clientbound__play, clientbound__status,
    serverbound__configuration, serverbound__handshake, serverbound__login, serverbound__play,
    serverbound__status,
};
use mser::{Error, Read, UnsafeWriter, Write};

pub trait PacketId: Write + for<'a> Read<'a> + Clone + Copy + Eq {}

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
        if id == T::ID {
            Ok(Packet(T::read(buf)?, core::marker::PhantomData))
        } else {
            Err(Error)
        }
    }
}

impl<'a, I: PacketId, T: Id<I> + 'a + Write> Write for Packet<'a, I, T> {
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            T::ID.write(w);
            self.0.write(w);
        }
    }

    fn sz(&self) -> usize {
        T::ID.sz() + self.0.sz()
    }
}

pub trait Id<T: PacketId> {
    const ID: T;
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
