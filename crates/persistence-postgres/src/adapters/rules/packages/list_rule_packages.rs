use super::{map_rule_package_row, RulePackageRow};
use crate::{PersistenceResult, PostgresStore};
use football_domain::RulePackageSummary;

impl PostgresStore {
    pub async fn list_rule_packages(&self) -> PersistenceResult<Vec<RulePackageSummary>> {
        let rows = sqlx::query_as::<_, RulePackageRow>(
            r#"
            SELECT
                rp.id, rp.format_version, rp.package_key, rp.version, rp.display_name,
                rp.competition_kind, d.model_key, v.version AS model_version,
                p.parameter_version, rp.priority, rp.content_sha256,
                rp.status, rp.created_at
            FROM model.rule_packages rp
            JOIN model.versions v ON v.id = rp.model_version_id
            JOIN model.definitions d ON d.id = v.model_id
            JOIN model.parameter_sets p ON p.id = rp.parameter_set_id
            ORDER BY rp.created_at DESC, rp.package_key, rp.version
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(map_rule_package_row).collect()
    }
}
