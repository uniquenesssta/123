use super::preflight::{
    check_from_references,
    references::{player_reference_counts, team_reference_counts},
};
use crate::{write_audit_event, PersistenceError, PersistenceResult};
use serde_json::json;
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

pub(crate) async fn write_player_delete(pool: &PgPool, player_id: Uuid) -> PersistenceResult<()> {
    let mut tx = pool.begin().await?;
    // A fresh post-lock snapshot must see references committed while the lock waited.
    sqlx::query("SET TRANSACTION ISOLATION LEVEL READ COMMITTED")
        .execute(&mut *tx)
        .await?;
    let player_name: String =
        sqlx::query_scalar("SELECT canonical_name FROM football.players WHERE id = $1 FOR UPDATE")
            .bind(player_id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(|| PersistenceError::InvalidState("球员不存在".to_string()))?;
    ensure_no_references(&mut tx, "player", player_id, &player_name).await?;

    sqlx::query(
        "DELETE FROM football.external_entity_ids WHERE entity_type = 'player' AND entity_id = $1",
    )
    .bind(player_id)
    .execute(&mut *tx)
    .await?;
    write_audit_event(
        &mut tx,
        "player_deleted",
        "player",
        player_id.to_string(),
        json!({"canonical_name": player_name, "reference_check": "passed"}),
    )
    .await?;
    sqlx::query("DELETE FROM football.players WHERE id = $1")
        .bind(player_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}

pub(crate) async fn write_team_delete(pool: &PgPool, team_id: Uuid) -> PersistenceResult<()> {
    let mut tx = pool.begin().await?;
    // A fresh post-lock snapshot must see references committed while the lock waited.
    sqlx::query("SET TRANSACTION ISOLATION LEVEL READ COMMITTED")
        .execute(&mut *tx)
        .await?;
    let team_name = sqlx::query_scalar::<_, String>(
        "SELECT canonical_name FROM football.teams WHERE id = $1 FOR UPDATE",
    )
    .bind(team_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| PersistenceError::InvalidState("球队不存在".to_string()))?;
    ensure_no_references(&mut tx, "team", team_id, &team_name).await?;
    sqlx::query(
        "DELETE FROM football.external_entity_ids WHERE entity_type='team' AND entity_id=$1",
    )
    .bind(team_id)
    .execute(&mut *tx)
    .await?;
    sqlx::query("DELETE FROM football.teams WHERE id=$1")
        .bind(team_id)
        .execute(&mut *tx)
        .await?;
    write_audit_event(
        &mut tx,
        "team_deleted",
        "team",
        team_id.to_string(),
        json!({"canonical_name": team_name}),
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

async fn ensure_no_references(
    connection: &mut PgConnection,
    entity_type: &str,
    entity_id: Uuid,
    label: &str,
) -> PersistenceResult<()> {
    let references = match entity_type {
        "player" => player_reference_counts(connection, entity_id).await?,
        "team" => team_reference_counts(connection, entity_id).await?,
        _ => unreachable!(),
    };
    let check = check_from_references(entity_type, entity_id, label.to_string(), references);
    if !check.can_permanently_delete {
        return Err(PersistenceError::InvalidState(check.reason));
    }
    Ok(())
}
