use crate::{sha256_json, PersistenceError, PersistenceResult};
use football_domain::P4FreezeTaskState;
use serde_json::{json, Value};
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

pub(super) async fn append_task_event(
    tx: &mut Transaction<'_, Postgres>,
    task_id: Uuid,
    from_state: Option<P4FreezeTaskState>,
    to_state: P4FreezeTaskState,
    reason: &str,
    payload: Value,
) -> PersistenceResult<()> {
    let idempotency_key = format!(
        "{}:{}:{}",
        from_state.map(P4FreezeTaskState::as_str).unwrap_or("NONE"),
        to_state.as_str(),
        sha256_json(&json!({"reason": reason, "payload": &payload}))?
    );
    let event_fingerprint = sha256_json(&json!({
        "task_id": task_id,
        "from_state": from_state.map(P4FreezeTaskState::as_str),
        "to_state": to_state.as_str(),
        "reason": reason,
        "payload": &payload,
    }))?;
    let inserted: Option<Uuid> = sqlx::query_scalar(
        r#"
        INSERT INTO platform.p4_freeze_task_events (
            id, task_id, from_state, to_state, reason, payload,
            idempotency_key, event_fingerprint
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (task_id, idempotency_key) DO NOTHING
        RETURNING id
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(task_id)
    .bind(from_state.map(P4FreezeTaskState::as_str))
    .bind(to_state.as_str())
    .bind(reason)
    .bind(&payload)
    .bind(&idempotency_key)
    .bind(&event_fingerprint)
    .fetch_optional(&mut **tx)
    .await?;
    if inserted.is_none() {
        let existing: String = sqlx::query_scalar(
            r#"
            SELECT event_fingerprint
            FROM platform.p4_freeze_task_events
            WHERE task_id = $1 AND idempotency_key = $2
            "#,
        )
        .bind(task_id)
        .bind(&idempotency_key)
        .fetch_one(&mut **tx)
        .await?;
        if existing != event_fingerprint {
            return Err(PersistenceError::InvalidState(
                "P4冻结任务事件幂等冲突".to_string(),
            ));
        }
    }
    Ok(())
}
