use chrono::{DateTime, Utc};
use football_domain::ResolvedCompetitionContext;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::PersistenceResult;

pub(super) struct NewBinding<'a> {
    pub id: Uuid,
    pub binding_name: &'a str,
    pub resolved: &'a ResolvedCompetitionContext,
    pub model_version_id: Uuid,
    pub parameter_set_id: Uuid,
    pub rule_package_id: Uuid,
    pub priority: i32,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_to: Option<DateTime<Utc>>,
}

pub(super) async fn insert_binding(
    tx: &mut Transaction<'_, Postgres>,
    binding: NewBinding<'_>,
) -> PersistenceResult<()> {
    sqlx::query(
        r#"
        INSERT INTO model.competition_bindings (
            id, binding_name, competition_id, season_id, stage_id,
            competition_kind, model_version_id, parameter_set_id,
            rule_package_id, priority, is_active, valid_from, valid_to
        ) VALUES (
            $1, $2, $3, $4, $5,
            $6, $7, $8,
            $9, $10, true, $11, $12
        )
        "#,
    )
    .bind(binding.id)
    .bind(binding.binding_name)
    .bind(binding.resolved.competition_id)
    .bind(binding.resolved.season_id)
    .bind(binding.resolved.stage_id)
    .bind(binding.resolved.competition_kind.as_str())
    .bind(binding.model_version_id)
    .bind(binding.parameter_set_id)
    .bind(binding.rule_package_id)
    .bind(binding.priority)
    .bind(binding.valid_from)
    .bind(binding.valid_to)
    .execute(&mut **tx)
    .await?;

    Ok(())
}
