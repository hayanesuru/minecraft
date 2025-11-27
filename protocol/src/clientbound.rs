use alloc::alloc::Allocator;
use minecraft_data::{clientbound__login, clientbound__status};

pub mod login;
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
    clientbound__status,
    status::StatusResponse<'_> => clientbound__status::status_response,
    status::PongResponse => clientbound__status::pong_response,
}
packets! {
    clientbound__login,
    login::LoginDisconnect<'_> => clientbound__login::login_disconnect,
    login::Hello<'_> => clientbound__login::hello,
    login::LoginFinished<'_, A> where Allocator => clientbound__login::login_finished,
    login::LoginCompression => clientbound__login::login_compression,
    login::CustomQuery<'_> => clientbound__login::custom_query,
    
}
