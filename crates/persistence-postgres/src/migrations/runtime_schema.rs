use crate::PersistenceResult;
use sqlx::postgres::PgPool;

pub(crate) async fn ensure_runtime_schema_compatibility(pool: &PgPool) -> PersistenceResult<()> {
    let mut transaction = pool.begin().await?;
    sqlx::query(
        "SELECT pg_advisory_xact_lock(hashtext('football-platform-schema-compatibility')::bigint)",
    )
    .execute(&mut *transaction)
    .await?;

    let observation_table_exists: bool =
        sqlx::query_scalar("SELECT to_regclass('feature.player_ability_observations') IS NOT NULL")
            .fetch_one(&mut *transaction)
            .await?;
    if !observation_table_exists {
        transaction.commit().await?;
        return Ok(());
    }

    let cutoff_column_ready: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM information_schema.columns
            WHERE table_schema = 'feature'
              AND table_name = 'player_ability_observations'
              AND column_name = 'created_at'
              AND is_nullable = 'NO'
              AND column_default IS NOT NULL
        )
        "#,
    )
    .fetch_one(&mut *transaction)
    .await?;

    if !cutoff_column_ready {
        sqlx::query(
            "ALTER TABLE feature.player_ability_observations ADD COLUMN IF NOT EXISTS created_at timestamptz",
        )
        .execute(&mut *transaction)
        .await?;
        sqlx::query(
            "UPDATE feature.player_ability_observations SET created_at = observed_at WHERE created_at IS NULL",
        )
        .execute(&mut *transaction)
        .await?;
        sqlx::query(
            "ALTER TABLE feature.player_ability_observations ALTER COLUMN created_at SET DEFAULT now()",
        )
        .execute(&mut *transaction)
        .await?;
        sqlx::query(
            "ALTER TABLE feature.player_ability_observations ALTER COLUMN created_at SET NOT NULL",
        )
        .execute(&mut *transaction)
        .await?;
    }

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS ability_observations_player_cutoff_idx
            ON feature.player_ability_observations
               (player_id, created_at DESC, dimension_code, effective_from DESC)
        "#,
    )
    .execute(&mut *transaction)
    .await?;

    transaction.commit().await?;
    Ok(())
}
