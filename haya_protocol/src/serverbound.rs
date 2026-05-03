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
    ping_request = ping::StatusPingRequest,
}
packets! {
    serverbound__login,
    LoginHandler,
    handle,
    hello = login::Hello<'_>,
    key = login::Key<'_>,
    custom_query_answer = login::CustomQueryAnswer<'_>,
    login_acknowledged = login::LoginAcknowledged,
    cookie_response = cookie::LoginCookieResponse<'_>,
}
packets! {
    serverbound__configuration,
    ConfigurationHandler,
    handle,
    client_information = common::ConfigurationClientInformation<'_>,
    cookie_response = cookie::ConfigurationCookieResponse<'_>,
    custom_payload = common::ConfigurationCustomPayload<'_>,
    finish_configuration = configuration::FinishConfiguration,
    keep_alive = common::ConfigurationKeepAlive,
    pong = common::ConfigurationPong,
    resource_pack = common::ConfigurationResourcePack,
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
    container_close = game::ContainerClose,
    container_slot_state_changed = game::ContainerSlotStateChanged,
    cookie_response = cookie::GameCookieResponse<'_>,
    custom_payload = common::GameCustomPayload<'_>,
    debug_subscription_request = game::DebugSubscriptionRequest<'_>,
    edit_book = game::EditBook<'_>,
    entity_tag_query = game::EntityTagQuery,
    interact = game::Interact,
    jigsaw_generate = game::JigsawGenerate,
    keep_alive = common::GameKeepAlive,
    lock_difficulty = game::LockDifficulty,
    move_player_pos = game::MovePlayerPos,
    move_player_pos_rot = game::MovePlayerPosRot,
    move_player_rot = game::MovePlayerRot,
    move_player_status_only = game::MovePlayerStatusOnly,
    move_vehicle = game::MoveVehicle,
    paddle_boat = game::PaddleBoat,
    pick_item_from_block = game::PickItemFromBlock,
    pick_item_from_entity = game::PickItemFromEntity,
    ping_request = ping::GamePingRequest,
    place_recipe = game::PlaceRecipe,
    player_abilities = game::PlayerAbilities,
    player_action = game::PlayerAction,
    player_command = game::PlayerCommand,
    player_input = game::PlayerInput,
    player_loaded = game::PlayerLoaded,
    pong = common::GamePong,
    recipe_book_change_settings = game::RecipeBookChangeSettings,
    recipe_book_seen_recipe = game::RecipeBookSeenRecipe,
    rename_item = game::RenameItem<'_>,
    resource_pack = common::GameResourcePack,
    seen_advancements = game::SeenAdvancements<'_>,
    select_trade = game::SelectTrade,
    set_beacon = game::SetBeacon,
    set_carried_item = game::SetCarriedItem,
    set_command_block = game::SetCommandBlock<'_>,
    set_command_minecart = game::SetCommandMinecart<'_>,
    set_creative_mode_slot = game::SetCreativeModeSlot<'_>,
    set_jigsaw_block = game::SetJigsawBlock<'_>,
    set_structure_block = game::SetStructureBlock<'_>,
    set_test_block = game::SetTestBlock<'_>,
    sign_update = game::SignUpdate<'_>,
    swing = game::Swing,
    teleport_to_entity = game::TeleportToEntity,
    // test_instance_block_action,
    // use_item_on,
    // use_item,
    // custom_click_action,
}
