pub trait PacketType: mser::Write + for<'a> mser::Read<'a> + Clone + Copy + Eq {}

pub trait Id {
    type T: PacketType;

    const ID: Self::T;
}

impl PacketType for minecraft_data::clientbound__status {}
impl PacketType for minecraft_data::clientbound__configuration {}
impl PacketType for minecraft_data::clientbound__login {}
impl PacketType for minecraft_data::clientbound__play {}
impl PacketType for minecraft_data::serverbound__handshake {}
impl PacketType for minecraft_data::serverbound__status {}
impl PacketType for minecraft_data::serverbound__configuration {}
impl PacketType for minecraft_data::serverbound__login {}
impl PacketType for minecraft_data::serverbound__play {}

pub fn packet_id<R: PacketType, T: Id<T = R> + ?Sized>(_: &T) -> R {
    T::ID
}
