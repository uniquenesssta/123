use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use super::kind::CompetitionKind;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageDraft {
    pub season_id: Uuid,
    pub code: String,
    pub name: String,
    pub stage_kind: CompetitionKind,
    #[serde(default)]
    pub sequence_no: i32,
    #[serde(default)]
    pub rules: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageRecord {
    pub id: Uuid,
    pub season_id: Uuid,
    pub season_name: String,
    pub competition_id: Uuid,
    pub competition_name: String,
    pub code: String,
    pub name: String,
    pub stage_kind: CompetitionKind,
    pub sequence_no: i32,
}
