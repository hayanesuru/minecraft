use minecraft_data::{clientbound__configuration, clientbound__login, clientbound__status};

pub mod common;
pub mod configuration;
pub mod cookie;
pub mod login;
pub mod ping;
pub mod status;

macro_rules! packets {
    ($m:ty,$($rest:tt)*) => {
        packets!(@munch $m; $($rest)*);
    };
    (@munch $m:ty;
        $type:ty => $variant:ident,
        $($tail:tt)*
    ) => {
        #[automatically_derived]
        impl crate::types::Id<$m> for $type {
            const ID: $m = <$m>::$variant;
        }
        packets!(@munch $m; $($tail)*);
    };
    (@munch $m:ty;
        $type:ty => $variant:ident
    ) => {
        #[automatically_derived]
        impl crate::types::Id<$m> for $type {
            const ID: $m = <$m>::$variant;
        }
    };
    (@munch $m:ty; , $($tail:tt)*) => {
        packets!(@munch $m; $($tail)*);
    };
    (@munch $m:ty; ,) => {};
    (@munch $m:ty;) => {};
}
packets! {
    clientbound__status,
    status::StatusResponse<'_> => status_response,
    ping::PongResponse => pong_response,
}
packets! {
    clientbound__login,
    login::LoginDisconnect<'_> => login_disconnect,
    login::Hello<'_> => hello,
    login::LoginFinished<'_>  => login_finished,
    login::LoginCompression => login_compression,
    login::CustomQuery<'_> => custom_query,
    cookie::CookieRequest<'_> => cookie_request,
}
packets! {
    clientbound__configuration,
    cookie::CookieRequest<'_> => cookie_request,
    common::CustomPayload<'_> => custom_payload,
    common::Disconnect => disconnect,
    configuration::FinishConfiguration => finish_configuration,
    common::KeepAlive => keep_alive,
    common::Ping => ping,
    common::ResetChat => reset_chat,
    // registry_data,
    // resource_pack_pop,
    // resource_pack_push,
    // store_cookie,
    // transfer,
    // update_enabled_features,
    // update_tags,
    // select_known_packs,
    // custom_report_details,
    // server_links,
    // clear_dialog,
    // show_dialog,
    // code_of_conduct,
}
