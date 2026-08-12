use crate::{PersistenceResult, PostgresStore};

const APPLICATION_SCHEMAS: [&str; 10] = [
    "ai_workspace",
    "analytics",
    "audit",
    "catalog",
    "feature",
    "football",
    "model",
    "platform",
    "research",
    "review",
];

impl PostgresStore {
    pub async fn reset_to_pristine(&self) -> PersistenceResult<()> {
        let mut transaction = self.pool.begin().await?;
        sqlx::query(
            "SELECT pg_advisory_xact_lock(hashtext('football-platform-destructive-reset')::bigint)",
        )
        .execute(&mut *transaction)
        .await?;

        for schema in APPLICATION_SCHEMAS {
            let statement = format!("DROP SCHEMA IF EXISTS {schema} CASCADE");
            sqlx::query(&statement).execute(&mut *transaction).await?;
        }
        sqlx::query("DROP TABLE IF EXISTS public._sqlx_migrations")
            .execute(&mut *transaction)
            .await?;
        transaction.commit().await?;

        self.migrate().await
    }
}
