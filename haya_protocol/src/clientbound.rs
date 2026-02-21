use minecraft_data::{clientbound__configuration, clientbound__login, clientbound__status};

pub mod common;
pub mod configuration;
pub mod cookie;
pub mod game;
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
    clientbound__status,
    StatusHandler,
    handle_status,
    status_response = status::StatusResponse<'_>,
    pong_response = ping::PongResponse,
}
packets! {
    clientbound__login,
    LoginHandler,
    handle_login,
    login_disconnect = login::LoginDisconnect<'_>,
    hello = login::Hello<'_>,
    login_finished = login::LoginFinished<'_>,
    login_compression = login::LoginCompression,
    custom_query = login::CustomQuery<'_>,
    cookie_request = cookie::LoginCookieRequest<'_>,
}
packets! {
    clientbound__configuration,
    ConfigurationHandler,
    handle_configuration,
    cookie_request = cookie::ConfigurationCookieRequest<'_>,
    custom_payload = common::CustomPayload<'_>,
    disconnect = common::Disconnect,
    finish_configuration = configuration::FinishConfiguration,
    keep_alive = common::KeepAlive,
    ping = common::Ping,
    reset_chat = common::ResetChat,
    registry_data = configuration::RegistryData<'_>,
    resource_pack_pop = common::ResourcePackPop,
    resource_pack_push = common::ResourcePackPush<'_>,
    store_cookie = common::StoreCookie<'_>,
    transfer = common::Transfer<'_>,
    update_enabled_features = configuration::UpdateEnabledFeatures<'_>,
    update_tags = common::UpdateTags<'_>,
    select_known_packs = configuration::SelectKnownPacks<'_>,
    custom_report_details = common::CustomReportDetails<'_>,
    server_links = common::ServerLinks<'_>,
    clear_dialog = common::ClearDialog,
    show_dialog = common::ShowDialog,
    code_of_conduct = configuration::CodeOfConduct<'_>,
}
// packets! {
//     minecraft_data::clientbound__play,
//     GameHandler,
//     handle_game,
//     bundle_delimiter = game::BundleDelimiter,
//     add_entity = game::AddEntity,
//     animate = game::Animate,
//     award_stats = game::AwardStats<'_>,
//     block_changed_ack = game::BlockChangedAck,
//     block_destruction = game::BlockDestruction,
//     block_entity_data = game::BlockEntityData,
//     block_event = game::BlockEvent,
//     block_update = game::BlockUpdate,
//     boss_event = game::BossEvent,
//     change_difficulty = game::ChangeDifficulty,
//     chunk_batch_finished = game::ChunkBatchFinished,
//     chunk_batch_start = game::ChunkBatchStart,
// }
