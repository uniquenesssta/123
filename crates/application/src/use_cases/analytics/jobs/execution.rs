use super::{super::calculation::refresh_analytics, request::parse_refresh_request};
use crate::{
    ports::analytics::{
        AnalyticsJobProgressPayload, AnalyticsJobResult, AnalyticsPort, JobQueuePort,
    },
    ApplicationError, ApplicationResult,
};
use serde_json::Value;
use uuid::Uuid;

pub(super) async fn execute_job<P>(
    port: &P,
    job_type: &str,
    payload: &Value,
    job_id: Uuid,
) -> ApplicationResult<AnalyticsJobResult>
where
    P: AnalyticsPort + JobQueuePort + ?Sized,
{
    match job_type {
        "refresh_analytics" => {
            let request = parse_refresh_request(payload)?;
            if port
                .update_progress(
                    job_id,
                    15.0,
                    "正在读取已复盘推演",
                    AnalyticsJobProgressPayload::Empty,
                )
                .await?
            {
                return Err(ApplicationError::Validation("任务已取消".to_string()));
            }
            Ok(AnalyticsJobResult::AnalyticsOverview(Box::new(
                refresh_analytics(port, &request).await?,
            )))
        }
        "data_quality_scan" => {
            if port
                .update_progress(
                    job_id,
                    20.0,
                    "正在检查数据完整性",
                    AnalyticsJobProgressPayload::Empty,
                )
                .await?
            {
                return Err(ApplicationError::Validation("任务已取消".to_string()));
            }
            Ok(AnalyticsJobResult::DataQuality(
                port.run_data_quality_scan().await?,
            ))
        }
        "query_performance_scan" => {
            if port
                .update_progress(
                    job_id,
                    20.0,
                    "正在读取 PostgreSQL 统计信息",
                    AnalyticsJobProgressPayload::Empty,
                )
                .await?
            {
                return Err(ApplicationError::Validation("任务已取消".to_string()));
            }
            Ok(AnalyticsJobResult::QueryPerformance(
                port.capture_query_performance().await?,
            ))
        }
        "full_analysis_refresh" => {
            let request = parse_refresh_request(payload)?;
            if port
                .update_progress(
                    job_id,
                    10.0,
                    "正在刷新模型评估",
                    AnalyticsJobProgressPayload::Empty,
                )
                .await?
            {
                return Err(ApplicationError::Validation("任务已取消".to_string()));
            }
            let overview = refresh_analytics(port, &request).await?;
            if port
                .update_progress(
                    job_id,
                    55.0,
                    "正在扫描数据质量",
                    AnalyticsJobProgressPayload::SampleSize(overview.sample_size),
                )
                .await?
            {
                return Err(ApplicationError::Validation("任务已取消".to_string()));
            }
            let quality = port.run_data_quality_scan().await?;
            if port
                .update_progress(
                    job_id,
                    82.0,
                    "正在分析数据库查询",
                    AnalyticsJobProgressPayload::OpenFindings(quality.open_total),
                )
                .await?
            {
                return Err(ApplicationError::Validation("任务已取消".to_string()));
            }
            let query = port.capture_query_performance().await?;
            Ok(AnalyticsJobResult::FullAnalysisRefresh {
                sample_size: overview.sample_size,
                expected_calibration_error: overview.expected_calibration_error,
                quality_findings: quality.open_total,
                database_size_bytes: query.database_size_bytes,
            })
        }
        other => Err(ApplicationError::Validation(format!(
            "不支持的后台任务：{other}"
        ))),
    }
}
