use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct RuntimeOperationDraft {
    pub(super) phase: String,
    pub(super) operation: String,
    pub(super) trace_id: Option<String>,
    pub(super) duration_ms: Option<u64>,
    pub(super) context: Option<Value>,
}
