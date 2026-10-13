use crate::{PersistenceError, PersistenceResult, PostgresStore};
use sqlx::Row;
use uuid::Uuid;

#[derive(Debug)]
pub(crate) struct PostmatchPartition {
    pub(crate) competition_id: Uuid,
    pub(crate) competition_profile_id: Uuid,
    pub(crate) model_version_id: Uuid,
    pub(crate) parameter_set_id: Uuid,
    pub(crate) horizon: String,
    pub(crate) partition_key: String,
}

impl PostgresStore {
    pub(super) async fn active_postmatch_partition(
        &self,
        competition_id: Uuid,
        horizon: &str,
    ) -> PersistenceResult<PostmatchPartition> {
        let row = sqlx::query(
            r#"
            SELECT competition.id AS competition_id, package.competition_profile_id,
                   binding.model_version_id, binding.parameter_set_id
            FROM model.competition_bindings binding
            JOIN football.competitions competition ON competition.id=binding.competition_id
            JOIN model.rule_packages package ON package.id=binding.rule_package_id
            WHERE binding.competition_id=$1 AND binding.is_active=true
              AND package.status='active'
            ORDER BY binding.priority DESC, binding.id DESC
            LIMIT 1
            "#,
        )
        .bind(competition_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("未找到该赛事的活动模型绑定".to_string()))?;
        let model_version_id: Uuid = row.try_get("model_version_id")?;
        let competition_profile_id: Uuid = row.try_get("competition_profile_id")?;
        let parameter_set_id: Uuid = row.try_get("parameter_set_id")?;
        Ok(PostmatchPartition {
            competition_id: row.try_get("competition_id")?,
            competition_profile_id,
            model_version_id,
            parameter_set_id,
            horizon: horizon.to_string(),
            partition_key: format!(
                "{model_version_id}:{competition_profile_id}:{parameter_set_id}:{horizon}"
            ),
        })
    }
}
