use crate::{PersistenceResult, PostgresStore};
use football_domain::P4EvidenceWorkspaceRecord;
use sqlx::Row;
use uuid::Uuid;

pub(super) async fn read(
    store: &PostgresStore,
    research_run_id: Uuid,
) -> PersistenceResult<Vec<P4EvidenceWorkspaceRecord>> {
    let rows = sqlx::query(
        r#"
                SELECT id, field_key, entity_type, entity_id, value,
                       verification_state, source_tier, source_url, source_title,
                       source_domain, published_at, observed_at, effective_at,
                       retrieved_at, timezone, conflict_group_id, created_at
                FROM research.evidence_claims
                WHERE research_run_id = $1
                ORDER BY field_key, created_at, id
                "#,
    )
    .bind(research_run_id)
    .fetch_all(&store.pool)
    .await?;
    rows.iter()
        .map(|row| {
            Ok(P4EvidenceWorkspaceRecord {
                id: row.try_get("id")?,
                field_key: row.try_get("field_key")?,
                entity_type: row.try_get("entity_type")?,
                entity_id: row.try_get("entity_id")?,
                value: row.try_get("value")?,
                verification_state: row.try_get("verification_state")?,
                source_tier: row.try_get("source_tier")?,
                source_url: row.try_get("source_url")?,
                source_title: row.try_get("source_title")?,
                source_domain: row.try_get("source_domain")?,
                published_at: row.try_get("published_at")?,
                observed_at: row.try_get("observed_at")?,
                effective_at: row.try_get("effective_at")?,
                retrieved_at: row.try_get("retrieved_at")?,
                timezone: row.try_get("timezone")?,
                conflict_group_id: row.try_get("conflict_group_id")?,
                created_at: row.try_get("created_at")?,
            })
        })
        .collect::<PersistenceResult<Vec<_>>>()
}
