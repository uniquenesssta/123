use super::super::detail::{map_team_record, TeamRecordRow};
use super::name_policy::normalize_team_name;
use crate::{PersistenceError, PersistenceResult, PostgresStore};
use football_domain::{TeamDraft, TeamRecord};
use serde_json::json;
use uuid::Uuid;

impl PostgresStore {
    pub async fn update_team(
        &self,
        team_id: Uuid,
        draft: &TeamDraft,
    ) -> PersistenceResult<TeamRecord> {
        let canonical_name = draft.canonical_name.trim();
        if canonical_name.is_empty() {
            return Err(PersistenceError::InvalidState(
                "球队名称不能为空".to_string(),
            ));
        }
        let mut tx = self.pool.begin().await?;
        let row = sqlx::query_as::<_, TeamRecordRow>(
            r#"
            UPDATE football.teams
            SET canonical_name = $2, normalized_name = $3, country_code = $4,
                metadata = metadata || $5, updated_at = now()
            WHERE id = $1
            RETURNING id, canonical_name, normalized_name, country_code, is_active, created_at
            "#,
        )
        .bind(team_id)
        .bind(canonical_name)
        .bind(normalize_team_name(canonical_name))
        .bind(
            draft
                .country_code
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty()),
        )
        .bind(&draft.metadata)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("球队不存在".to_string()))?;
        crate::write_audit_event(
            &mut tx,
            "team_updated",
            "team",
            team_id.to_string(),
            json!({"canonical_name": canonical_name, "source": "manual"}),
        )
        .await?;
        tx.commit().await?;
        map_team_record(row)
    }
}
