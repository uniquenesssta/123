use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::competition::CompetitionKind;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitionBindingDraft {
    #[serde(default)]
    pub binding_name: Option<String>,
    #[serde(default)]
    pub competition_id: Option<Uuid>,
    #[serde(default)]
    pub season_id: Option<Uuid>,
    #[serde(default)]
    pub stage_id: Option<Uuid>,
    #[serde(default)]
    pub competition_kind: Option<CompetitionKind>,
    pub rule_package_id: Uuid,
    #[serde(default)]
    pub priority: i32,
    #[serde(default)]
    pub valid_from: Option<DateTime<Utc>>,
    #[serde(default)]
    pub valid_to: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitionBindingSummary {
    pub id: Uuid,
    pub binding_name: String,
    pub competition_id: Option<Uuid>,
    pub competition_name: Option<String>,
    pub season_id: Option<Uuid>,
    pub stage_id: Option<Uuid>,
    pub competition_kind: Option<CompetitionKind>,
    pub rule_package_id: Uuid,
    pub rule_package_name: String,
    pub model_id: String,
    pub priority: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}
