use crate::{PersistenceResult, PostgresStore};
use football_domain::P4ConflictWorkspaceRecord;
use sqlx::Row;
use uuid::Uuid;

pub(super) async fn read(
    store: &PostgresStore,
    research_run_id: Uuid,
    task_id: Uuid,
) -> PersistenceResult<Vec<P4ConflictWorkspaceRecord>> {
    let rows = sqlx::query(
        r#"
                SELECT conflict.id, conflict.field_key, conflict.entity_type,
                       conflict.entity_id, conflict.conflict_key, conflict.created_at,
                       COALESCE(latest_event.event_type, 'opened') AS conflict_status,
                       latest_evaluation.evaluation_status,
                       members.evidence_ids,
                       manual.decision_kind AS manual_decision_kind,
                       manual.selected_evidence_ids,
                       manual.note AS manual_decision_note,
                       manual.created_at AS manual_decision_at
                FROM research.evidence_conflicts conflict
                JOIN LATERAL (
                    SELECT COALESCE(array_agg(member.evidence_id ORDER BY member.evidence_id), '{}'::uuid[]) AS evidence_ids
                    FROM research.evidence_conflict_members member
                    JOIN research.evidence_claims claim ON claim.id = member.evidence_id
                    WHERE member.conflict_id = conflict.id
                      AND claim.research_run_id = $1
                ) members ON cardinality(members.evidence_ids) > 0
                LEFT JOIN LATERAL (
                    SELECT event.event_type
                    FROM research.evidence_conflict_events event
                    WHERE event.conflict_id = conflict.id
                    ORDER BY event.occurred_at DESC, event.id DESC
                    LIMIT 1
                ) latest_event ON true
                LEFT JOIN LATERAL (
                    SELECT evaluation.evaluation_status
                    FROM research.conflict_evaluations evaluation
                    WHERE evaluation.conflict_id = conflict.id
                      AND evaluation.research_run_id = $1
                    ORDER BY evaluation.created_at DESC, evaluation.id DESC
                    LIMIT 1
                ) latest_evaluation ON true
                LEFT JOIN LATERAL (
                    SELECT manual_override.decision_kind, manual_override.selected_evidence_ids,
                           manual_override.note, manual_override.created_at
                    FROM research.manual_route_overrides manual_override
                    WHERE manual_override.task_id = $2
                      AND manual_override.conflict_id = conflict.id
                    ORDER BY manual_override.created_at DESC, manual_override.id DESC
                    LIMIT 1
                ) manual ON true
                ORDER BY conflict.field_key, conflict.created_at, conflict.id
                "#,
    )
    .bind(research_run_id)
    .bind(task_id)
    .fetch_all(&store.pool)
    .await?;
    rows.iter()
        .map(|row| {
            Ok(P4ConflictWorkspaceRecord {
                id: row.try_get("id")?,
                field_key: row.try_get("field_key")?,
                entity_type: row.try_get("entity_type")?,
                entity_id: row.try_get("entity_id")?,
                conflict_key: row.try_get("conflict_key")?,
                status: row.try_get("conflict_status")?,
                evaluation_status: row.try_get("evaluation_status")?,
                evidence_ids: row.try_get("evidence_ids")?,
                selected_evidence_ids: row
                    .try_get::<Option<Vec<Uuid>>, _>("selected_evidence_ids")?
                    .unwrap_or_default(),
                manual_decision_kind: row.try_get("manual_decision_kind")?,
                manual_decision_note: row.try_get("manual_decision_note")?,
                manual_decision_at: row.try_get("manual_decision_at")?,
                created_at: row.try_get("created_at")?,
            })
        })
        .collect::<PersistenceResult<Vec<_>>>()
}
