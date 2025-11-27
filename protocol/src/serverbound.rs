use minecraft_data::{serverbound__handshake, serverbound__login, serverbound__status};

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
        $type:ty where Allocator => $variant:path,
        $($tail:tt)*
    ) => {
        #[automatically_derived]
        impl<A: Allocator> crate::Id<$m> for $type {
            fn id() -> $m { $variant }
        }
        packets!(@munch $m; $($tail)*);
    };
    (@munch $m:ty;
        $type:ty where Allocator => $variant:path
    ) => {
        #[automatically_derived]
        impl<A: Allocator> crate::Id<$m> for $type {
            fn id() -> $m { $variant }
        }
    };
    (@munch $m:ty;
        $type:ty => $variant:path,
        $($tail:tt)*
    ) => {
        #[automatically_derived]
        impl crate::Id<$m> for $type {
            fn id() -> $m { $variant }
        }
        packets!(@munch $m; $($tail)*);
    };
    (@munch $m:ty;
        $type:ty => $variant:path
    ) => {
        #[automatically_derived]
        impl crate::Id<$m> for $type {
            fn id() -> $m { $variant }
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
    handshake::Intention<'_> => serverbound__handshake::intention
}
packets! {
    serverbound__status,
    status::StatusRequest => serverbound__status::status_request,
    ping::PingRequest => serverbound__status::ping_request,
}
packets! {
    serverbound__login,
    login::Hello<'_> => serverbound__login::hello,
    login::Key<'_> => serverbound__login::key,
    login::CustomQueryAnswer<'_> => serverbound__login::custom_query_answer,
    login::LoginAcknowledged => serverbound__login::login_acknowledged,
    cookie::CookieResponse<'_> => serverbound__login::cookie_response,
}
