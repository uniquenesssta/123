use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::kind::CompetitionKind;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitionProfile {
    pub profile_id: String,
    pub name: String,
    pub competition_kind: CompetitionKind,
    #[serde(default = "default_normal_time")]
    pub normal_time_minutes: u16,
    #[serde(default)]
    pub extra_time_possible: bool,
    #[serde(default)]
    pub penalties_possible: bool,
    #[serde(default)]
    pub two_legged: bool,
    #[serde(default)]
    pub neutral_venue: bool,
    #[serde(default)]
    pub metadata: Value,
}

fn default_normal_time() -> u16 {
    90
}
