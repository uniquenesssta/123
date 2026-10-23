mod add_coach_name;
mod add_team_coach_period;
mod create_coach;
mod list_coaches;
mod read_coach;

pub use add_coach_name::{
    __cmd__add_coach_name, __tauri_command_name_add_coach_name, add_coach_name,
};
pub use add_team_coach_period::{
    __cmd__add_team_coach_period, __tauri_command_name_add_team_coach_period, add_team_coach_period,
};
pub use create_coach::{__cmd__create_coach, __tauri_command_name_create_coach, create_coach};
pub use list_coaches::{__cmd__list_coaches, __tauri_command_name_list_coaches, list_coaches};
pub use read_coach::{__cmd__read_coach, __tauri_command_name_read_coach, read_coach};
