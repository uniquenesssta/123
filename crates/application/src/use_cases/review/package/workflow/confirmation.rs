use football_domain::{MatchReviewPackagePreview, MatchReviewPackageWorkflowRecord};
use serde_json::{json, Value};

pub(in crate::use_cases::review) fn apply_confirmation_metadata(
    preview: &mut MatchReviewPackagePreview,
    workflow: &MatchReviewPackageWorkflowRecord,
) {
    let confirmation = json!({
        "package_id": workflow.package_id,
        "source_sha256": workflow.import_sha256.as_deref(),
        "confirmed_by": workflow.confirmed_by.as_deref(),
        "confirmation_note": workflow.confirmation_note.as_deref(),
        "confirmed_at": workflow.confirmed_at.as_ref(),
    });
    merge_metadata(
        &mut preview.review.result.metadata,
        "review_package_confirmation",
        confirmation.clone(),
    );
    merge_metadata(
        &mut preview.lineup_pair.home.metadata,
        "review_package_confirmation",
        confirmation.clone(),
    );
    merge_metadata(
        &mut preview.lineup_pair.away.metadata,
        "review_package_confirmation",
        confirmation,
    );
    if let Some(note) = workflow
        .confirmation_note
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        preview.review.notes = Some(match preview.review.notes.take() {
            Some(existing) if !existing.trim().is_empty() => {
                format!("{}\n人工确认：{}", existing.trim(), note)
            }
            _ => format!("人工确认：{note}"),
        });
    }
}

fn merge_metadata(target: &mut Value, key: &str, value: Value) {
    if !target.is_object() {
        *target = json!({});
    }
    if let Some(object) = target.as_object_mut() {
        object.insert(key.to_string(), value);
    }
}
