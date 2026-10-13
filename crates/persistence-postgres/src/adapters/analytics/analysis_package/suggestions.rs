use super::mapping::ai_suggestion_from_row;
use crate::{PersistenceError, PersistenceResult, PostgresStore};
use football_domain::AiAnalysisSuggestionRecord;
use uuid::Uuid;

impl PostgresStore {
    pub async fn list_ai_suggestions(
        &self,
        status: Option<&str>,
        limit: u32,
    ) -> PersistenceResult<Vec<AiAnalysisSuggestionRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT id,response_id,suggestion_type,title,summary,severity,scope,payload,evidence,status,
                   created_at,decided_at,decision_note,linked_candidate_id
            FROM analytics.ai_suggestions
            WHERE ($1::text IS NULL OR status=$1)
            ORDER BY created_at DESC,id DESC LIMIT $2
            "#,
        )
        .bind(status)
        .bind(i64::from(limit.clamp(1, 1000)))
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(ai_suggestion_from_row).collect()
    }

    pub(super) async fn read_ai_suggestion(
        &self,
        id: Uuid,
    ) -> PersistenceResult<AiAnalysisSuggestionRecord> {
        let row = sqlx::query("SELECT id,response_id,suggestion_type,title,summary,severity,scope,payload,evidence,status,created_at,decided_at,decision_note,linked_candidate_id FROM analytics.ai_suggestions WHERE id=$1")
            .bind(id).fetch_optional(&self.pool).await?.ok_or_else(|| PersistenceError::InvalidState("AI 建议不存在".to_string()))?;
        ai_suggestion_from_row(&row)
    }
}
