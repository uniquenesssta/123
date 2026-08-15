use super::mapper::map_team_name;
use super::normalization::normalize_team_name;
use super::row::TeamNameRow;
use super::validation::{normalized_language_code, validated_name};
use crate::{PersistenceResult, PostgresStore};
use football_domain::{TeamNameDraft, TeamNameRecord};
use serde_json::json;
use uuid::Uuid;

impl PostgresStore {
    pub async fn add_team_name(&self, draft: &TeamNameDraft) -> PersistenceResult<TeamNameRecord> {
        let name = validated_name(draft)?;
        let mut tx = self.pool.begin().await?;
        let row = sqlx::query_as::<_, TeamNameRow>(
            r#"
            INSERT INTO football.team_names (
                id, team_id, name, normalized_name, language_code, valid_from, valid_to
            ) VALUES ($1,$2,$3,$4,$5,$6,$7)
            RETURNING id, team_id, name, normalized_name, language_code, valid_from, valid_to
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(draft.team_id)
        .bind(name)
        .bind(normalize_team_name(name))
        .bind(normalized_language_code(draft))
        .bind(draft.valid_from)
        .bind(draft.valid_to)
        .fetch_one(&mut *tx)
        .await?;
        crate::write_audit_event(
            &mut tx,
            "team_name_added",
            "team",
            draft.team_id.to_string(),
            json!({"name": name, "language_code": draft.language_code, "source": "manual"}),
        )
        .await?;
        tx.commit().await?;
        Ok(map_team_name(row))
    }
}
