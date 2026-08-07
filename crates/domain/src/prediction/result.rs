use crate::ModelIdentity;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionSummary {
    pub home_win: f64,
    pub draw: f64,
    pub away_win: f64,
    pub btts: Option<f64>,
    pub over_2_5: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedModelRun {
    pub id: Uuid,
    pub match_key: String,
    pub identity: ModelIdentity,
    pub snapshot_type: String,
    pub created_at: DateTime<Utc>,
    pub summary: PredictionSummary,
    pub output: Value,
}
