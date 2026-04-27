use minecraft_data::{
    clientbound__configuration, clientbound__login, clientbound__play, clientbound__status,
};

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
            fn $handle(&mut self, mut packet: mser::Reader<'_>) -> Result<(), mser::Error> {
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
                    #[allow(unused)]
                    _ => {} // todo
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
    handle,
    status_response = status::StatusResponse<'_>,
    pong_response = ping::StatusPongResponse,
}
packets! {
    clientbound__login,
    LoginHandler,
    handle,
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
    handle,
    cookie_request = cookie::ConfigurationCookieRequest<'_>,
    custom_payload = common::ConfigurationCustomPayload<'_>,
    disconnect = common::ConfigurationDisconnect,
    finish_configuration = configuration::FinishConfiguration,
    keep_alive = common::ConfigurationKeepAlive,
    ping = common::ConfigurationPing,
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
packets! {
    clientbound__play,
    GameHandler,
    handle,
    bundle_delimiter = game::BundleDelimiter,
    add_entity = game::AddEntity,
    animate = game::Animate,
    award_stats = game::AwardStats<'_>,
    block_changed_ack = game::BlockChangedAck,
    block_destruction = game::BlockDestruction,
    block_entity_data = game::BlockEntityData,
    block_event = game::BlockEvent,
    block_update = game::BlockUpdate,
    boss_event = game::BossEvent,
    change_difficulty = game::ChangeDifficulty,
    chunk_batch_finished = game::ChunkBatchFinished,
    chunk_batch_start = game::ChunkBatchStart,
    chunks_biomes = game::ChunkBiomes<'_>,
    clear_titles = game::ClearTitles,
    command_suggestions = game::CommandSuggestions<'_>,
    commands = game::Commands<'_>,
    container_close = game::ContainerClose,
    container_set_content = game::ContainerSetContent<'_>,
    container_set_data = game::ContainerSetData,
    container_set_slot = game::ContainerSetSlot<'_>,
    cookie_request = cookie::GameCookieRequest<'_>,
    cooldown = game::Cooldown<'_>,
    custom_chat_completions = game::CustomChatCompletions<'_>,
    custom_payload = common::GameCustomPayload<'_>,
    damage_event = game::DamageEvent,
    debug_block_value = game::DebugBlockValue<'_>,
    debug_chunk_value = game::DebugChunkValue<'_>,
    debug_entity_value = game::DebugEntityValue<'_>,
    debug_event = game::DebugEvent<'_>,
    debug_sample = game::DebugSample<'_>,
    delete_chat = game::DeleteChat<'_>,
    disconnect = common::GameDisconnect,
    disguised_chat = game::DisguisedChat<'_>,
    entity_event = game::EntityEvent,
    entity_position_sync = game::EntityPositionSync,
    explode = game::Explode<'_>,
    forget_level_chunk = game::ForgetLevelChunk,
    game_event = game::GameEvent,
    game_test_highlight_pos = game::GameTestHighlightPos,
    mount_screen_open = game::MountScreenOpen,
    hurt_animation = game::HurtAnimation,
    initialize_border = game::InitializeBorder,
    keep_alive = common::GameKeepAlive,
    level_chunk_with_light = game::LevelChunkWithLight<'_>,
    level_event = game::LevelEvent,
    level_particles = game::LevelParticles<'_>,
    light_update = game::LightUpdate<'_>,
    login = game::Login<'_>,
    map_item_data = game::MapItemData<'_>,
    merchant_offers = game::MerchantOffers<'_>,
    move_entity_pos = game::MoveEntityPos,
    move_entity_pos_rot = game::MoveEntityPosRot,
    move_minecart_along_track = game::MoveMinecartAlongTrack<'_>,
    move_entity_rot = game::MoveEntityRot,
    move_vehicle = game::MoveVehicle,
    open_book = game::OpenBook,
    open_screen = game::OpenScreen,
    open_sign_editor = game::OpenSignEditor,
    ping = common::GamePing,
    pong_response = ping::GamePongResponse,
    place_ghost_recipe = game::PlaceGhostRecipe<'_>,
    player_abilities = game::PlayerAbilities,
    player_chat = game::PlayerChat<'_>,
    player_combat_end = game::PlayerCombatEnd,
    player_combat_enter = game::PlayerCombatEnter,
    player_combat_kill = game::PlayerCombatKill,
    player_info_remove = game::PlayerInfoRemove<'_>,
    player_info_update = game::PlayerInfoUpdate<'_>,
    player_look_at = game::PlayerLookAt,
    player_position = game::PlayerPosition,
    player_rotation = game::PlayerRotation,
    recipe_book_add = game::RecipeBookAdd<'_>,
    recipe_book_remove = game::RecipeBookRemove<'_>,
    recipe_book_settings = game::RecipeBookSettings,
    remove_entities = game::RemoveEntities<'_>,
    remove_mob_effect = game::RemoveMobEffect,
    reset_score = game::ResetScore<'_>,
    resource_pack_pop = game::ResourcePackPop,
    resource_pack_push = game::ResourcePackPush<'_>,
    respawn = game::Respawn<'_>,
    rotate_head = game::RotateHead,
    section_blocks_update = game::SectionBlocksUpdate<'_>,
    select_advancements_tab = game::SelectAdvancementsTab<'_>,
    server_data = game::ServerData<'_>,
    set_action_bar_text = game::SetActionBarText,
    // set_border_center,
    // set_border_lerp_size,
    // set_border_size,
    // set_border_warning_delay,
    // set_border_warning_distance,
    // set_camera,
    // set_chunk_cache_center,
    // set_chunk_cache_radius,
    // set_cursor_item,
    // set_default_spawn_position,
    // set_display_objective,
    // set_entity_data,
    // set_entity_link,
    // set_entity_motion,
    // set_equipment,
    // set_experience,
    // set_health,
    // set_held_slot,
    // set_objective,
    // set_passengers,
    // set_player_inventory,
    // set_player_team,
    // set_score,
    // set_simulation_distance,
    // set_subtitle_text,
    // set_time,
    // set_title_text,
    // set_titles_animation,
    // sound_entity,
    // sound,
    // start_configuration,
    // stop_sound,
    // store_cookie,
    // system_chat,
    // tab_list,
    // tag_query,
    // take_item_entity,
    // teleport_entity,
    // test_instance_block_status,
    // ticking_state,
    // ticking_step,
    // transfer,
    // update_advancements,
    // update_attributes,
    // update_mob_effect,
    // update_recipes,
    // update_tags,
    // projectile_power,
    // custom_report_details,
    // server_links,
    // waypoint,
    // clear_dialog,
    // show_dialog,
}
