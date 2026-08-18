use crate::{
    adapters::catalog::coaches::{
        mapping::coach_from_row,
        normalization::{normalize_name, trim_option},
        validation::validate_coach_draft,
    },
    write_audit_event, PersistenceResult, PostgresStore,
};
use football_domain::{CoachDraft, CoachRecord};
use serde_json::json;
use uuid::Uuid;
impl PostgresStore {
    pub async fn create_coach(&self, draft: &CoachDraft) -> PersistenceResult<CoachRecord> {
        validate_coach_draft(draft)?;
        let name = draft.canonical_name.trim();
        let normalized = normalize_name(name);
        let id = Uuid::new_v4();
        let mut tx = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
                INSERT INTO football.coaches (
                    id, canonical_name, normalized_name, nationality_code, status, metadata
                ) VALUES ($1,$2,$3,$4,$5,$6)
                RETURNING id, canonical_name, normalized_name, nationality_code, status,
                          metadata, created_at, updated_at
                "#,
        )
        .bind(id)
        .bind(name)
        .bind(&normalized)
        .bind(trim_option(&draft.nationality_code))
        .bind(draft.status.trim())
        .bind(&draft.metadata)
        .fetch_one(&mut *tx)
        .await?;
        sqlx::query(
            r#"
                INSERT INTO football.coach_names (
                    id, coach_id, name, normalized_name, is_primary
                ) VALUES ($1,$2,$3,$4,true)
                "#,
        )
        .bind(Uuid::new_v4())
        .bind(id)
        .bind(name)
        .bind(&normalized)
        .execute(&mut *tx)
        .await?;
        write_audit_event(
            &mut tx,
            "coach_created",
            "coach",
            Some(id.to_string()),
            json!({"canonical_name": name}),
        )
        .await?;
        tx.commit().await?;
        coach_from_row(&row)
    }
}
