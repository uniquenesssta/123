use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use super::{kind::CompetitionKind, profile::CompetitionProfile};
use crate::routing::RuleRouting;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleSourceReference {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub source_uri: Option<String>,
    #[serde(default)]
    pub content_sha256: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RulePackageDraft {
    #[serde(default = "default_rule_package_format")]
    pub format_version: String,
    pub package_key: String,
    pub version: String,
    pub display_name: String,
    pub competition_profile: CompetitionProfile,
    pub routing: RuleRouting,
    pub parameters: Value,
    #[serde(default)]
    pub feature_requirements: Value,
    #[serde(default)]
    pub output_contract: Value,
    #[serde(default)]
    pub source_document: Option<RuleSourceReference>,
    #[serde(default)]
    pub metadata: Value,
}

fn default_rule_package_format() -> String {
    "football.rule-package.v1".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RulePackageSummary {
    pub id: Uuid,
    pub format_version: String,
    pub package_key: String,
    pub version: String,
    pub display_name: String,
    pub competition_kind: CompetitionKind,
    pub model_id: String,
    pub model_version: String,
    pub parameter_version: String,
    pub priority: i32,
    pub content_sha256: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}
