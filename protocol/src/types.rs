pub trait Packet: mser::Write + for<'a> mser::Read<'a> + Clone + Copy + Eq {}

pub trait Id<T: Packet> {
    const ID: T;
}

impl Packet for minecraft_data::clientbound__status {}
impl Packet for minecraft_data::clientbound__configuration {}
impl Packet for minecraft_data::clientbound__login {}
impl Packet for minecraft_data::clientbound__play {}
impl Packet for minecraft_data::serverbound__handshake {}
impl Packet for minecraft_data::serverbound__status {}
impl Packet for minecraft_data::serverbound__configuration {}
impl Packet for minecraft_data::serverbound__login {}
impl Packet for minecraft_data::serverbound__play {}

pub fn packet_id<R: Packet, T: Id<R>>(_: &T) -> R {
    T::ID
}
