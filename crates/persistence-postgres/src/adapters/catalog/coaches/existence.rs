use crate::{PersistenceError, PersistenceResult};
use sqlx::{Postgres, Transaction};
use uuid::Uuid;
pub(crate) async fn ensure_team_exists(
    tx: &mut Transaction<'_, Postgres>,
    team_id: Uuid,
) -> PersistenceResult<()> {
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM football.teams WHERE id=$1)")
            .bind(team_id)
            .fetch_one(&mut **tx)
            .await?;
    if !exists {
        return Err(PersistenceError::InvalidState("球队不存在".to_string()));
    }
    Ok(())
}

pub(crate) async fn ensure_coach_exists(
    tx: &mut Transaction<'_, Postgres>,
    coach_id: Uuid,
) -> PersistenceResult<()> {
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM football.coaches WHERE id=$1)")
            .bind(coach_id)
            .fetch_one(&mut **tx)
            .await?;
    if !exists {
        return Err(PersistenceError::InvalidState("教练不存在".to_string()));
    }
    Ok(())
}
