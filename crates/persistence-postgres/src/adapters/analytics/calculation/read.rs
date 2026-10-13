use super::projection::overview_from_calculation;
use crate::{PersistenceResult, PostgresStore};
use football_domain::AnalyticsOverview;
use serde_json::Value;

impl PostgresStore {
    pub async fn analytics_overview(&self) -> PersistenceResult<AnalyticsOverview> {
        let calculation: Option<football_domain::AnalyticsCalculation> =
            sqlx::query_scalar::<_, Value>(
                "SELECT summary FROM analytics.analysis_snapshots ORDER BY created_at DESC, id DESC LIMIT 1",
            )
            .fetch_optional(&self.pool)
            .await?
            .map(serde_json::from_value)
            .transpose()?;
        let quality = self.latest_data_quality_summary(100).await?;
        let query_performance = self.latest_query_performance().await?;
        Ok(overview_from_calculation(
            calculation,
            quality,
            query_performance,
        ))
    }
}
