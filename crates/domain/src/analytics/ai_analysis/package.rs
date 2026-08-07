use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use super::super::calculation::AnalyticsOverview;

pub const AI_ANALYSIS_PACKAGE_FORMAT: &str = "football.ai-analysis-package.v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiAnalysisPackageManifest {
    pub format_version: String,
    pub package_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub calculation_version: String,
    pub sample_size: u64,
    pub content_sha256: String,
    pub files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiAnalysisPackageData {
    pub overview: AnalyticsOverview,
    pub database_summary: Value,
    pub player_review_summary: Value,
    pub team_review_summary: Value,
    pub ability_candidates: Value,
    pub schema_summary: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiAnalysisPackageSummary {
    pub package_id: Uuid,
    pub output_path: String,
    pub content_sha256: String,
    pub sample_size: u64,
    pub created_at: DateTime<Utc>,
}
