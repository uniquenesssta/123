use crate::PersistenceResult;
use sqlx::{FromRow, Postgres, Transaction};
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub(super) struct ExistingRulePackage {
    pub(super) id: Uuid,
    pub(super) content_sha256: String,
    pub(super) competition_profile_id: Option<Uuid>,
}

pub(super) async fn find_existing_package(
    tx: &mut Transaction<'_, Postgres>,
    package_key: &str,
    version: &str,
) -> PersistenceResult<ExistingRulePackage> {
    Ok(sqlx::query_as::<_, ExistingRulePackage>(
        r#"
        SELECT id, content_sha256, competition_profile_id
        FROM model.rule_packages
        WHERE package_key = $1 AND version = $2
        "#,
    )
    .bind(package_key)
    .bind(version)
    .fetch_one(&mut **tx)
    .await?)
}
