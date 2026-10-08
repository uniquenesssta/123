use crate::{write_audit_event, PersistenceError, PersistenceResult, PostgresStore};
use serde_json::json;
use uuid::Uuid;

impl PostgresStore {
    pub async fn hide_run_from_history(
        &self,
        run_id: Uuid,
        reason: Option<&str>,
    ) -> PersistenceResult<()> {
        let reason = reason
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("用户从推演历史列表中删除");
        let mut tx = self.pool.begin().await?;
        let match_key = sqlx::query_scalar::<_, String>(
            r#"
            UPDATE model.runs
            SET history_hidden_at = COALESCE(history_hidden_at, now()),
                history_hidden_reason = $2
            WHERE id = $1 AND status = 'succeeded'
            RETURNING match_key
            "#,
        )
        .bind(run_id)
        .bind(reason)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| {
            PersistenceError::InvalidState("推演记录不存在或尚未成功完成".to_string())
        })?;
        write_audit_event(
            &mut tx,
            "model_run_history_hidden",
            "model_run",
            Some(run_id.to_string()),
            json!({"match_key": match_key, "reason": reason}),
        )
        .await?;
        tx.commit().await?;
        Ok(())
    }
}
