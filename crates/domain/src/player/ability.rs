use super::{default_context_type, default_sample_size};
use crate::default_confidence;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerAbilityObservationDraft {
    pub player_id: Uuid,
    pub dimension_code: String,
    #[serde(default = "default_context_type")]
    pub context_type: String,
    #[serde(default)]
    pub context_id: Option<Uuid>,
    pub value: f64,
    #[serde(default = "default_confidence")]
    pub confidence: f64,
    #[serde(default = "default_sample_size")]
    pub sample_size: i32,
    pub observed_at: DateTime<Utc>,
    pub effective_from: DateTime<Utc>,
    #[serde(default)]
    pub effective_to: Option<DateTime<Utc>>,
    pub calculation_version: String,
    #[serde(default)]
    pub source_document_id: Option<Uuid>,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerAbilityObservationRecord {
    pub id: Uuid,
    pub player_id: Uuid,
    pub dimension_code: String,
    pub dimension_name: String,
    pub context_type: String,
    pub context_id: Option<Uuid>,
    pub value: f64,
    pub confidence: f64,
    pub sample_size: i32,
    pub observed_at: DateTime<Utc>,
    pub effective_from: DateTime<Utc>,
    pub effective_to: Option<DateTime<Utc>>,
    pub calculation_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerAbilityProfile {
    pub player_id: Uuid,
    pub abilities: Value,
    pub average_value: Option<f64>,
    pub average_confidence: Option<f64>,
    pub dimension_count: i32,
    pub latest_observed_at: Option<DateTime<Utc>>,
    pub next_expiry_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}
