use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseStats {
    pub competitions: i64,
    pub teams: i64,
    pub players: i64,
    pub matches: i64,
    pub model_runs: i64,
    pub rule_packages: i64,
    pub route_bindings: i64,
    pub ability_observations: i64,
    pub pending_ability_updates: i64,
    pub data_providers: i64,
    pub availability_records: i64,
    pub active_lineups: i64,
    pub large_counts_are_estimates: bool,
}
