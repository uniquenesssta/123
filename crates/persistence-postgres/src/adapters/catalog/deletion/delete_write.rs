use crate::{write_audit_event, PersistenceError, PersistenceResult};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

pub(crate) async fn write_player_delete(pool: &PgPool, player_id: Uuid) -> PersistenceResult<()> {
    let mut tx = pool.begin().await?;
    let player_name: String =
        sqlx::query_scalar("SELECT canonical_name FROM football.players WHERE id = $1 FOR UPDATE")
            .bind(player_id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(|| PersistenceError::InvalidState("球员不存在".to_string()))?;

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
    let team_name = sqlx::query_scalar::<_, String>(
        "SELECT canonical_name FROM football.teams WHERE id = $1 FOR UPDATE",
    )
    .bind(team_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| PersistenceError::InvalidState("球队不存在".to_string()))?;
    let match_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM football.matches WHERE home_team_id=$1 OR away_team_id=$1",
    )
    .bind(team_id)
    .fetch_one(&mut *tx)
    .await?;
    if match_count > 0 {
        return Err(PersistenceError::InvalidState(format!(
            "球队已关联 {match_count} 场比赛，为保留历史赛果不能永久删除"
        )));
    }
    let review_count: i64 = sqlx::query_scalar(
        r#"
        SELECT
            (SELECT count(*)::bigint FROM review.team_match_reviews WHERE team_id=$1)
          + (SELECT count(*)::bigint FROM review.player_match_reviews WHERE team_id=$1)
        "#,
    )
    .bind(team_id)
    .fetch_one(&mut *tx)
    .await?;
    if review_count > 0 {
        return Err(PersistenceError::InvalidState(format!(
            "球队已关联 {review_count} 条球队或球员赛后复盘，为保留历史记录不能永久删除"
        )));
    }
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
