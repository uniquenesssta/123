use football_domain::CompetitionBindingSummary;

use crate::{PersistenceResult, PostgresStore};

use super::{map_binding_row, BindingRow};

impl PostgresStore {
    pub async fn list_competition_bindings(
        &self,
    ) -> PersistenceResult<Vec<CompetitionBindingSummary>> {
        let rows = sqlx::query_as::<_, BindingRow>(
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
            WHERE b.is_active = true
              AND (b.valid_from IS NULL OR b.valid_from <= now())
              AND (b.valid_to IS NULL OR b.valid_to >= now())
            ORDER BY b.priority DESC, b.created_at DESC, b.id DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(map_binding_row).collect()
    }
}
