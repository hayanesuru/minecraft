use minecraft_data::{
    serverbound__configuration, serverbound__handshake, serverbound__login, serverbound__play,
    serverbound__status,
};

pub mod common;
pub mod configuration;
pub mod cookie;
pub mod game;
pub mod handshake;
pub mod login;
pub mod ping;
pub mod status;

#[cold]
fn err() -> Result<(), mser::Error> {
    Err(mser::Error)
}

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
            fn $handle(&mut self, mut packet: mser::Reader<'_>) -> Result<(), mser::Error> {
                match <$m as mser::Read>::read(&mut packet)? {
                    $(
                        <$m>::$variant => {
                            let e = <$type as mser::Read>::read(&mut packet)?;
                            if !packet.is_empty() {
                                return err();
                            }
                            self.$variant(e);
                        }
                    )+
                    #[allow(unreachable_patterns)]
                    _ => todo!(),
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
packets! {
    serverbound__play,
    GameHandler,
    handle,
    accept_teleportation = game::AcceptTeleportation,
    block_entity_tag_query = game::BlockEntityTagQuery,
    bundle_item_selected = game::BundleItemSelected,
    change_difficulty = game::ChangeDifficulty,
    change_game_mode = game::ChangeGameMode,
    chat_ack = game::ChatAck,
    chat_command = game::ChatCommand<'_>,
    chat_command_signed = game::ChatCommandSigned<'_>,
    chat = game::Chat<'_>,
    chat_session_update = game::ChatSessionUpdate<'_>,
    chunk_batch_received = game::ChunkBatchReceived,
    client_command = game::ClientCommand,
    client_tick_end = game::ClientTickEnd,
    client_information = game::ClientInformation<'_>,
    command_suggestion = game::CommandSuggestion<'_>,
    configuration_acknowledged = game::ConfigurationAcknowledged,
    container_button_click = game::ContainerButtonClick,
    container_click = game::ContainerClick<'_>,
    // container_close,
    // container_slot_state_changed,
    // cookie_response,
    // custom_payload,
    // debug_subscription_request,
    // edit_book,
    // entity_tag_query,
    // interact,
    // jigsaw_generate,
    // keep_alive,
    // lock_difficulty,
    // move_player_pos,
    // move_player_pos_rot,
    // move_player_rot,
    // move_player_status_only,
    // move_vehicle,
    // paddle_boat,
    // pick_item_from_block,
    // pick_item_from_entity,
    // ping_request,
    // place_recipe,
    // player_abilities,
    // player_action,
    // player_command,
    // player_input,
    // player_loaded,
    // pong,
    // recipe_book_change_settings,
    // recipe_book_seen_recipe,
    // rename_item,
    // resource_pack,
    // seen_advancements,
    // select_trade,
    // set_beacon,
    // set_carried_item,
    // set_command_block,
    // set_command_minecart,
    // set_creative_mode_slot,
    // set_jigsaw_block,
    // set_structure_block,
    // set_test_block,
    // sign_update,
    // swing,
    // teleport_to_entity,
    // test_instance_block_action,
    // use_item_on,
    // use_item,
    // custom_click_action,
}
