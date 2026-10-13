use super::policy::{suggestion_severity, validate_import_preview};
use crate::{write_audit_event, PersistenceError, PersistenceResult, PostgresStore};
use football_domain::{AiAnalysisResponsePreview, AiAnalysisSuggestionRecord};
use serde_json::json;
use uuid::Uuid;

impl PostgresStore {
    pub async fn import_ai_response(
        &self,
        input_path: &str,
        preview: &AiAnalysisResponsePreview,
    ) -> PersistenceResult<Vec<AiAnalysisSuggestionRecord>> {
        validate_import_preview(preview)?;
        let mut tx = self.pool.begin().await?;
        let already_imported: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM analytics.ai_package_imports WHERE response_id=$1)",
        )
        .bind(preview.manifest.response_id)
        .fetch_one(&mut *tx)
        .await?;
        if already_imported {
            return Err(PersistenceError::InvalidState(
                "该 AI 回包已经导入，不能重复写入建议".to_string(),
            ));
        }
        if let Some(source_package_id) = preview.manifest.source_package_id {
            let source_exists: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM analytics.ai_package_exports WHERE package_id=$1)",
            )
            .bind(source_package_id)
            .fetch_one(&mut *tx)
            .await?;
            if !source_exists {
                return Err(PersistenceError::InvalidState(
                    "AI 回包引用的来源分析包不属于当前数据库".to_string(),
                ));
            }
        }
        sqlx::query(
            r#"
            INSERT INTO analytics.ai_package_imports (
                id,response_id,source_package_id,input_path,content_sha256,suggestion_count,status,warnings,imported_at
            ) VALUES ($1,$2,$3,$4,$5,$6,'imported',$7,now())
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(preview.manifest.response_id)
        .bind(preview.manifest.source_package_id)
        .bind(input_path)
        .bind(&preview.manifest.content_sha256)
        .bind(preview.suggestions.len() as i32)
        .bind(serde_json::to_value(&preview.warnings)?)
        .execute(&mut *tx)
        .await?;
        for suggestion in &preview.suggestions {
            sqlx::query(
                r#"
                INSERT INTO analytics.ai_suggestions (
                    id,response_id,suggestion_type,title,summary,severity,scope,payload,evidence,status
                ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,'pending')
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(preview.manifest.response_id)
            .bind(&suggestion.suggestion_type)
            .bind(&suggestion.title)
            .bind(&suggestion.summary)
            .bind(suggestion_severity(&suggestion.severity))
            .bind(&suggestion.scope)
            .bind(&suggestion.payload)
            .bind(&suggestion.evidence)
            .execute(&mut *tx)
            .await?;
        }
        write_audit_event(
            &mut tx,
            "ai_analysis_response_imported",
            "ai_response",
            Some(preview.manifest.response_id.to_string()),
            json!({"suggestion_count": preview.suggestions.len(), "source_package_id": preview.manifest.source_package_id}),
        ).await?;
        tx.commit().await?;
        self.list_ai_suggestions(Some("pending"), 500).await
    }
}
