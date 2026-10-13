use crate::{PersistenceResult, PostgresStore};
use football_domain::{AiAnalysisPackageSummary, ANALYTICS_CALCULATION_VERSION};
use uuid::Uuid;

impl PostgresStore {
    pub async fn record_ai_export(
        &self,
        summary: &AiAnalysisPackageSummary,
    ) -> PersistenceResult<()> {
        sqlx::query("INSERT INTO analytics.ai_package_exports (id,package_id,output_path,content_sha256,sample_size,calculation_version,metadata) VALUES ($1,$2,$3,$4,$5,$6,'{}'::jsonb)")
            .bind(Uuid::new_v4())
            .bind(summary.package_id)
            .bind(&summary.output_path)
            .bind(&summary.content_sha256)
            .bind(summary.sample_size as i64)
            .bind(ANALYTICS_CALCULATION_VERSION)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
