use minecraft_data::{clientbound__login, clientbound__status};

pub mod common;
pub mod configuration;
pub mod cookie;
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
    clientbound__status,
    StatusHandler,
    status_response = status::StatusResponse<'_>,
    pong_response = ping::PongResponse,
}
packets! {
    clientbound__login,
    LoginHandler,
    login_disconnect = login::LoginDisconnect<'_>,
    hello = login::Hello<'_>,
    login_finished = login::LoginFinished<'_>,
    login_compression = login::LoginCompression,
    custom_query = login::CustomQuery<'_>,
    cookie_request = cookie::LoginCookieRequest<'_>,
}
// packets! {
//     clientbound__configuration,
//     ConfigurationHandler,
//     cookie_request = cookie::ConfigCookieRequest<'_>,
//     custom_payload = common::CustomPayload<'_>,
//     disconnect = common::Disconnect,
//     finish_configuration = configuration::FinishConfiguration,
//     keep_alive = common::KeepAlive,
//     ping = common::Ping,
//     reset_chat = common::ResetChat,
//     registry_data = configuration::RegistryData<'_>,
//     resource_pack_pop = common::ResourcePackPop,
//     resource_pack_push = common::ResourcePackPush<'_>,
//     store_cookie = common::StoreCookie<'_>,
//     transfer = common::Transfer<'_>,
//     update_enabled_features = configuration::UpdateEnabledFeatures<'_>,
//     update_tags,
//     select_known_packs,
//     custom_report_details,
//     server_links,
//     clear_dialog,
//     show_dialog,
//     code_of_conduct,
// }
