use crate::{PersistenceResult, PostgresStore};
use football_domain::P4ResearchRunWorkspace;
use sqlx::Row;
use uuid::Uuid;

pub(super) async fn read(
    store: &PostgresStore,
    research_run_id: Uuid,
) -> PersistenceResult<P4ResearchRunWorkspace> {
    let row = sqlx::query(
        r#"
                SELECT id, status, attempt_count, response_id, model_id,
                       error_category, error_message, created_at, started_at, finished_at
                FROM research.runs
                WHERE id = $1
                "#,
    )
    .bind(research_run_id)
    .fetch_one(&store.pool)
    .await?;
    Ok(P4ResearchRunWorkspace {
        id: row.try_get("id")?,
        status: row.try_get("status")?,
        attempt_count: row.try_get("attempt_count")?,
        response_id: row.try_get("response_id")?,
        model_id: row.try_get("model_id")?,
        error_category: row.try_get("error_category")?,
        error_message: row.try_get("error_message")?,
        created_at: row.try_get("created_at")?,
        started_at: row.try_get("started_at")?,
        finished_at: row.try_get("finished_at")?,
    })
}
