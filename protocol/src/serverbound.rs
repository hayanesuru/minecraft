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
    ($m:ty,$($rest:tt)*) => {
        packets!(@munch $m; $($rest)*);
    };
    (@munch $m:ty;
        $variant:ident = $type:ty,
        $($tail:tt)*
    ) => {
        #[automatically_derived]
        impl crate::types::Id for $type {
            type T = $m;
            const ID: $m = <$m>::$variant;
        }
        packets!(@munch $m; $($tail)*);
    };
    (@munch $m:ty;
        $variant:ident = $type:ty
    ) => {
        #[automatically_derived]
        impl crate::types::Id for $type {
            type T = $m;
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
    serverbound__handshake,
    intention = handshake::Intention<'_>,
}
packets! {
    serverbound__status,
    status_request = status::StatusRequest,
    ping_request = ping::PingRequest,
}
packets! {
    serverbound__login,
    hello = login::Hello<'_>,
    key = login::Key<'_>,
    custom_query_answer = login::CustomQueryAnswer<'_>,
    login_acknowledged = login::LoginAcknowledged,
    cookie_response = cookie::CookieResponse<'_>,
}
packets! {
    serverbound__configuration,

}
