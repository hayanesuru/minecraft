use minecraft_data::{serverbound__handshake, serverbound__login, serverbound__status};

pub mod common;
pub mod configuration;
pub mod cookie;
pub mod handshake;
pub mod login;
pub mod ping;
pub mod status;

macro_rules! packets {
    ($m:ty, $handler:ident, $($variant:ident = $type:ty),+ $(,)*) => {
        $(
        #[automatically_derived]
        impl crate::types::Id for $type {
            type T = $m;
            const ID: $m = <$m>::$variant;
        }
        )+

        pub trait $handler {
            fn handle(&mut self, mut packet: &[u8]) -> Result<(), mser::Error> {
                match <$m as mser::Read>::read(&mut packet)? {
                    $(
                        <$m>::$variant => {
                            let e = <$type as mser::Read>::read(&mut packet)?;
                            if !packet.is_empty() {
                                mser::cold_path();
                                return Err(mser::Error);
                            }
                            self.$variant(e);
                        }
                    )+
                }
                Ok(())
            }
        $(
            fn $variant(&mut self, packet: $type);
        )+
        }
    };
}
packets! {
    serverbound__handshake,
    HandshakeHandler,
    intention = handshake::Intention<'_>,
}
packets! {
    serverbound__status,
    StatusHandler,
    status_request = status::StatusRequest,
    ping_request = ping::PingRequest,
}
packets! {
    serverbound__login,
    LoginHandler,
    hello = login::Hello<'_>,
    key = login::Key<'_>,
    custom_query_answer = login::CustomQueryAnswer<'_>,
    login_acknowledged = login::LoginAcknowledged,
    cookie_response = cookie::CookieResponse<'_>,
}
