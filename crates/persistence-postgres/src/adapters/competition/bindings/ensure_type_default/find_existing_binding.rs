use football_domain::CompetitionKind;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::PersistenceResult;

pub(super) async fn find_existing_binding(
    tx: &mut Transaction<'_, Postgres>,
    package_id: Uuid,
    competition_kind: CompetitionKind,
) -> PersistenceResult<Option<Uuid>> {
    let id = sqlx::query_scalar(
        r#"
        SELECT id
        FROM model.competition_bindings
        WHERE rule_package_id = $1
          AND competition_id IS NULL
          AND season_id IS NULL
          AND stage_id IS NULL
          AND competition_kind = $2
          AND is_active = true
        ORDER BY priority DESC, created_at DESC
        LIMIT 1
        "#,
    )
    .bind(package_id)
    .bind(competition_kind.as_str())
    .fetch_optional(&mut **tx)
    .await?;

    Ok(id)
}
