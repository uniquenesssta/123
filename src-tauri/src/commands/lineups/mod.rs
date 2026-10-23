mod create_lineup;
mod create_lineup_pair;
mod formations;
mod list_lineups;
mod list_team_match_lineups;
mod matches;
mod presets;
mod read_lineup;
mod read_match_lineup_chain;
mod remove_lineup_history;

pub use create_lineup::{__cmd__create_lineup, __tauri_command_name_create_lineup, create_lineup};
pub use create_lineup_pair::{
    __cmd__create_lineup_pair, __tauri_command_name_create_lineup_pair, create_lineup_pair,
};
pub use formations::{
    __cmd__list_formation_usage_distributions, __cmd__list_formations,
    __cmd__resolve_formation_distribution, __cmd__save_formation_usage_distribution,
    __tauri_command_name_list_formation_usage_distributions, __tauri_command_name_list_formations,
    __tauri_command_name_resolve_formation_distribution,
    __tauri_command_name_save_formation_usage_distribution, list_formation_usage_distributions,
    list_formations, resolve_formation_distribution, save_formation_usage_distribution,
};
pub use list_lineups::{__cmd__list_lineups, __tauri_command_name_list_lineups, list_lineups};
pub use list_team_match_lineups::{
    __cmd__list_team_match_lineups, __tauri_command_name_list_team_match_lineups,
    list_team_match_lineups,
};
pub use matches::{
    __cmd__create_match, __cmd__delete_match, __tauri_command_name_create_match,
    __tauri_command_name_delete_match, create_match, delete_match,
};
pub use presets::{
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
pub use read_lineup::{__cmd__read_lineup, __tauri_command_name_read_lineup, read_lineup};
pub use read_match_lineup_chain::{
    __cmd__read_match_lineup_chain, __tauri_command_name_read_match_lineup_chain,
    read_match_lineup_chain,
};
pub use remove_lineup_history::{
    __cmd__remove_lineup_history, __tauri_command_name_remove_lineup_history, remove_lineup_history,
};
