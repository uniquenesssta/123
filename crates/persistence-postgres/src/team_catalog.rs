use crate::{PersistenceError, PersistenceResult, PostgresStore};
use football_domain::{BulkDeleteBlockedItem, BulkDeleteResult};
use serde_json::json;
use uuid::Uuid;

impl PostgresStore {
    pub async fn bulk_delete_players(
        &self,
        player_ids: &[Uuid],
    ) -> PersistenceResult<BulkDeleteResult> {
        let mut deleted_ids = Vec::new();
        let mut blocked = Vec::new();
        for player_id in unique_ids(player_ids) {
            let label = sqlx::query_scalar::<_, String>(
                "SELECT canonical_name FROM football.players WHERE id = $1",
            )
            .bind(player_id)
            .fetch_optional(&self.pool)
            .await?
            .unwrap_or_else(|| player_id.to_string());
            match self.delete_player(player_id).await {
                Ok(()) => deleted_ids.push(player_id),
                Err(error) => blocked.push(BulkDeleteBlockedItem {
                    id: player_id,
                    label,
                    reason: error.to_string(),
                }),
            }
        }
        Ok(BulkDeleteResult {
            requested_count: unique_ids(player_ids).len() as u64,
            deleted_ids,
            blocked,
        })
    }

    pub async fn bulk_delete_teams(
        &self,
        team_ids: &[Uuid],
    ) -> PersistenceResult<BulkDeleteResult> {
        let ids = unique_ids(team_ids);
        let mut deleted_ids = Vec::new();
        let mut blocked = Vec::new();
        for team_id in &ids {
            match self.delete_team(*team_id).await {
                Ok(()) => deleted_ids.push(*team_id),
                Err(error) => {
                    let label = sqlx::query_scalar::<_, String>(
                        "SELECT canonical_name FROM football.teams WHERE id = $1",
                    )
                    .bind(team_id)
                    .fetch_optional(&self.pool)
                    .await?
                    .unwrap_or_else(|| team_id.to_string());
                    blocked.push(BulkDeleteBlockedItem {
                        id: *team_id,
                        label,
                        reason: error.to_string(),
                    });
                }
            }
        }
        Ok(BulkDeleteResult {
            requested_count: ids.len() as u64,
            deleted_ids,
            blocked,
        })
    }

    async fn delete_team(&self, team_id: Uuid) -> PersistenceResult<()> {
        let check = self.check_entity_deletion("team", team_id).await?;
        if !check.can_permanently_delete {
            return Err(PersistenceError::InvalidState(check.reason));
        }
        let mut tx = self.pool.begin().await?;
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
        crate::write_audit_event(
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
}

fn unique_ids(ids: &[Uuid]) -> Vec<Uuid> {
    let mut output = Vec::new();
    for id in ids {
        if !output.contains(id) {
            output.push(*id);
        }
    }
    output
}
