use super::{PersistenceError, PersistenceResult};
use sha2::{Digest, Sha384};
use sqlx::{postgres::PgPool, Postgres, Row, Transaction};

const LEGACY_INTEGRATION_CONTRACT_SHA256: &str =
    "b7ff6e3cc13afc8c9d6d8cac1b6b4f566fc7b7fd9f171be305fb57725e3a8371";
const COMPATIBILITY_LOCK_KEY: &str = "football-platform-migration-compatibility";

const COMPATIBLE_MIGRATION_VERSIONS: [i64; 11] = [12, 13, 14, 15, 16, 17, 18, 25, 26, 27, 31];

pub async fn reconcile_known_legacy_migrations(pool: &PgPool) -> PersistenceResult<()> {
    let mut transaction = pool.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock(hashtext($1)::bigint)")
        .bind(COMPATIBILITY_LOCK_KEY)
        .execute(&mut *transaction)
        .await?;

    if !migration_ledger_exists(&mut transaction).await? {
        transaction.commit().await?;
        return Ok(());
    }

    let Some(stored_checksum) = applied_checksum(&mut transaction, 12).await? else {
        transaction.commit().await?;
        return Ok(());
    };
    let current_checksum = current_migration_checksum(12).expect("迁移 12 必须登记在兼容清单");
    if stored_checksum == current_checksum {
        transaction.commit().await?;
        return Ok(());
    }

    verify_known_legacy_lineage(&mut transaction).await?;
    ensure_public_engine_artifact_shape(&mut transaction).await?;

    for version in COMPATIBLE_MIGRATION_VERSIONS {
        reconcile_applied_checksum(&mut transaction, version).await?;
    }

    transaction.commit().await?;
    Ok(())
}

async fn migration_ledger_exists(
    transaction: &mut Transaction<'_, Postgres>,
) -> PersistenceResult<bool> {
    Ok(
        sqlx::query_scalar::<_, bool>("SELECT to_regclass('public._sqlx_migrations') IS NOT NULL")
            .fetch_one(&mut **transaction)
            .await?,
    )
}

async fn applied_checksum(
    transaction: &mut Transaction<'_, Postgres>,
    version: i64,
) -> PersistenceResult<Option<Vec<u8>>> {
    let row = sqlx::query("SELECT checksum, success FROM public._sqlx_migrations WHERE version=$1")
        .bind(version)
        .fetch_optional(&mut **transaction)
        .await?;
    let Some(row) = row else {
        return Ok(None);
    };
    let success: bool = row.try_get("success")?;
    if !success {
        return Err(PersistenceError::InvalidState(format!(
            "数据库迁移 {version} 的历史记录不是成功状态，拒绝自动修复迁移账本"
        )));
    }
    Ok(Some(row.try_get("checksum")?))
}

async fn verify_known_legacy_lineage(
    transaction: &mut Transaction<'_, Postgres>,
) -> PersistenceResult<()> {
    let contract_table_exists: bool =
        sqlx::query_scalar("SELECT to_regclass('platform.integration_contracts') IS NOT NULL")
            .fetch_one(&mut **transaction)
            .await?;
    if !contract_table_exists {
        return Err(unknown_migration_history_error());
    }

    let recognized: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM platform.integration_contracts
            WHERE contract_key = 'p4-software-integration'
              AND contract_version = '1.0.0'
              AND content_sha256 = $1
        )
        "#,
    )
    .bind(LEGACY_INTEGRATION_CONTRACT_SHA256)
    .fetch_one(&mut **transaction)
    .await?;
    if !recognized {
        return Err(unknown_migration_history_error());
    }
    Ok(())
}

fn unknown_migration_history_error() -> PersistenceError {
    PersistenceError::InvalidState(
        "检测到数据库迁移 12 的校验值与当前公开版本不同，但数据库不属于已登记的兼容历史；为保护数据，未修改迁移账本"
            .to_string(),
    )
}

async fn reconcile_applied_checksum(
    transaction: &mut Transaction<'_, Postgres>,
    version: i64,
) -> PersistenceResult<()> {
    let Some(stored_checksum) = applied_checksum(transaction, version).await? else {
        return Ok(());
    };
    let current_checksum = current_migration_checksum(version).ok_or_else(|| {
        PersistenceError::InvalidState(format!("迁移 {version} 未登记公开兼容校验值"))
    })?;
    if stored_checksum == current_checksum {
        return Ok(());
    }

    sqlx::query("UPDATE public._sqlx_migrations SET checksum=$2 WHERE version=$1 AND success=true")
        .bind(version)
        .bind(current_checksum)
        .execute(&mut **transaction)
        .await?;
    Ok(())
}

async fn ensure_public_engine_artifact_shape(
    transaction: &mut Transaction<'_, Postgres>,
) -> PersistenceResult<()> {
    let table_exists: bool =
        sqlx::query_scalar("SELECT to_regclass('model.engine_artifacts') IS NOT NULL")
            .fetch_one(&mut **transaction)
            .await?;
    if !table_exists {
        return Err(PersistenceError::InvalidState(
            "已识别兼容历史，但缺少 model.engine_artifacts；为保护数据，未修改迁移账本".to_string(),
        ));
    }

    let provider_fixture_exists = column_exists(
        transaction,
        "model",
        "engine_artifacts",
        "provider_fixture_sha256",
    )
    .await?;
    if provider_fixture_exists {
        return Ok(());
    }

    let legacy_fixture_exists = column_exists(
        transaction,
        "model",
        "engine_artifacts",
        "golden_master_sha256",
    )
    .await?;
    if !legacy_fixture_exists {
        return Err(PersistenceError::InvalidState(
            "已识别兼容历史，但模型制品账本结构无法安全映射；为保护数据，未修改迁移账本"
                .to_string(),
        ));
    }

    sqlx::query("ALTER TABLE model.engine_artifacts ADD COLUMN provider_fixture_sha256 text")
        .execute(&mut **transaction)
        .await?;
    sqlx::query(
        "UPDATE model.engine_artifacts SET provider_fixture_sha256 = golden_master_sha256 WHERE provider_fixture_sha256 IS NULL",
    )
    .execute(&mut **transaction)
    .await?;
    sqlx::query(
        "ALTER TABLE model.engine_artifacts ALTER COLUMN provider_fixture_sha256 SET NOT NULL",
    )
    .execute(&mut **transaction)
    .await?;

    let constraint_exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM pg_constraint
            WHERE conrelid = 'model.engine_artifacts'::regclass
              AND conname = 'engine_artifacts_provider_fixture_sha256_check'
        )
        "#,
    )
    .fetch_one(&mut **transaction)
    .await?;
    if !constraint_exists {
        sqlx::query(
            "ALTER TABLE model.engine_artifacts ADD CONSTRAINT engine_artifacts_provider_fixture_sha256_check CHECK (provider_fixture_sha256 ~ '^[0-9a-f]{64}$')",
        )
        .execute(&mut **transaction)
        .await?;
    }
    Ok(())
}

async fn column_exists(
    transaction: &mut Transaction<'_, Postgres>,
    schema: &str,
    table: &str,
    column: &str,
) -> PersistenceResult<bool> {
    Ok(sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM information_schema.columns
            WHERE table_schema=$1 AND table_name=$2 AND column_name=$3
        )
        "#,
    )
    .bind(schema)
    .bind(table)
    .bind(column)
    .fetch_one(&mut **transaction)
    .await?)
}

fn current_migration_checksum(version: i64) -> Option<Vec<u8>> {
    let sql = match version {
        12 => include_str!("../migrations/0012_p4_integration_contract.sql"),
        13 => include_str!("../migrations/0013_p4_engine_artifacts.sql"),
        14 => include_str!("../migrations/0014_p4_evidence_and_snapshots.sql"),
        15 => include_str!("../migrations/0015_p4_openai_research_gateway.sql"),
        16 => include_str!("../migrations/0016_p4_fact_pipeline.sql"),
        17 => include_str!("../migrations/0017_p4_horizon_orchestration.sql"),
        18 => include_str!("../migrations/0018_p4_single_match_workbench.sql"),
        25 => include_str!("../migrations/0025_parameter_lifecycle.sql"),
        26 => include_str!("../migrations/0026_postmatch_settlement.sql"),
        27 => include_str!("../migrations/0027_release_acceptance.sql"),
        31 => include_str!("../migrations/0031_p4_model_family_time_windows.sql"),
        _ => return None,
    };
    Some(Sha384::digest(sql.as_bytes()).to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compatibility_allowlist_is_exact_and_checksums_are_sha384() {
        assert_eq!(
            COMPATIBLE_MIGRATION_VERSIONS,
            [12, 13, 14, 15, 16, 17, 18, 25, 26, 27, 31]
        );
        for version in COMPATIBLE_MIGRATION_VERSIONS {
            assert_eq!(current_migration_checksum(version).unwrap().len(), 48);
        }
        assert!(current_migration_checksum(11).is_none());
        assert!(current_migration_checksum(32).is_none());
    }
}
