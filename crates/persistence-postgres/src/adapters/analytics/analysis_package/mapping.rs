use crate::PersistenceResult;
use football_domain::AiAnalysisSuggestionRecord;
use sqlx::Row;

pub(super) fn ai_suggestion_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<AiAnalysisSuggestionRecord> {
    Ok(AiAnalysisSuggestionRecord {
        id: row.try_get("id")?,
        response_id: row.try_get("response_id")?,
        suggestion_type: row.try_get("suggestion_type")?,
        title: row.try_get("title")?,
        summary: row.try_get("summary")?,
        severity: row.try_get("severity")?,
        scope: row.try_get("scope")?,
        payload: row.try_get("payload")?,
        evidence: row.try_get("evidence")?,
        status: row.try_get("status")?,
        created_at: row.try_get("created_at")?,
        decided_at: row.try_get("decided_at")?,
        decision_note: row.try_get("decision_note")?,
        linked_candidate_id: row.try_get("linked_candidate_id")?,
    })
}
