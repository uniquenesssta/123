mod commit;
mod conflict;
mod formation;
mod identity;
mod names;
mod preview;
mod validation;
mod values;
mod write;

use football_domain::{SpreadsheetConflictCandidate, SpreadsheetRowStatus};
use serde_json::Value;
use uuid::Uuid;

#[derive(Default)]
struct CommitOutcome {
    inserted: u64,
    updated: u64,
    ended_previous: u64,
}
struct RowValidation {
    status: SpreadsheetRowStatus,
    message: Option<String>,
    payload: Value,
    matched_entity_id: Option<Uuid>,
    conflict_candidates: Vec<SpreadsheetConflictCandidate>,
}
impl RowValidation {
    fn skip(payload: Value) -> Self {
        Self {
            status: SpreadsheetRowStatus::Skip,
            message: Some("action=skip".into()),
            payload,
            matched_entity_id: None,
            conflict_candidates: vec![],
        }
    }
    fn error(payload: Value, message: &str) -> Self {
        Self {
            status: SpreadsheetRowStatus::Error,
            message: Some(message.into()),
            payload,
            matched_entity_id: None,
            conflict_candidates: vec![],
        }
    }
    fn ready_add(payload: Value, message: &str) -> Self {
        Self {
            status: SpreadsheetRowStatus::ReadyAdd,
            message: Some(message.into()),
            payload,
            matched_entity_id: None,
            conflict_candidates: vec![],
        }
    }
}
