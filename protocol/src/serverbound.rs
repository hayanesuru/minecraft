use minecraft_data::{serverbound__handshake, serverbound__login, serverbound__status};

pub mod common;
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
    serverbound__handshake,
    handshake::Intention<'_> => intention
}
packets! {
    serverbound__status,
    status::StatusRequest => status_request,
    ping::PingRequest => ping_request,
}
packets! {
    serverbound__login,
    login::Hello<'_> => hello,
    login::Key<'_> => key,
    login::CustomQueryAnswer<'_> => custom_query_answer,
    login::LoginAcknowledged => login_acknowledged,
    cookie::CookieResponse<'_> => cookie_response,
}
