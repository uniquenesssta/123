mod add_player_availability;
mod input_policy;
mod mapper;
mod row;

pub(in crate::adapters::catalog) use mapper::map_player_availability;
pub(in crate::adapters::catalog) use row::PlayerAvailabilityRow;
