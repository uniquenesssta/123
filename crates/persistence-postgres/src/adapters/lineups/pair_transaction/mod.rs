mod validation;
mod write;

use crate::{PersistenceError, PersistenceResult, PostgresStore};
use football_domain::{LineupDraft, LineupPairDraft, LineupPairRecord, LineupRecord};
use serde_json::json;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;
use validation::validate_lineup_draft;
use write::insert_lineup_in_tx;

impl PostgresStore {
    pub async fn create_lineup(&self, draft: &LineupDraft) -> PersistenceResult<LineupRecord> {
        let validated = validate_lineup_draft(draft)?;
        let mut tx = self.pool.begin().await?;
        sqlx::query("SET TRANSACTION ISOLATION LEVEL READ COMMITTED")
            .execute(&mut *tx)
            .await?;
        lock_match_in_tx(&mut tx, draft.match_id)
            .await?
            .ok_or_else(|| {
                PersistenceError::InvalidState("阵容球队不是该场比赛的参赛队".to_string())
            })?;
        let lineup_id = insert_lineup_in_tx(&mut tx, draft, &validated).await?;
        tx.commit().await?;
        self.read_lineup(lineup_id).await
    }

    pub async fn create_lineup_pair(
        &self,
        draft: &LineupPairDraft,
    ) -> PersistenceResult<LineupPairRecord> {
        if draft.home.match_id != draft.away.match_id {
            return Err(PersistenceError::InvalidState(
                "双方阵容必须属于同一场比赛".to_string(),
            ));
        }
        if draft.home.team_id == draft.away.team_id {
            return Err(PersistenceError::InvalidState(
                "双方阵容不能使用同一支球队".to_string(),
            ));
        }
        if draft.home.snapshot_type != draft.away.snapshot_type {
            return Err(PersistenceError::InvalidState(
                "双方阵容必须使用同一数据窗口".to_string(),
            ));
        }
        if draft.home.lineup_type != draft.away.lineup_type {
            return Err(PersistenceError::InvalidState(
                "双方阵容必须使用同一阵容类型".to_string(),
            ));
        }
        let home_validated = validate_lineup_draft(&draft.home)?;
        let away_validated = validate_lineup_draft(&draft.away)?;
        let mut tx = self.pool.begin().await?;
        sqlx::query("SET TRANSACTION ISOLATION LEVEL READ COMMITTED")
            .execute(&mut *tx)
            .await?;
        let (home_team_id, away_team_id) = lock_match_in_tx(&mut tx, draft.home.match_id)
            .await?
            .ok_or_else(|| PersistenceError::InvalidState("比赛不存在".to_string()))?;
        if draft.home.team_id != home_team_id || draft.away.team_id != away_team_id {
            return Err(PersistenceError::InvalidState(
                "双方阵容必须分别对应比赛主队和客队".to_string(),
            ));
        }
        let home_id = insert_lineup_in_tx(&mut tx, &draft.home, &home_validated).await?;
        let away_id = insert_lineup_in_tx(&mut tx, &draft.away, &away_validated).await?;
        crate::write_audit_event(
            &mut tx,
            "lineup_pair_created",
            "match",
            draft.home.match_id.to_string(),
            json!({
                "home_lineup_id": home_id,
                "away_lineup_id": away_id,
                "snapshot_type": home_validated.snapshot_type,
                "lineup_type": draft.home.lineup_type.as_str(),
            }),
        )
        .await?;
        tx.commit().await?;
        Ok(LineupPairRecord {
            home: self.read_lineup(home_id).await?,
            away: self.read_lineup(away_id).await?,
        })
    }
}

/// 阵容版本写入的共同串行边界，调用方持有并提交原事务。
/// Workbook 复用此锁，不在内部开启或提交另一事务。
pub(crate) async fn lock_match_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    match_id: Uuid,
) -> PersistenceResult<Option<(Uuid, Uuid)>> {
    Ok(sqlx::query_as(
        "SELECT home_team_id, away_team_id FROM football.matches WHERE id=$1 FOR UPDATE",
    )
    .bind(match_id)
    .fetch_optional(&mut **tx)
    .await?)
}
