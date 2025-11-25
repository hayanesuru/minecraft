use minecraft_data::{serverbound__handshake, serverbound__status};

pub mod handshake;
pub mod status;

macro_rules! packets {
    ($m:ty, $($type:ty => $variant:path),+ $(,)?) => {
        $(
            #[automatically_derived]
            impl crate::Id<$m> for $type {
                fn id() -> $m {
                    $variant
                }
            }
        )+
    };
}

packets! {
    serverbound__handshake,
    handshake::Intention<'_> => serverbound__handshake::intention
}
packets! {
    serverbound__status,
    status::StatusRequest => serverbound__status::status_request,
    status::PingRequest => serverbound__status::ping_request,
}
