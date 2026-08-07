use super::{LineupPlayerDraft, LineupPlayerRecord, LineupType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineupDraft {
    pub match_id: Uuid,
    pub team_id: Uuid,
    pub lineup_type: LineupType,
    #[serde(default = "default_lineup_snapshot_type")]
    pub snapshot_type: String,
    #[serde(default)]
    pub formation: Option<String>,
    #[serde(default)]
    pub formation_id: Option<Uuid>,
    #[serde(default)]
    pub coach_id: Option<Uuid>,
    pub captured_at: DateTime<Utc>,
    #[serde(default)]
    pub source_document_id: Option<Uuid>,
    #[serde(default)]
    pub source_urls: Vec<String>,
    #[serde(default)]
    pub quality_score: Option<f64>,
    #[serde(default)]
    pub metadata: Value,
    pub players: Vec<LineupPlayerDraft>,
}

fn default_lineup_snapshot_type() -> String {
    "T-1h".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineupPairDraft {
    pub home: LineupDraft,
    pub away: LineupDraft,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineupPairRecord {
    pub home: LineupRecord,
    pub away: LineupRecord,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineupRecord {
    pub id: Uuid,
    pub match_id: Uuid,
    pub match_key: String,
    pub team_id: Uuid,
    pub team_name: String,
    pub lineup_type: LineupType,
    pub snapshot_type: String,
    pub formation: Option<String>,
    pub formation_id: Option<Uuid>,
    pub formation_code: Option<String>,
    pub formation_name: Option<String>,
    pub coach_id: Option<Uuid>,
    pub coach_name: Option<String>,
    pub captured_at: DateTime<Utc>,
    pub status: String,
    pub quality_score: Option<f64>,
    pub source_urls: Vec<String>,
    pub supersedes_lineup_id: Option<Uuid>,
    pub model_validation_status: String,
    pub model_eligible: bool,
    pub validation_errors: Vec<String>,
    pub validation_warnings: Vec<String>,
    pub player_count: i64,
    pub starter_count: i64,
    pub players: Vec<LineupPlayerRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineupHistoryRemovalResult {
    pub lineup_id: Uuid,
    pub removal_mode: String,
    pub restored_lineup_id: Option<Uuid>,
}
