pub trait PacketType: mser::Write + for<'a> mser::Read<'a> + Clone + Copy + Eq {}

pub trait Id {
    type T: PacketType;

    const ID: Self::T;
}

impl PacketType for minecraft_data::clientbound_status {}
impl PacketType for minecraft_data::clientbound_configuration {}
impl PacketType for minecraft_data::clientbound_login {}
impl PacketType for minecraft_data::clientbound_play {}
impl PacketType for minecraft_data::serverbound_handshake {}
impl PacketType for minecraft_data::serverbound_status {}
impl PacketType for minecraft_data::serverbound_configuration {}
impl PacketType for minecraft_data::serverbound_login {}
impl PacketType for minecraft_data::serverbound_play {}

#[inline]
pub fn packet_id<R: PacketType, T: Id<T = R> + ?Sized>(_: &T) -> R {
    T::ID
}
