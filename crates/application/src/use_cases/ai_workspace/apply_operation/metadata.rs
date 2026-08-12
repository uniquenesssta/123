use super::payload::optional_text;
use crate::{ApplicationError, ApplicationResult};
use football_domain::ApiWorkspaceOperationRecord;
use serde_json::{json, Map, Value};

pub(super) fn operation_metadata(
    operation: &ApiWorkspaceOperationRecord,
) -> ApplicationResult<Value> {
    let mut metadata = optional_text(&operation.payload, "metadata_json")
        .map(|raw| serde_json::from_str::<Value>(&raw))
        .transpose()?
        .unwrap_or_else(|| json!({}));
    if !metadata.is_object() {
        return Err(ApplicationError::Validation(
            "metadata_json必须是JSON对象".to_string(),
        ));
    }
    let object: &mut Map<String, Value> = metadata.as_object_mut().expect("checked object");
    object.insert(
        "api_workspace_operation_id".to_string(),
        json!(operation.id),
    );
    object.insert(
        "api_workspace_session_id".to_string(),
        json!(operation.session_id),
    );
    object.insert(
        "api_workspace_rationale".to_string(),
        json!(operation.rationale),
    );
    object.insert(
        "source_urls".to_string(),
        operation
            .payload
            .get("source_urls")
            .cloned()
            .unwrap_or_else(|| json!([])),
    );
    Ok(metadata)
}
