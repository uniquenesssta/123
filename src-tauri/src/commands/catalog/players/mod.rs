mod add_player_ability_observation;
mod add_player_availability;
mod add_player_dynamic_tag;
mod add_player_name;
mod add_player_team_period;
mod assign_player_position;
mod bulk_delete_players;
mod calculate_player_match_contribution;
mod create_player;
mod delete_player;
mod list_players;
mod player_catalog_reference_data;
mod read_player;
mod update_player;

pub use add_player_ability_observation::{
    __cmd__add_player_ability_observation, __tauri_command_name_add_player_ability_observation,
    add_player_ability_observation,
};
pub use add_player_availability::{
    __cmd__add_player_availability, __tauri_command_name_add_player_availability,
    add_player_availability,
};
pub use add_player_dynamic_tag::{
    __cmd__add_player_dynamic_tag, __tauri_command_name_add_player_dynamic_tag,
    add_player_dynamic_tag,
};
pub use add_player_name::{
    __cmd__add_player_name, __tauri_command_name_add_player_name, add_player_name,
};
pub use add_player_team_period::{
    __cmd__add_player_team_period, __tauri_command_name_add_player_team_period,
    add_player_team_period,
};
pub use assign_player_position::{
    __cmd__assign_player_position, __tauri_command_name_assign_player_position,
    assign_player_position,
};
pub use bulk_delete_players::{
    __cmd__bulk_delete_players, __tauri_command_name_bulk_delete_players, bulk_delete_players,
};
pub use calculate_player_match_contribution::{
    __cmd__calculate_player_match_contribution,
    __tauri_command_name_calculate_player_match_contribution, calculate_player_match_contribution,
};
pub use create_player::{__cmd__create_player, __tauri_command_name_create_player, create_player};
pub use delete_player::{__cmd__delete_player, __tauri_command_name_delete_player, delete_player};
pub use list_players::{__cmd__list_players, __tauri_command_name_list_players, list_players};
pub use player_catalog_reference_data::{
    __cmd__player_catalog_reference_data, __tauri_command_name_player_catalog_reference_data,
    player_catalog_reference_data,
};
pub use read_player::{__cmd__read_player, __tauri_command_name_read_player, read_player};
pub use update_player::{__cmd__update_player, __tauri_command_name_update_player, update_player};
