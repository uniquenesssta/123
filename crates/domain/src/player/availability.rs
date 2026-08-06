use super::AvailabilityStatus;
use crate::default_confidence;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerAvailabilityDraft {
    pub player_id: Uuid,
    #[serde(default)]
    pub team_id: Option<Uuid>,
    #[serde(default)]
    pub competition_id: Option<Uuid>,
    pub status: AvailabilityStatus,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default = "default_confidence")]
    pub confidence: f64,
    pub valid_from: DateTime<Utc>,
    #[serde(default)]
    pub valid_to: Option<DateTime<Utc>>,
    #[serde(default)]
    pub source_document_id: Option<Uuid>,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerAvailabilityRecord {
    pub id: Uuid,
    pub player_id: Uuid,
    pub team_id: Option<Uuid>,
    pub team_name: Option<String>,
    pub competition_id: Option<Uuid>,
    pub status: AvailabilityStatus,
    pub reason: Option<String>,
    pub confidence: f64,
    pub valid_from: DateTime<Utc>,
    pub valid_to: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}
