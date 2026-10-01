mod commit;
mod conflict;
mod context;
mod identity;
mod preview;
mod read;
mod validation;
mod values;
mod write;

use football_domain::{SpreadsheetConflictCandidate, SpreadsheetRowStatus};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug)]
struct Validation {
    status: SpreadsheetRowStatus,
    message: Option<String>,
    payload: Value,
    matched_entity_id: Option<Uuid>,
    candidates: Vec<SpreadsheetConflictCandidate>,
}

#[derive(Debug, Default)]
struct ApplyOutcome {
    was_update: bool,
    ended_previous: u64,
    lineup_id: Option<Uuid>,
}
