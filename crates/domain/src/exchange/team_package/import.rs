use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::super::spreadsheet::{SpreadsheetImportCommitResult, SpreadsheetImportPreview};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TeamPackageCoverage {
    pub team_count: u64,
    pub player_count: u64,
    pub coach_count: u64,
    pub formation_usage_count: u64,
    pub team_ability_count: u64,
    pub player_ability_count: u64,
    pub player_dynamic_tag_count: u64,
    #[serde(default)]
    pub player_role_count: u64,
    pub availability_count: u64,
    pub readiness_score: u8,
    pub p4_input_ready: bool,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamPackageImportPreview {
    pub source_file_name: String,
    pub source_sha256: String,
    pub team_preview: Option<SpreadsheetImportPreview>,
    pub player_preview: Option<SpreadsheetImportPreview>,
    pub coverage: TeamPackageCoverage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamPackageCommitRequest {
    #[serde(default)]
    pub team_batch_id: Option<Uuid>,
    #[serde(default)]
    pub player_batch_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamPackageCommitResult {
    pub team_result: Option<SpreadsheetImportCommitResult>,
    pub player_result: Option<SpreadsheetImportCommitResult>,
    pub inserted_count: u64,
    pub updated_count: u64,
    pub ended_previous_count: u64,
    pub skipped_count: u64,
    pub error_count: u64,
}
