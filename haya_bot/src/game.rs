use crate::Client;
use haya_protocol::clientbound::{GameHandler, common, cookie, game, ping};
use haya_protocol::serverbound::common::{GameKeepAlive, KeepAlive};
use haya_protocol::serverbound::game::{AcceptTeleportation, ClientInformation};
use minecraft_data::clientbound_play;
use mser::{Read, Reader, Utf8, V21};
use tokio::io::AsyncReadExt;

pub async fn handle_game(
    client: &mut Client,
    b: &mut Vec<u8>,
    c: &mut usize,
) -> tokio::io::Result<()> {
    let mut h = Game {
        c: client,
        disconnected: false,
    };
    h.c.write(&ClientInformation {
        information: haya_protocol::ClientInformation {
            language: Utf8("en_US"),
            view_distance: 16,
            chat_visibility: haya_protocol::ChatVisibility::Full,
            chat_colors: true,
            model_customisation: 0,
            main_hand: haya_protocol::inventory::HumanoidArm::Left,
            text_filtering_enabled: false,
            allows_listing: true,
            particle_status: haya_protocol::ParticleStatus::All,
        },
    });
    h.c.flush().await?;
    loop {
        let mut flag = true;
        let (read, p) = loop {
            if *c == b.len() {
                b.clear();
                *c = 0;
            }
            if !flag {
                b.reserve(4096);
                if h.c.s.read_buf(b).await? == 0 {
                    h.disconnected = true;
                }
            }
            flag = false;
            let buf = &b[*c..];
            let ptr = buf.as_ptr();
            let mut r = Reader::new(buf);
            let len = match V21::read(&mut r) {
                Ok(x) => x.0 as usize,
                Err(_) => continue,
            };
            match r.read_slice(len) {
                Ok(x) => unsafe { break (r.offset_from(ptr), x) },
                Err(_) => continue,
            }
        };
        *c += read;
        let reader = Reader::new(p);
        match h.handle(reader) {
            Ok(()) => {}
            Err(_) => {
                let mut r = Reader::new(p);
                let id = clientbound_play::read(&mut r).unwrap();
                let size = r.len();
                panic!("failed read {id} size={size} {p:x?}");
            }
        }
        if !h.c.b.is_empty() {
            h.c.flush().await?;
        }
        if h.disconnected {
            break;
        }
    }
    Ok(())
}

pub struct Game<'a> {
    c: &'a mut Client,
    disconnected: bool,
}

impl<'a> GameHandler for Game<'a> {
    fn bundle_delimiter(&mut self, _: game::BundleDelimiter) {}

    fn add_entity(&mut self, _: game::AddEntity) {}

    fn animate(&mut self, _: game::Animate) {}

    fn award_stats(&mut self, _: game::AwardStats<'_>) {}

    fn block_changed_ack(&mut self, _: game::BlockChangedAck) {}

    fn block_destruction(&mut self, _: game::BlockDestruction) {}

    fn block_entity_data(&mut self, _: game::BlockEntityData) {}

    fn block_event(&mut self, _: game::BlockEvent) {}

    fn block_update(&mut self, _: game::BlockUpdate) {}

    fn boss_event(&mut self, _: game::BossEvent) {}

    fn change_difficulty(&mut self, _: game::ChangeDifficulty) {}

    fn chunk_batch_finished(&mut self, _: game::ChunkBatchFinished) {}

    fn chunk_batch_start(&mut self, _: game::ChunkBatchStart) {}

    fn chunks_biomes(&mut self, _: game::ChunkBiomes<'_>) {}

    fn clear_titles(&mut self, _: game::ClearTitles) {}

    fn command_suggestions(&mut self, _: game::CommandSuggestions<'_>) {}

    fn commands(&mut self, _: game::Commands<'_>) {}

    fn container_close(&mut self, _: game::ContainerClose) {}

    fn container_set_content(&mut self, _: game::ContainerSetContent<'_>) {}

    fn container_set_data(&mut self, _: game::ContainerSetData) {}

    fn container_set_slot(&mut self, _: game::ContainerSetSlot<'_>) {}

    fn cookie_request(&mut self, _: cookie::GameCookieRequest<'_>) {}

    fn cooldown(&mut self, _: game::Cooldown<'_>) {}

    fn custom_chat_completions(&mut self, _: game::CustomChatCompletions<'_>) {}

    fn custom_payload(&mut self, _: common::GameCustomPayload<'_>) {}

    fn damage_event(&mut self, _: game::DamageEvent) {}

    fn debug_block_value(&mut self, _: game::DebugBlockValue<'_>) {}

    fn debug_chunk_value(&mut self, _: game::DebugChunkValue<'_>) {}

    fn debug_entity_value(&mut self, _: game::DebugEntityValue<'_>) {}

    fn debug_event(&mut self, _: game::DebugEvent<'_>) {}

    fn debug_sample(&mut self, _: game::DebugSample<'_>) {}

    fn delete_chat(&mut self, _: game::DeleteChat<'_>) {}

    fn disconnect(&mut self, _: common::GameDisconnect) {
        self.disconnected = true;
    }

    fn disguised_chat(&mut self, _: game::DisguisedChat<'_>) {}

    fn entity_event(&mut self, _: game::EntityEvent) {}

    fn entity_position_sync(&mut self, _: game::EntityPositionSync) {}

    fn explode(&mut self, _: game::Explode<'_>) {}

    fn forget_level_chunk(&mut self, _: game::ForgetLevelChunk) {}

    fn game_event(&mut self, _: game::GameEvent) {}

    fn game_test_highlight_pos(&mut self, _: game::GameTestHighlightPos) {}

    fn mount_screen_open(&mut self, _: game::MountScreenOpen) {}

    fn hurt_animation(&mut self, _: game::HurtAnimation) {}

    fn initialize_border(&mut self, _: game::InitializeBorder) {}

    fn keep_alive(&mut self, p: common::GameKeepAlive) {
        self.c.write(&GameKeepAlive(KeepAlive { id: p.0.id }));
    }

    fn level_chunk_with_light(&mut self, _: game::LevelChunkWithLight<'_>) {}

    fn level_event(&mut self, _: game::LevelEvent) {}

    fn level_particles(&mut self, _: game::LevelParticles<'_>) {}

    fn light_update(&mut self, _: game::LightUpdate<'_>) {}

    fn login(&mut self, _: game::Login<'_>) {}

    fn map_item_data(&mut self, _: game::MapItemData<'_>) {}

    fn merchant_offers(&mut self, _: game::MerchantOffers<'_>) {}

    fn move_entity_pos(&mut self, _: game::MoveEntityPos) {}

    fn move_entity_pos_rot(&mut self, _: game::MoveEntityPosRot) {}

    fn move_minecart_along_track(&mut self, _: game::MoveMinecartAlongTrack<'_>) {}

    fn move_entity_rot(&mut self, _: game::MoveEntityRot) {}

    fn move_vehicle(&mut self, _: game::MoveVehicle) {}

    fn open_book(&mut self, _: game::OpenBook) {}

    fn open_screen(&mut self, _: game::OpenScreen) {}

    fn open_sign_editor(&mut self, _: game::OpenSignEditor) {}

    fn ping(&mut self, _: common::GamePing) {}

    fn pong_response(&mut self, _: ping::GamePongResponse) {}

    fn place_ghost_recipe(&mut self, _: game::PlaceGhostRecipe<'_>) {}

    fn player_abilities(&mut self, _: game::PlayerAbilities) {}

    fn player_chat(&mut self, _: game::PlayerChat<'_>) {}

    fn player_combat_end(&mut self, _: game::PlayerCombatEnd) {}

    fn player_combat_enter(&mut self, _: game::PlayerCombatEnter) {}

    fn player_combat_kill(&mut self, _: game::PlayerCombatKill) {}

    fn player_info_remove(&mut self, _: game::PlayerInfoRemove<'_>) {}

    fn player_info_update(&mut self, _: game::PlayerInfoUpdate<'_>) {}

    fn player_look_at(&mut self, _: game::PlayerLookAt) {}

    fn player_position(&mut self, p: game::PlayerPosition) {
        self.c.write(&AcceptTeleportation { id: p.id });
    }

    fn player_rotation(&mut self, _: game::PlayerRotation) {}

    fn recipe_book_add(&mut self, _: game::RecipeBookAdd<'_>) {}

    fn recipe_book_remove(&mut self, _: game::RecipeBookRemove<'_>) {}

    fn recipe_book_settings(&mut self, _: game::RecipeBookSettings) {}

    fn remove_entities(&mut self, _: game::RemoveEntities<'_>) {}

    fn remove_mob_effect(&mut self, _: game::RemoveMobEffect) {}

    fn reset_score(&mut self, _: game::ResetScore<'_>) {}

    fn resource_pack_pop(&mut self, _: game::ResourcePackPop) {}

    fn resource_pack_push(&mut self, _: game::ResourcePackPush<'_>) {}

    fn respawn(&mut self, _: game::Respawn<'_>) {}

    fn rotate_head(&mut self, _: game::RotateHead) {}

    fn section_blocks_update(&mut self, _: game::SectionBlocksUpdate<'_>) {}

    fn select_advancements_tab(&mut self, _: game::SelectAdvancementsTab<'_>) {}

    fn server_data(&mut self, _: game::ServerData<'_>) {}

    fn set_action_bar_text(&mut self, _: game::SetActionBarText) {}

    fn set_border_center(&mut self, _: game::SetBorderCenter) {}

    fn set_border_lerp_size(&mut self, _: game::SetBorderLerpSize) {}

    fn set_border_size(&mut self, _: game::SetBorderSize) {}

    fn set_border_warning_delay(&mut self, _: game::SetBorderWarningDelay) {}

    fn set_border_warning_distance(&mut self, _: game::SetBorderWarningDistance) {}

    fn set_camera(&mut self, _: game::SetCamera) {}

    fn set_chunk_cache_center(&mut self, _: game::SetChunkCacheCenter) {}

    fn set_chunk_cache_radius(&mut self, _: game::SetChunkCacheRadius) {}

    fn set_cursor_item(&mut self, _: game::SetCursorItem<'_>) {}

    fn set_default_spawn_position(&mut self, _: game::SetDefaultSpawnPosition<'_>) {}

    fn set_display_objective(&mut self, _: game::SetDisplayObjective<'_>) {}

    fn set_entity_data(&mut self, _: game::SetEntityData<'_>) {}

    fn set_entity_link(&mut self, _: game::SetEntityLink) {}

    fn set_entity_motion(&mut self, _: game::SetEntityMotion) {}

    fn set_equipment(&mut self, _: game::SetEquipment<'_>) {}

    fn set_experience(&mut self, _: game::SetExperience) {}

    fn set_health(&mut self, _: game::SetHealth) {}

    fn set_held_slot(&mut self, _: game::SetHeldSlot) {}

    fn set_objective(&mut self, _: game::SetObjective<'_>) {}

    fn set_passengers(&mut self, _: game::SetPassengers<'_>) {}

    fn set_player_inventory(&mut self, _: game::SetPlayerInventory<'_>) {}

    fn set_player_team(&mut self, _: game::SetPlayerTeam<'_>) {}

    fn set_score(&mut self, _: game::SetScore<'_>) {}

    fn set_simulation_distance(&mut self, _: game::SetSimulationDistance) {}

    fn set_subtitle_text(&mut self, _: game::SetSubtitleText) {}

    fn set_time(&mut self, _: game::SetTime) {}

    fn set_title_text(&mut self, _: game::SetTitleText) {}

    fn set_titles_animation(&mut self, _: game::SetTitlesAnimation) {}

    fn sound_entity(&mut self, _: game::SoundEntity<'_>) {}

    fn sound(&mut self, _: game::Sound<'_>) {}

    fn start_configuration(&mut self, _: game::StartConfiguration) {}

    fn stop_sound(&mut self, _: game::StopSound<'_>) {}

    fn store_cookie(&mut self, _: common::GameStoreCookie<'_>) {}

    fn system_chat(&mut self, _: game::SystemChat) {}

    fn tab_list(&mut self, _: game::TabList) {}

    fn tag_query(&mut self, _: game::TagQuery) {}

    fn take_item_entity(&mut self, _: game::TakeItemEntity) {}

    fn teleport_entity(&mut self, _: game::TeleportEntity) {}

    fn test_instance_block_status(&mut self, _: game::TestInstanceBlockStatus) {}

    fn ticking_state(&mut self, _: game::TickingState) {}

    fn ticking_step(&mut self, _: game::TickingStep) {}

    fn transfer(&mut self, _: common::GameTransfer<'_>) {}

    fn update_advancements(&mut self, _: game::UpdateAdvancements<'_>) {}

    fn update_attributes(&mut self, _: game::UpdateAttributes<'_>) {}

    fn update_mob_effect(&mut self, _: game::UpdateMobEffect) {}

    fn update_recipes(&mut self, _: game::UpdateRecipes<'_>) {}

    fn update_tags(&mut self, _: common::GameUpdateTags<'_>) {}

    fn projectile_power(&mut self, _: game::ProjectilePower) {}

    fn custom_report_details(&mut self, _: common::GameCustomReportDetails<'_>) {}

    fn server_links(&mut self, _: common::GameServerLinks<'_>) {}

    fn waypoint(&mut self, _: game::Waypoint<'_>) {}

    fn clear_dialog(&mut self, _: common::GameClearDialog) {}

    fn show_dialog(&mut self, _: common::GameShowDialog) {}

    fn game_rule_values(&mut self, _: game::GameRuleValues<'_>) {}

    fn low_disk_space_warning(&mut self, _: game::LowDiskSpaceWarning) {}
}
