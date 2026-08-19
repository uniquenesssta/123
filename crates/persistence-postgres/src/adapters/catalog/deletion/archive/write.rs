use crate::{write_audit_event, PersistenceError, PersistenceResult};
use serde_json::json;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

pub(crate) async fn archive_entity(
    pool: &sqlx::PgPool,
    entity_type: &str,
    id: Uuid,
) -> PersistenceResult<bool> {
    let mut tx = pool.begin().await?;
    let changed = match entity_type {
        "team" => sqlx::query("UPDATE football.teams SET is_active=false, updated_at=now() WHERE id=$1 AND is_active")
            .bind(id).execute(&mut *tx).await?.rows_affected(),
        "player" => sqlx::query("UPDATE football.players SET status='inactive', updated_at=now() WHERE id=$1 AND status NOT IN ('inactive','retired')")
            .bind(id).execute(&mut *tx).await?.rows_affected(),
        "coach" => sqlx::query("UPDATE football.coaches SET status='inactive', updated_at=now() WHERE id=$1 AND status NOT IN ('inactive','retired')")
            .bind(id).execute(&mut *tx).await?.rows_affected(),
        _ => unreachable!(),
    };
    if changed > 0 {
        write_audit_event(
            &mut tx,
            &format!("{entity_type}_archived"),
            entity_type,
            Some(id.to_string()),
            json!({"source":"manual_bulk_archive"}),
        )
        .await?;
    } else if entity_label_in_tx(&mut tx, entity_type, id)
        .await?
        .is_none()
    {
        return Err(PersistenceError::InvalidState("实体不存在".to_string()));
    }
    tx.commit().await?;
    Ok(changed > 0)
}

async fn entity_label_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    entity_type: &str,
    id: Uuid,
) -> PersistenceResult<Option<String>> {
    let query = match entity_type {
        "team" => "SELECT canonical_name FROM football.teams WHERE id=$1",
        "player" => "SELECT canonical_name FROM football.players WHERE id=$1",
        "coach" => "SELECT canonical_name FROM football.coaches WHERE id=$1",
        _ => unreachable!(),
    };
    Ok(sqlx::query_scalar(query)
        .bind(id)
        .fetch_optional(&mut **tx)
        .await?)
}
