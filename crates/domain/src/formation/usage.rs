use super::{
    default_formation_alpha, default_formation_usage_limit, default_formation_window_preset,
};
use crate::default_confidence;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormationUsageEntryDraft {
    pub formation_id: Uuid,
    pub usage_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormationUsageDistributionDraft {
    pub scope_type: String,
    #[serde(default)]
    pub team_id: Option<Uuid>,
    #[serde(default)]
    pub coach_id: Option<Uuid>,
    #[serde(default)]
    pub competition_id: Option<Uuid>,
    #[serde(default = "default_formation_window_preset")]
    pub window_preset: String,
    #[serde(default)]
    pub window_start: Option<chrono::NaiveDate>,
    #[serde(default)]
    pub window_end: Option<chrono::NaiveDate>,
    pub observed_matches: i32,
    #[serde(default = "default_confidence")]
    pub confidence: f64,
    #[serde(default = "default_formation_alpha")]
    pub alpha: f64,
    #[serde(default)]
    pub source_document_id: Option<Uuid>,
    #[serde(default)]
    pub metadata: Value,
    pub entries: Vec<FormationUsageEntryDraft>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormationUsageEntryRecord {
    pub id: Uuid,
    pub formation_id: Uuid,
    pub formation_code: String,
    pub formation_name: String,
    pub usage_count: i32,
    pub raw_probability: f64,
    pub smoothed_probability: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormationUsageDistributionRecord {
    pub scope_type: String,
    pub team_id: Option<Uuid>,
    pub team_name: Option<String>,
    pub coach_id: Option<Uuid>,
    pub coach_name: Option<String>,
    pub competition_id: Option<Uuid>,
    pub competition_name: Option<String>,
    pub window_preset: String,
    pub window_start: chrono::NaiveDate,
    pub window_end: chrono::NaiveDate,
    pub observed_matches: i32,
    pub confidence: f64,
    pub alpha: f64,
    pub observed_at: DateTime<Utc>,
    pub entries: Vec<FormationUsageEntryRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormationUsageListQuery {
    #[serde(default)]
    pub team_id: Option<Uuid>,
    #[serde(default)]
    pub coach_id: Option<Uuid>,
    #[serde(default)]
    pub competition_id: Option<Uuid>,
    #[serde(default = "default_formation_usage_limit")]
    pub limit: u32,
}
