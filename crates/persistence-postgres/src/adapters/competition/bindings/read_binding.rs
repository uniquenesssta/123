use football_domain::CompetitionBindingSummary;
use uuid::Uuid;

use crate::{PersistenceResult, PostgresStore};

use super::{map_binding_row, BindingRow};

pub(in crate::adapters::competition::bindings) async fn read_binding(
    store: &PostgresStore,
    id: Uuid,
) -> PersistenceResult<CompetitionBindingSummary> {
    let row = sqlx::query_as::<_, BindingRow>(
        r#"
        SELECT
            b.id, b.binding_name, b.competition_id, c.name AS competition_name,
            b.season_id, b.stage_id, b.competition_kind,
            b.rule_package_id, rp.display_name AS rule_package_name,
            d.model_key, b.priority, b.is_active, b.created_at
        FROM model.competition_bindings b
        JOIN model.rule_packages rp ON rp.id = b.rule_package_id
        JOIN model.versions v ON v.id = b.model_version_id
        JOIN model.definitions d ON d.id = v.model_id
        LEFT JOIN football.competitions c ON c.id = b.competition_id
        WHERE b.id = $1
        "#,
    )
    .bind(id)
    .fetch_one(&store.pool)
    .await?;

    map_binding_row(row)
}
