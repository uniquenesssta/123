mod coaches;
mod entities;
mod formations;
mod lineup_presets;
mod lineups;
mod matches;
mod players;
mod teams;

pub use coaches::{
    __cmd__add_coach_name, __cmd__add_team_coach_period, __cmd__create_coach, __cmd__list_coaches,
    __cmd__read_coach, __tauri_command_name_add_coach_name,
    __tauri_command_name_add_team_coach_period, __tauri_command_name_create_coach,
    __tauri_command_name_list_coaches, __tauri_command_name_read_coach, add_coach_name,
    add_team_coach_period, create_coach, list_coaches, read_coach,
};
pub use entities::{
    __cmd__add_external_entity_id, __cmd__bulk_archive_entities, __cmd__check_entity_deletion,
    __cmd__create_data_provider, __cmd__list_entity_references, __cmd__resolve_entity_reference,
    __tauri_command_name_add_external_entity_id, __tauri_command_name_bulk_archive_entities,
    __tauri_command_name_check_entity_deletion, __tauri_command_name_create_data_provider,
    __tauri_command_name_list_entity_references, __tauri_command_name_resolve_entity_reference,
    add_external_entity_id, bulk_archive_entities, check_entity_deletion, create_data_provider,
    list_entity_references, resolve_entity_reference,
};
pub use formations::{
    __cmd__list_formation_usage_distributions, __cmd__list_formations,
    __cmd__resolve_formation_distribution, __cmd__save_formation_usage_distribution,
    __tauri_command_name_list_formation_usage_distributions, __tauri_command_name_list_formations,
    __tauri_command_name_resolve_formation_distribution,
    __tauri_command_name_save_formation_usage_distribution, list_formation_usage_distributions,
    list_formations, resolve_formation_distribution, save_formation_usage_distribution,
};
pub use lineup_presets::{
    __cmd__archive_team_lineup_preset, __cmd__delete_team_lineup_preset,
    __cmd__duplicate_team_lineup_preset, __cmd__list_team_lineup_presets,
    __cmd__preview_team_lineup_preset_application, __cmd__save_team_lineup_preset,
    __tauri_command_name_archive_team_lineup_preset,
    __tauri_command_name_delete_team_lineup_preset,
    __tauri_command_name_duplicate_team_lineup_preset,
    __tauri_command_name_list_team_lineup_presets,
    __tauri_command_name_preview_team_lineup_preset_application,
    __tauri_command_name_save_team_lineup_preset, archive_team_lineup_preset,
    delete_team_lineup_preset, duplicate_team_lineup_preset, list_team_lineup_presets,
    preview_team_lineup_preset_application, save_team_lineup_preset,
};
pub use lineups::{
    __cmd__create_lineup, __cmd__create_lineup_pair, __cmd__list_lineups,
    __cmd__list_team_match_lineups, __cmd__read_lineup, __cmd__read_match_lineup_chain,
    __cmd__remove_lineup_history, __tauri_command_name_create_lineup,
    __tauri_command_name_create_lineup_pair, __tauri_command_name_list_lineups,
    __tauri_command_name_list_team_match_lineups, __tauri_command_name_read_lineup,
    __tauri_command_name_read_match_lineup_chain, __tauri_command_name_remove_lineup_history,
    create_lineup, create_lineup_pair, list_lineups, list_team_match_lineups, read_lineup,
    read_match_lineup_chain, remove_lineup_history,
};
pub use matches::{
    __cmd__create_match, __cmd__delete_match, __tauri_command_name_create_match,
    __tauri_command_name_delete_match, create_match, delete_match,
};
pub use players::{
    __cmd__add_player_ability_observation, __cmd__add_player_availability,
    __cmd__add_player_dynamic_tag, __cmd__add_player_name, __cmd__add_player_team_period,
    __cmd__assign_player_position, __cmd__bulk_delete_players,
    __cmd__calculate_player_match_contribution, __cmd__create_player, __cmd__delete_player,
    __cmd__list_players, __cmd__player_catalog_reference_data, __cmd__read_player,
    __cmd__update_player, __tauri_command_name_add_player_ability_observation,
    __tauri_command_name_add_player_availability, __tauri_command_name_add_player_dynamic_tag,
    __tauri_command_name_add_player_name, __tauri_command_name_add_player_team_period,
    __tauri_command_name_assign_player_position, __tauri_command_name_bulk_delete_players,
    __tauri_command_name_calculate_player_match_contribution, __tauri_command_name_create_player,
    __tauri_command_name_delete_player, __tauri_command_name_list_players,
    __tauri_command_name_player_catalog_reference_data, __tauri_command_name_read_player,
    __tauri_command_name_update_player, add_player_ability_observation, add_player_availability,
    add_player_dynamic_tag, add_player_name, add_player_team_period, assign_player_position,
    bulk_delete_players, calculate_player_match_contribution, create_player, delete_player,
    list_players, player_catalog_reference_data, read_player, update_player,
};
pub use teams::{
    __cmd__add_team_name, __cmd__bulk_delete_teams, __cmd__create_team, __cmd__force_delete_team,
    __cmd__list_team_options, __cmd__list_teams, __cmd__preview_force_delete_team,
    __cmd__read_team, __cmd__update_team, __cmd__upsert_team_profile,
    __tauri_command_name_add_team_name, __tauri_command_name_bulk_delete_teams,
    __tauri_command_name_create_team, __tauri_command_name_force_delete_team,
    __tauri_command_name_list_team_options, __tauri_command_name_list_teams,
    __tauri_command_name_preview_force_delete_team, __tauri_command_name_read_team,
    __tauri_command_name_update_team, __tauri_command_name_upsert_team_profile, add_team_name,
    bulk_delete_teams, create_team, force_delete_team, list_team_options, list_teams,
    preview_force_delete_team, read_team, update_team, upsert_team_profile,
};
