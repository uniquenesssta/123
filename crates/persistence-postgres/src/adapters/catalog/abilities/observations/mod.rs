mod add;
mod input_policy;
mod mapper;
mod row;

pub(in crate::adapters::catalog) use mapper::map_player_ability_observation;
pub(in crate::adapters::catalog) use row::PlayerAbilityObservationRow;
