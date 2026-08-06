use super::default_coach_role;
use crate::{default_confidence, default_true};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamCoachPeriodDraft {
    pub team_id: Uuid,
    pub coach_id: Uuid,
    #[serde(default = "default_coach_role")]
    pub role: String,
    pub valid_from: chrono::NaiveDate,
    #[serde(default)]
    pub valid_to: Option<chrono::NaiveDate>,
    #[serde(default)]
    pub is_interim: bool,
    #[serde(default = "default_confidence")]
    pub confidence: f64,
    #[serde(default)]
    pub source_document_id: Option<Uuid>,
    #[serde(default = "default_true")]
    pub end_previous: bool,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamCoachPeriodRecord {
    pub id: Uuid,
    pub team_id: Uuid,
    pub team_name: String,
    pub coach_id: Uuid,
    pub coach_name: String,
    pub role: String,
    pub valid_from: chrono::NaiveDate,
    pub valid_to: Option<chrono::NaiveDate>,
    pub is_interim: bool,
    pub confidence: f64,
}
