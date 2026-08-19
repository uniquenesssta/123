mod assign_player_position;
mod input_policy;
mod list_positions;
mod mapper;
mod reference_mapper;
mod reference_row;
mod row;

pub(in crate::adapters::catalog::players) use mapper::map_player_position;
pub(in crate::adapters::catalog::players) use row::PlayerPositionRow;
