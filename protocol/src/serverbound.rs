use minecraft_data::{serverbound__handshake, serverbound__status};

pub mod handshake;
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
    status::PingRequest => serverbound__status::ping_request,
}
