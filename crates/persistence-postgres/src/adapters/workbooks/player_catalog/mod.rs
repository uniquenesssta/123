mod commit;
mod conflict;
mod identity;
mod preview;
mod validation;
mod values;

use football_domain::{SpreadsheetConflictCandidate, SpreadsheetImportMode, SpreadsheetRowStatus};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

struct SpreadsheetValidationContext<'a> {
    mode: SpreadsheetImportMode,
    player_keys: &'a HashSet<String>,
    team_keys: &'a HashSet<String>,
    duplicate_player_keys: &'a HashSet<String>,
    duplicate_team_keys: &'a HashSet<String>,
    external_team_references: &'a HashMap<String, String>,
}

#[derive(Debug)]
struct RowValidation {
    status: SpreadsheetRowStatus,
    message: Option<String>,
    payload: Value,
    matched_entity_id: Option<Uuid>,
    conflict_candidates: Vec<SpreadsheetConflictCandidate>,
}

impl RowValidation {
    fn skip(payload: Value, message: &str) -> Self {
        Self {
            status: SpreadsheetRowStatus::Skip,
            message: Some(message.to_string()),
            payload,
            matched_entity_id: None,
            conflict_candidates: Vec::new(),
        }
    }
    fn error(payload: Value, message: &str) -> Self {
        Self {
            status: SpreadsheetRowStatus::Error,
            message: Some(message.to_string()),
            payload,
            matched_entity_id: None,
            conflict_candidates: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
struct MatchCandidate {
    id: Uuid,
    name: String,
    detail: Option<String>,
}

enum ReferenceResolution {
    Resolved(Uuid),
    Deferred(String),
    DeferredExternal { key: String, name: String },
    Conflict(Vec<MatchCandidate>),
    Missing(String),
}
