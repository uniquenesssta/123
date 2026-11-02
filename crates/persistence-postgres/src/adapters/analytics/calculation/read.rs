use super::projection::overview_from_calculation;
use crate::PersistenceResult;
use crate::PostgresStore;
use football_domain::{AnalyticsOverview, AnalyticsSnapshotIdentity};
use serde_json::Value;
use sqlx::Row;
use uuid::Uuid;

impl PostgresStore {
    pub async fn analytics_overview(&self) -> PersistenceResult<AnalyticsOverview> {
        self.read_analysis_snapshot_overview(None).await
    }

    // A completed request reads the snapshot it inserted, never whichever concurrent request finished last.
    pub async fn read_analysis_snapshot_overview(
        &self,
        snapshot_id: Option<Uuid>,
    ) -> PersistenceResult<AnalyticsOverview> {
        let row = sqlx::query(
            "SELECT id,competition_id,window_start,window_end,summary FROM analytics.analysis_snapshots WHERE ($1::uuid IS NULL OR id=$1) ORDER BY created_at DESC,id DESC LIMIT 1",
        ).bind(snapshot_id).fetch_optional(&self.pool).await?;
        if snapshot_id.is_some() && row.is_none() {
            return Err(crate::PersistenceError::InvalidState(
                "原分析快照已不存在，不能用其他最新结果替代".to_string(),
            ));
        }
        let (calculation, source) = if let Some(row) = row {
            let summary: Value = row.try_get("summary")?;
            let request = summary
                .get("request")
                .cloned()
                .map(serde_json::from_value)
                .transpose()?;
            let source = AnalyticsSnapshotIdentity {
                id: row.try_get("id")?,
                competition_id: row.try_get("competition_id")?,
                window_start: row.try_get("window_start")?,
                window_end: row.try_get("window_end")?,
                request,
            };
            (Some(serde_json::from_value(summary)?), Some(source))
        } else {
            (None, None)
        };
        let quality = self.latest_data_quality_summary(100).await?;
        let query = self.latest_query_performance().await?;
        let mut overview = overview_from_calculation(calculation, quality, query);
        overview.source_snapshot = source;
        Ok(overview)
    }
}
