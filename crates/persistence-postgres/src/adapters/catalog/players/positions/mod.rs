mod assign_player_position;
mod input_policy;
mod mapper;
mod row;

pub(in crate::adapters::catalog::players) use mapper::map_player_position;
pub(in crate::adapters::catalog::players) use row::PlayerPositionRow;
