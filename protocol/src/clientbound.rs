use minecraft_data::{clientbound__login, clientbound__status};

pub mod login;
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
    clientbound__status,
    status::StatusResponse<'_> => clientbound__status::status_response,
    status::PongResponse => clientbound__status::pong_response,
}
packets! {
    clientbound__login,
    login::LoginDisconnect<'_> => clientbound__login::login_disconnect,
}
