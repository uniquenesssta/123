use crate::PersistenceResult;
use chrono::{DateTime, Utc};
use football_domain::{
    AnalyticsRefreshRequest, CalibrationBucket, DriftFinding, ANALYTICS_CALCULATION_VERSION,
};
use sqlx::Postgres;
use uuid::Uuid;

pub(super) async fn insert_calibration_bucket(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    snapshot_id: Uuid,
    request: &AnalyticsRefreshRequest,
    window_start: Option<DateTime<Utc>>,
    window_end: Option<DateTime<Utc>>,
    bucket: &CalibrationBucket,
) -> PersistenceResult<()> {
    sqlx::query("INSERT INTO analytics.calibration_buckets (id,snapshot_id,competition_id,outcome,bucket_index,lower_bound,upper_bound,sample_size,predicted_mean,actual_rate,absolute_gap,ece_component,window_start,window_end,calculation_version) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15)")
        .bind(Uuid::new_v4()).bind(snapshot_id).bind(request.competition_id).bind(&bucket.outcome).bind(i16::from(bucket.bucket_index))
        .bind(bucket.lower_bound).bind(bucket.upper_bound).bind(bucket.sample_size as i64).bind(bucket.predicted_mean).bind(bucket.actual_rate)
        .bind(bucket.absolute_gap).bind(bucket.ece_component).bind(window_start).bind(window_end).bind(ANALYTICS_CALCULATION_VERSION)
        .execute(&mut **tx).await?;
    Ok(())
}

pub(super) async fn insert_drift_finding(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    snapshot_id: Uuid,
    request: &AnalyticsRefreshRequest,
    finding: &DriftFinding,
) -> PersistenceResult<()> {
    sqlx::query("INSERT INTO analytics.drift_snapshots (id,snapshot_id,competition_id,metric_name,baseline_mean,current_mean,absolute_delta,relative_delta,baseline_size,current_size,severity,direction,details,calculation_version) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,'{}'::jsonb,$13)")
        .bind(Uuid::new_v4()).bind(snapshot_id).bind(request.competition_id).bind(&finding.metric_name).bind(finding.baseline_mean)
        .bind(finding.current_mean).bind(finding.absolute_delta).bind(finding.relative_delta).bind(finding.baseline_size as i64)
        .bind(finding.current_size as i64).bind(&finding.severity).bind(&finding.direction).bind(ANALYTICS_CALCULATION_VERSION)
        .execute(&mut **tx).await?;
    Ok(())
}
