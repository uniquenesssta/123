mod add_team_name;
mod bulk_delete_teams;
mod create_team;
mod force_delete_team;
mod list_team_options;
mod list_teams;
mod preview_force_delete_team;
mod read_team;
mod update_team;
mod upsert_team_profile;

pub use add_team_name::{__cmd__add_team_name, __tauri_command_name_add_team_name, add_team_name};
pub use bulk_delete_teams::{
    __cmd__bulk_delete_teams, __tauri_command_name_bulk_delete_teams, bulk_delete_teams,
};
pub use create_team::{__cmd__create_team, __tauri_command_name_create_team, create_team};
pub use force_delete_team::{
    __cmd__force_delete_team, __tauri_command_name_force_delete_team, force_delete_team,
};
pub use list_team_options::{
    __cmd__list_team_options, __tauri_command_name_list_team_options, list_team_options,
};
pub use list_teams::{__cmd__list_teams, __tauri_command_name_list_teams, list_teams};
pub use preview_force_delete_team::{
    __cmd__preview_force_delete_team, __tauri_command_name_preview_force_delete_team,
    preview_force_delete_team,
};
pub use read_team::{__cmd__read_team, __tauri_command_name_read_team, read_team};
pub use update_team::{__cmd__update_team, __tauri_command_name_update_team, update_team};
pub use upsert_team_profile::{
    __cmd__upsert_team_profile, __tauri_command_name_upsert_team_profile, upsert_team_profile,
};
