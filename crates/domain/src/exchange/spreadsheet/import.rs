use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use super::contract::{SpreadsheetAction, SpreadsheetEntityType, SpreadsheetImportMode, SpreadsheetRowStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpreadsheetRawRow {
    pub sheet_name: String,
    pub row_number: u32,
    pub entity_type: SpreadsheetEntityType,
    pub action: SpreadsheetAction,
    pub values: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpreadsheetParsedWorkbook {
    pub format_version: String,
    pub source_file_name: String,
    pub source_sha256: String,
    pub rows: Vec<SpreadsheetRawRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpreadsheetConflictCandidate {
    pub entity_id: Uuid,
    pub display_name: String,
    #[serde(default)]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpreadsheetImportRow {
    pub id: Uuid,
    pub sheet_name: String,
    pub row_number: u32,
    pub entity_type: SpreadsheetEntityType,
    pub action: SpreadsheetAction,
    pub status: SpreadsheetRowStatus,
    #[serde(default)]
    pub message: Option<String>,
    pub payload: Value,
    #[serde(default)]
    pub matched_entity_id: Option<Uuid>,
    #[serde(default)]
    pub conflict_candidates: Vec<SpreadsheetConflictCandidate>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SpreadsheetImportCounts {
    pub total: u64,
    pub ready_add: u64,
    pub ready_update: u64,
    pub ready_end_previous: u64,
    pub conflict: u64,
    pub error: u64,
    pub skipped: u64,
    pub imported: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpreadsheetImportPreview {
    pub batch_id: Uuid,
    pub source_file_name: String,
    pub source_sha256: String,
    pub import_mode: SpreadsheetImportMode,
    pub counts: SpreadsheetImportCounts,
    pub rows: Vec<SpreadsheetImportRow>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpreadsheetImportResolution {
    pub row_id: Uuid,
    #[serde(default)]
    pub selected_entity_id: Option<Uuid>,
    #[serde(default)]
    pub skip: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpreadsheetImportCommitResult {
    pub batch_id: Uuid,
    pub inserted_count: u64,
    pub updated_count: u64,
    pub ended_previous_count: u64,
    pub skipped_count: u64,
    pub error_count: u64,
    pub finished_at: DateTime<Utc>,
}
