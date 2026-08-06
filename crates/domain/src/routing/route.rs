use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use super::rules::RuleRouting;
use crate::competition::{CompetitionKind, CompetitionProfile};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteRequest {
    #[serde(default)]
    pub competition_id: Option<Uuid>,
    #[serde(default)]
    pub season_id: Option<Uuid>,
    #[serde(default)]
    pub stage_id: Option<Uuid>,
    pub competition_kind: CompetitionKind,
    pub kickoff_time: DateTime<Utc>,
    #[serde(default)]
    pub preferred_model_family: Option<String>,
    #[serde(default)]
    pub preferred_model_id: Option<String>,
    #[serde(default)]
    pub explicit_rule_package_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RouteSource {
    ExplicitRulePackage,
    StageBinding,
    SeasonBinding,
    CompetitionBinding,
    CompetitionKindDefault,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteDecision {
    pub source: RouteSource,
    pub binding_id: Option<Uuid>,
    pub rule_package_id: Uuid,
    pub package_key: String,
    pub package_version: String,
    pub package_display_name: String,
    pub model_id: String,
    pub model_version_id: Uuid,
    pub model_version: String,
    pub parameter_set_id: Uuid,
    pub parameter_version: String,
    pub competition_profile_id: Uuid,
    pub parameters: Value,
    pub routing: RuleRouting,
    pub competition_profile: CompetitionProfile,
    pub feature_requirements: Value,
    pub output_contract: Value,
    pub priority: i32,
    pub reason: Value,
}
