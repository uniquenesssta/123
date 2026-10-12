use super::mapping::file_from_row;
use crate::{PersistenceError, PersistenceResult, PostgresStore};
use football_domain::ApiWorkspaceGeneratedFileContent;
use sqlx::Row;
use uuid::Uuid;

impl PostgresStore {
    pub async fn read_api_workspace_generated_file(
        &self,
        file_id: Uuid,
    ) -> PersistenceResult<ApiWorkspaceGeneratedFileContent> {
        let row = sqlx::query(
            r#"
            SELECT id, session_id, message_id, filename, media_type, content_sha256,
                   octet_length(content)::bigint AS size_bytes, content, created_at
            FROM ai_workspace.generated_files
            WHERE id = $1
            "#,
        )
        .bind(file_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("API生成文件不存在".to_string()))?;
        Ok(ApiWorkspaceGeneratedFileContent {
            file: file_from_row(&row)?,
            content: row.try_get("content")?,
        })
    }
}
