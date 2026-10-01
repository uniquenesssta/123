use super::values::normalize_name;
use crate::PersistenceResult;
use chrono::NaiveDate;
use serde_json::Value;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

pub(super) async fn ensure_team_name_alias(
    tx: &mut Transaction<'_, Postgres>,
    team_id: Uuid,
    name: &str,
    language_code: Option<&str>,
    valid_from: Option<NaiveDate>,
    valid_to: Option<NaiveDate>,
    metadata: &Value,
) -> PersistenceResult<()> {
    let name = name.trim();
    if name.is_empty() {
        return Ok(());
    }
    let normalized_name = normalize_name(name);
    sqlx::query(
        r#"WITH updated AS (
            UPDATE football.team_names
            SET name = $3,
                language_code = COALESCE($5, language_code),
                valid_from = COALESCE($6, valid_from),
                valid_to = COALESCE($7, valid_to),
                metadata = metadata || $8
            WHERE team_id = $2 AND normalized_name = $4
            RETURNING id
        )
        INSERT INTO football.team_names (
            id, team_id, name, normalized_name, language_code, valid_from, valid_to, metadata
        )
        SELECT $1, $2, $3, $4, $5, $6, $7, $8
        WHERE NOT EXISTS (SELECT 1 FROM updated)"#,
    )
    .bind(Uuid::new_v4())
    .bind(team_id)
    .bind(name)
    .bind(normalized_name)
    .bind(language_code)
    .bind(valid_from)
    .bind(valid_to)
    .bind(metadata)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub(super) async fn preserve_current_team_canonical_alias(
    tx: &mut Transaction<'_, Postgres>,
    team_id: Uuid,
    metadata: &Value,
) -> PersistenceResult<()> {
    let current_name =
        sqlx::query_scalar::<_, String>("SELECT canonical_name FROM football.teams WHERE id=$1")
            .bind(team_id)
            .fetch_optional(&mut **tx)
            .await?;
    if let Some(current_name) = current_name {
        ensure_team_name_alias(tx, team_id, &current_name, None, None, None, metadata).await?;
    }
    Ok(())
}
