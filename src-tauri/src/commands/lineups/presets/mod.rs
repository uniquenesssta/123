mod archive_team_lineup_preset;
mod delete_team_lineup_preset;
mod duplicate_team_lineup_preset;
mod list_team_lineup_presets;
mod preview_team_lineup_preset_application;
mod save_team_lineup_preset;

pub use archive_team_lineup_preset::{
    __cmd__archive_team_lineup_preset, __tauri_command_name_archive_team_lineup_preset,
    archive_team_lineup_preset,
};
pub use delete_team_lineup_preset::{
    __cmd__delete_team_lineup_preset, __tauri_command_name_delete_team_lineup_preset,
    delete_team_lineup_preset,
};
pub use duplicate_team_lineup_preset::{
    __cmd__duplicate_team_lineup_preset, __tauri_command_name_duplicate_team_lineup_preset,
    duplicate_team_lineup_preset,
};
pub use list_team_lineup_presets::{
    __cmd__list_team_lineup_presets, __tauri_command_name_list_team_lineup_presets,
    list_team_lineup_presets,
};
pub use preview_team_lineup_preset_application::{
    __cmd__preview_team_lineup_preset_application,
    __tauri_command_name_preview_team_lineup_preset_application,
    preview_team_lineup_preset_application,
};
pub use save_team_lineup_preset::{
    __cmd__save_team_lineup_preset, __tauri_command_name_save_team_lineup_preset,
    save_team_lineup_preset,
};
