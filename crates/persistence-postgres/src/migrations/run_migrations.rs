use super::{
    reconcile_known_migrations::reconcile_known_legacy_migrations,
    runtime_schema::ensure_runtime_schema_compatibility,
};
use crate::{PersistenceResult, PostgresStore};

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

impl PostgresStore {
    pub async fn migrate(&self) -> PersistenceResult<()> {
        reconcile_known_legacy_migrations(&self.pool).await?;
        MIGRATOR.run(&self.pool).await?;
        ensure_runtime_schema_compatibility(&self.pool).await?;
        sqlx::query_scalar::<_, i64>("SELECT feature.refresh_player_ability_projections()")
            .fetch_one(&self.pool)
            .await?;
        Ok(())
    }
}
