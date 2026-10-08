use crate::{PersistenceError, PersistenceResult};
use sqlx::{Postgres, Transaction};

pub(crate) async fn advisory_lock(
    tx: &mut Transaction<'_, Postgres>,
    key: &str,
) -> PersistenceResult<()> {
    sqlx::query("SELECT pg_advisory_xact_lock(hashtext($1)::bigint)")
        .bind(key)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

pub(crate) fn validate_idempotency_key(value: &str) -> PersistenceResult<()> {
    if value.trim().is_empty() || value.len() > 240 {
        return Err(PersistenceError::InvalidState(
            "幂等键不能为空且长度不得超过240".to_string(),
        ));
    }
    Ok(())
}

pub(crate) fn ensure_idempotent_fingerprint(
    entity: &str,
    idempotency_key: &str,
    existing: &str,
    expected: &str,
) -> PersistenceResult<()> {
    if existing != expected {
        return Err(PersistenceError::InvalidState(format!(
            "{entity}幂等键{idempotency_key}已绑定不同载荷"
        )));
    }
    Ok(())
}
