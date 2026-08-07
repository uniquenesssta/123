use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceTierRule {
    pub domain: String,
    pub tier: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceTierDefinition {
    pub key: String,
    pub rank: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourcePolicyDefinition {
    pub schema_version: String,
    pub default_tier: String,
    pub tiers: Vec<SourceTierDefinition>,
    pub domain_rules: Vec<SourceTierRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourcePolicyVersionDraft {
    pub policy_key: String,
    pub version: String,
    #[serde(default)]
    pub competition_profile_id: Option<Uuid>,
    pub definition: SourcePolicyDefinition,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourcePolicyVersionRecord {
    pub id: Uuid,
    pub policy_key: String,
    pub version: String,
    pub content_sha256: String,
    pub created_at: DateTime<Utc>,
}
