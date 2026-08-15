use football_domain::CompetitionKind;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::PersistenceResult;

pub(super) struct NewTypeDefaultBinding<'a> {
    pub id: Uuid,
    pub binding_name: &'a str,
    pub competition_kind: CompetitionKind,
    pub model_version_id: Uuid,
    pub parameter_set_id: Uuid,
    pub rule_package_id: Uuid,
    pub priority: i32,
}

pub(super) async fn insert_binding(
    tx: &mut Transaction<'_, Postgres>,
    binding: NewTypeDefaultBinding<'_>,
) -> PersistenceResult<()> {
    sqlx::query(
        r#"
        INSERT INTO model.competition_bindings (
            id, binding_name, competition_kind,
            model_version_id, parameter_set_id, rule_package_id,
            priority, is_active
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, true)
        "#,
    )
    .bind(binding.id)
    .bind(binding.binding_name)
    .bind(binding.competition_kind.as_str())
    .bind(binding.model_version_id)
    .bind(binding.parameter_set_id)
    .bind(binding.rule_package_id)
    .bind(binding.priority)
    .execute(&mut **tx)
    .await?;

    Ok(())
}
