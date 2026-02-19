use minecraft_data::{
    serverbound__configuration, serverbound__handshake, serverbound__login, serverbound__status,
};

pub mod common;
pub mod configuration;
pub mod cookie;
pub mod handshake;
pub mod login;
pub mod ping;
pub mod status;

macro_rules! packets {
    ($m:ty, $handler:ident, $handle:ident, $($variant:ident = $type:ty),+ $(,)*) => {
        $(
        #[automatically_derived]
        impl crate::types::Id for $type {
            type T = $m;
            const ID: $m = <$m>::$variant;
        }
        )+

        pub trait $handler {
            fn $handle(&mut self, mut packet: &[u8]) -> Result<(), mser::Error> {
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
    handle,
    intention = handshake::Intention<'_>,
}
packets! {
    serverbound__status,
    StatusHandler,
    handle,
    status_request = status::StatusRequest,
    ping_request = ping::PingRequest,
}
packets! {
    serverbound__login,
    LoginHandler,
    handle,
    hello = login::Hello<'_>,
    key = login::Key<'_>,
    custom_query_answer = login::CustomQueryAnswer<'_>,
    login_acknowledged = login::LoginAcknowledged,
    cookie_response = cookie::CookieResponse<'_>,
}
packets! {
    serverbound__configuration,
    ConfigurationHandler,
    handle,
    client_information = common::ConfigurationClientInformation<'_>,
    cookie_response = cookie::ConfigurationCookieResponse<'_>,
    custom_payload = common::CustomPayload<'_>,
    finish_configuration = configuration::FinishConfiguration,
    keep_alive = common::KeepAlive,
    pong = common::Pong,
    resource_pack = common::ResourcePack,
    select_known_packs = configuration::SelectKnownPacks<'_>,
    custom_click_action = common::CustomClickAction<'_>,
    accept_code_of_conduct = configuration::AcceptCodeOfConduct,
}
