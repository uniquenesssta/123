use crate::{PersistenceError, PersistenceResult, PostgresStore};
use football_domain::LineupHistoryRemovalResult;
use serde_json::json;
use sqlx::Row;
use uuid::Uuid;

impl PostgresStore {
    pub async fn remove_lineup_history(
        &self,
        lineup_id: Uuid,
        reason: Option<&str>,
    ) -> PersistenceResult<LineupHistoryRemovalResult> {
        let reason = reason
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("用户从阵容历史中删除");
        let mut tx = self.pool.begin().await?;
        sqlx::query("SET TRANSACTION ISOLATION LEVEL READ COMMITTED")
            .execute(&mut *tx)
            .await?;
        // 先定位父比赛，再按创建/导入相同顺序锁父行；锁后重新读取版本及引用。
        let parent_match_id: Uuid =
            sqlx::query_scalar("SELECT match_id FROM football.lineups WHERE id=$1")
                .bind(lineup_id)
                .fetch_optional(&mut *tx)
                .await?
                .ok_or_else(|| PersistenceError::InvalidState("阵容版本不存在".to_string()))?;
        super::super::pair_transaction::lock_match_in_tx(&mut tx, parent_match_id).await?;
        let row = sqlx::query(
            r#"
            SELECT id, match_id, team_id, snapshot_type, lineup_type, status,
                   history_hidden_at
            FROM football.lineups
            WHERE id = $1
            FOR UPDATE
            "#,
        )
        .bind(lineup_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("阵容版本不存在".to_string()))?;

        if row
            .try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("history_hidden_at")?
            .is_some()
        {
            return Err(PersistenceError::InvalidState(
                "阵容版本已经从历史列表隐藏".to_string(),
            ));
        }

        let match_id: Uuid = row.try_get("match_id")?;
        let team_id: Uuid = row.try_get("team_id")?;
        let snapshot_type: String = row.try_get("snapshot_type")?;
        let lineup_type: String = row.try_get("lineup_type")?;
        let status: String = row.try_get("status")?;

        let referenced: bool = sqlx::query_scalar(
            r#"
            SELECT
                EXISTS (SELECT 1 FROM football.lineups WHERE supersedes_lineup_id = $1)
                OR EXISTS (SELECT 1 FROM feature.match_player_contributions WHERE lineup_id = $1)
                OR EXISTS (SELECT 1 FROM feature.snapshots WHERE input_payload::text LIKE '%' || $1::text || '%')
                OR EXISTS (SELECT 1 FROM model.runs WHERE input_payload::text LIKE '%' || $1::text || '%')
            "#,
        )
        .bind(lineup_id)
        .fetch_one(&mut *tx)
        .await?;

        let removal_mode = if referenced {
            sqlx::query(
                r#"
                UPDATE football.lineups
                SET history_hidden_at = now(),
                    history_hidden_reason = $2,
                    status = CASE WHEN status = 'active' THEN 'withdrawn' ELSE status END,
                    updated_at = now()
                WHERE id = $1
                "#,
            )
            .bind(lineup_id)
            .bind(reason)
            .execute(&mut *tx)
            .await?;
            "archived"
        } else {
            sqlx::query("DELETE FROM football.lineups WHERE id = $1")
                .bind(lineup_id)
                .execute(&mut *tx)
                .await?;
            "deleted"
        };

        let restored_lineup_id = if status == "active" {
            let candidate = sqlx::query_scalar::<_, Uuid>(
                r#"
                SELECT id
                FROM football.lineups
                WHERE match_id = $1
                  AND team_id = $2
                  AND snapshot_type = $3
                  AND lineup_type = $4
                  AND status = 'superseded'
                  AND history_hidden_at IS NULL
                ORDER BY captured_at DESC, created_at DESC, id DESC
                LIMIT 1
                FOR UPDATE
                "#,
            )
            .bind(match_id)
            .bind(team_id)
            .bind(&snapshot_type)
            .bind(&lineup_type)
            .fetch_optional(&mut *tx)
            .await?;
            if let Some(candidate_id) = candidate {
                sqlx::query(
                    "UPDATE football.lineups SET status='active', updated_at=now() WHERE id=$1",
                )
                .bind(candidate_id)
                .execute(&mut *tx)
                .await?;
                Some(candidate_id)
            } else {
                None
            }
        } else {
            None
        };

        crate::write_audit_event(
            &mut tx,
            "lineup_history_removed",
            "lineup",
            lineup_id.to_string(),
            json!({
                "removal_mode": removal_mode,
                "reason": reason,
                "restored_lineup_id": restored_lineup_id,
                "match_id": match_id,
                "team_id": team_id,
                "snapshot_type": snapshot_type,
                "lineup_type": lineup_type,
            }),
        )
        .await?;
        tx.commit().await?;
        Ok(LineupHistoryRemovalResult {
            lineup_id,
            removal_mode: removal_mode.to_string(),
            restored_lineup_id,
        })
    }
}
