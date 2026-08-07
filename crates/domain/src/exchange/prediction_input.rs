use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreparedMatchPredictionInput {
    pub match_record: crate::MatchRecord,
    pub competition_kind: crate::CompetitionKind,
    pub snapshot_type: String,
    pub match_input: Value,
    pub data_quality: Value,
}
