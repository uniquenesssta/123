mod create_competition;
mod create_round;
mod create_season;
mod create_stage;
mod delete_competition;
mod rules;

pub use create_competition::{
    __cmd__create_competition, __tauri_command_name_create_competition, create_competition,
};
pub use create_round::{__cmd__create_round, __tauri_command_name_create_round, create_round};
pub use create_season::{__cmd__create_season, __tauri_command_name_create_season, create_season};
pub use create_stage::{__cmd__create_stage, __tauri_command_name_create_stage, create_stage};
pub use delete_competition::{
    __cmd__delete_competition, __tauri_command_name_delete_competition, delete_competition,
};
pub use rules::{
    __cmd__create_competition_binding, __cmd__register_rule_package,
    __tauri_command_name_create_competition_binding, __tauri_command_name_register_rule_package,
    create_competition_binding, register_rule_package,
};
