use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

pub const AI_ANALYSIS_RESPONSE_FORMAT: &str = "football.ai-analysis-response.v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AiAnalysisSuggestionDraft {
    pub suggestion_type: String,
    pub title: String,
    pub summary: String,
    #[serde(default)]
    pub severity: String,
    #[serde(default)]
    pub scope: Value,
    #[serde(default)]
    pub payload: Value,
    #[serde(default)]
    pub evidence: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AiAnalysisResponseManifest {
    pub format_version: String,
    pub response_id: Uuid,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub source_package_id: Option<Uuid>,
    pub content_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiAnalysisResponsePreview {
    pub manifest: AiAnalysisResponseManifest,
    pub suggestions: Vec<AiAnalysisSuggestionDraft>,
    pub blocking_errors: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiAnalysisSuggestionRecord {
    pub id: Uuid,
    pub response_id: Uuid,
    pub suggestion_type: String,
    pub title: String,
    pub summary: String,
    pub severity: String,
    pub scope: Value,
    pub payload: Value,
    pub evidence: Value,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub decided_at: Option<DateTime<Utc>>,
    pub decision_note: Option<String>,
    pub linked_candidate_id: Option<Uuid>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AiSuggestionDecision {
    Accept,
    Reject,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiSuggestionDecisionDraft {
    pub suggestion_id: Uuid,
    pub decision: AiSuggestionDecision,
    #[serde(default)]
    pub decided_by: Option<String>,
    #[serde(default)]
    pub decision_note: Option<String>,
}
