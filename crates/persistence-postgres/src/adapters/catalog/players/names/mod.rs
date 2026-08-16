mod add_player_name;
mod input_policy;
mod mapper;
mod row;

pub(in crate::adapters::catalog::players) use mapper::map_player_name;
pub(in crate::adapters::catalog::players) use row::PlayerNameRow;
