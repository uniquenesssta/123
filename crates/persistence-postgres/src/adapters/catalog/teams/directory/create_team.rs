use super::super::detail::{map_team_record, TeamRecordRow};
use super::super::names::normalize_team_name;
use crate::{PersistenceError, PersistenceResult, PostgresStore};
use football_domain::{TeamDraft, TeamRecord};
use serde_json::json;
use uuid::Uuid;

impl PostgresStore {
    pub async fn create_team(&self, draft: &TeamDraft) -> PersistenceResult<TeamRecord> {
        let canonical_name = draft.canonical_name.trim();
        if canonical_name.is_empty() {
            return Err(PersistenceError::InvalidState(
                "球队名称不能为空".to_string(),
            ));
        }
        let normalized_name = normalize_team_name(canonical_name);
        let id = Uuid::new_v4();
        let mut tx = self.pool.begin().await?;
        let row = sqlx::query_as::<_, TeamRecordRow>(
            r#"
            INSERT INTO football.teams (
                id, canonical_name, normalized_name, country_code, metadata
            ) VALUES ($1, $2, $3, $4, $5)
            RETURNING id, canonical_name, normalized_name, country_code, is_active, created_at
            "#,
        )
        .bind(id)
        .bind(canonical_name)
        .bind(&normalized_name)
        .bind(
            draft
                .country_code
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty()),
        )
        .bind(&draft.metadata)
        .fetch_one(&mut *tx)
        .await?;
        crate::write_audit_event(
            &mut tx,
            "team_created",
            "team",
            id.to_string(),
            json!({"canonical_name": canonical_name}),
        )
        .await?;
        tx.commit().await?;
        map_team_record(row)
    }
}
