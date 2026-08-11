use crate::{
    ports::analytics::{
        AnalyticsJobProgressPayload, AnalyticsJobResult, AnalyticsPort, JobQueuePort,
    },
    ApplicationError, ApplicationResult,
};
use football_domain::AnalyticsRefreshRequest;
use serde_json::Value;
use uuid::Uuid;

pub(crate) fn spawn<P>(port: P)
where
    P: AnalyticsPort + JobQueuePort + Clone + Send + Sync + 'static,
{
    tokio::spawn(async move {
        loop {
            let job = match port
                .claim_next_by_types(&[
                    "refresh_analytics",
                    "data_quality_scan",
                    "query_performance_scan",
                    "full_analysis_refresh",
                ])
                .await
            {
                Ok(Some(job)) => job,
                Ok(None) => break,
                Err(_) => break,
            };
            let result = execute_job(&port, &job.job_type, &job.payload, job.id).await;
            match result {
                Ok(value) => {
                    let _ = port.complete(job.id, value).await;
                }
                Err(error) => {
                    let _ = port.fail(job.id, &error.to_string()).await;
                }
            }
        }
    });
}

async fn execute_job<P>(
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
                port.refresh(&request).await?,
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
            let overview = port.refresh(&request).await?;
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

fn parse_refresh_request(payload: &Value) -> ApplicationResult<AnalyticsRefreshRequest> {
    if payload.is_null() || payload.as_object().is_some_and(|item| item.is_empty()) {
        Ok(AnalyticsRefreshRequest::default())
    } else {
        Ok(serde_json::from_value(payload.clone())?)
    }
}
