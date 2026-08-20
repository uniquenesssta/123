use crate::{PersistenceError, PersistenceResult, PostgresStore};
use serde_json::json;
use uuid::Uuid;

impl PostgresStore {
    pub async fn delete_match(&self, match_id: Uuid) -> PersistenceResult<()> {
        let mut tx = self.pool.begin().await?;
        let match_key: String = sqlx::query_scalar(
            "SELECT external_key FROM football.matches WHERE id = $1 FOR UPDATE",
        )
        .bind(match_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("比赛不存在".to_string()))?;
        let protected_count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT
                (SELECT count(*) FROM research.runs WHERE match_id = $1)
              + (SELECT count(*) FROM platform.p4_freeze_tasks WHERE match_id = $1)
              + (SELECT count(*) FROM review.postmatch_settlements WHERE match_id = $1)
            "#,
        )
        .bind(match_id)
        .fetch_one(&mut *tx)
        .await?;
        if protected_count > 0 {
            return Err(PersistenceError::InvalidState(
                "该比赛已进入P4研究、冻结或正式赛后结算，必须保留不可变审计血缘，不能永久删除"
                    .to_string(),
            ));
        }

        sqlx::query("DELETE FROM football.external_entity_ids WHERE entity_type = 'match' AND entity_id = $1")
            .bind(match_id).execute(&mut *tx).await?;
        sqlx::query("UPDATE model.runs SET match_id = NULL WHERE match_id = $1")
            .bind(match_id)
            .execute(&mut *tx)
            .await?;
        sqlx::query("UPDATE feature.snapshots SET match_id = NULL WHERE match_id = $1")
            .bind(match_id)
            .execute(&mut *tx)
            .await?;
        sqlx::query("UPDATE ai_workspace.sessions SET match_id = NULL WHERE match_id = $1")
            .bind(match_id)
            .execute(&mut *tx)
            .await?;
        sqlx::query(
            "DELETE FROM review.ability_update_candidates WHERE match_review_id IN (SELECT id FROM review.match_reviews WHERE match_id = $1)",
        )
        .bind(match_id)
        .execute(&mut *tx)
        .await?;
        sqlx::query("DELETE FROM review.match_reviews WHERE match_id = $1")
            .bind(match_id)
            .execute(&mut *tx)
            .await?;
        crate::write_audit_event(
            &mut tx,
            "match_deleted",
            "match",
            match_id.to_string(),
            json!({"external_key": match_key}),
        )
        .await?;
        sqlx::query("DELETE FROM football.matches WHERE id = $1")
            .bind(match_id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(())
    }
}
