use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use super::kind::CompetitionKind;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitionDraft {
    pub code: String,
    pub name: String,
    #[serde(default)]
    pub country_code: Option<String>,
    #[serde(default = "default_timezone")]
    pub timezone: String,
    pub competition_kind: CompetitionKind,
    #[serde(default)]
    pub metadata: Value,
}

fn default_timezone() -> String {
    "UTC".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitionRecord {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub country_code: Option<String>,
    pub timezone: String,
    pub competition_kind: CompetitionKind,
    pub is_active: bool,
    #[serde(default)]
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
}
