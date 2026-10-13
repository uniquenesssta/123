use super::{
    metrics::{insert_calibration_bucket, insert_drift_finding},
    projection::{analysis_windows, sample_group_ids},
};
use crate::{write_audit_event, PersistenceResult, PostgresStore};
use football_analytics_engine::calculate_analytics;
use football_domain::{AnalyticsOverview, AnalyticsRefreshRequest, ANALYTICS_CALCULATION_VERSION};
use serde_json::json;
use sqlx::{Postgres, QueryBuilder};
use uuid::Uuid;

impl PostgresStore {
    pub async fn refresh_analytics(
        &self,
        request: &AnalyticsRefreshRequest,
    ) -> PersistenceResult<AnalyticsOverview> {
        let samples = self.load_evaluation_samples(request).await?;
        let calculation = calculate_analytics(
            &samples,
            request.bucket_count,
            request.baseline_size,
            request.current_size,
        );
        let snapshot_id = Uuid::new_v4();
        let mut tx = self.pool.begin().await?;

        for chunk in samples.chunks(500) {
            let mut builder = QueryBuilder::<Postgres>::new(
                "INSERT INTO analytics.evaluation_samples (id, review_id, run_id, model_version_id, parameter_set_id, competition_id, season_id, stage_id, snapshot_type, kickoff_time, actual_outcome, home_win, draw, away_win, log_loss, brier, scoreline_nll, data_coverage, calculation_version, calculated_at) ",
            );
            builder.push_values(chunk, |mut row, sample| {
                row.push_bind(Uuid::new_v4())
                    .push_bind(sample.review_id)
                    .push_bind(sample.run_id)
                    .push_bind(sample.model_version_id)
                    .push_bind(sample.parameter_set_id)
                    .push_bind(sample.competition_id)
                    .push_bind(sample.season_id)
                    .push_bind(sample.stage_id)
                    .push_bind(&sample.snapshot_type)
                    .push_bind(sample.kickoff_time)
                    .push_bind(&sample.actual_outcome)
                    .push_bind(sample.home_win)
                    .push_bind(sample.draw)
                    .push_bind(sample.away_win)
                    .push_bind(sample.log_loss)
                    .push_bind(sample.brier)
                    .push_bind(sample.scoreline_nll)
                    .push_bind(sample.data_coverage)
                    .push_bind(ANALYTICS_CALCULATION_VERSION)
                    .push_bind(calculation.generated_at);
            });
            builder.push(
                " ON CONFLICT (run_id, kickoff_time, calculation_version) DO UPDATE SET \
                  review_id=EXCLUDED.review_id, model_version_id=EXCLUDED.model_version_id, \
                  parameter_set_id=EXCLUDED.parameter_set_id, competition_id=EXCLUDED.competition_id, \
                  season_id=EXCLUDED.season_id, stage_id=EXCLUDED.stage_id, snapshot_type=EXCLUDED.snapshot_type, \
                  actual_outcome=EXCLUDED.actual_outcome, home_win=EXCLUDED.home_win, draw=EXCLUDED.draw, \
                  away_win=EXCLUDED.away_win, log_loss=EXCLUDED.log_loss, brier=EXCLUDED.brier, \
                  scoreline_nll=EXCLUDED.scoreline_nll, data_coverage=EXCLUDED.data_coverage, \
                  calculated_at=EXCLUDED.calculated_at",
            );
            builder.build().execute(&mut *tx).await?;
        }

        let (window_start, window_end, metric_window_start, metric_window_end) =
            analysis_windows(request, &samples, calculation.generated_at);
        let summary = serde_json::to_value(&calculation)?;
        sqlx::query(
            "INSERT INTO analytics.analysis_snapshots (id, competition_id, window_start, window_end, sample_size, summary, calculation_version, created_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8)",
        )
        .bind(snapshot_id)
        .bind(request.competition_id)
        .bind(window_start)
        .bind(window_end)
        .bind(calculation.sample_size as i64)
        .bind(&summary)
        .bind(&calculation.calculation_version)
        .bind(calculation.generated_at)
        .execute(&mut *tx)
        .await?;

        for bucket in &calculation.calibration {
            insert_calibration_bucket(
                &mut tx,
                snapshot_id,
                request,
                window_start,
                window_end,
                bucket,
            )
            .await?;
        }
        for finding in &calculation.drift {
            insert_drift_finding(&mut tx, snapshot_id, request, finding).await?;
        }

        let ids_by_group = sample_group_ids(&samples);
        for comparison in &calculation.comparisons {
            let key = (
                comparison.model_key.clone(),
                comparison.model_version.clone(),
                comparison.parameter_version.clone(),
                comparison.snapshot_type.clone(),
            );
            if let Some((model_version_id, parameter_set_id)) = ids_by_group.get(&key) {
                let metrics = serde_json::to_value(comparison)?;
                sqlx::query(
                    r#"
                    INSERT INTO analytics.model_metrics (
                        id, model_version_id, parameter_set_id, competition_id,
                        window_start, window_end, sample_size, metrics,
                        calculation_version, calculated_at
                    ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
                    ON CONFLICT (model_version_id, parameter_set_id, competition_id, window_start, window_end, calculation_version)
                    DO UPDATE SET sample_size=EXCLUDED.sample_size, metrics=EXCLUDED.metrics, calculated_at=EXCLUDED.calculated_at
                    "#,
                )
                .bind(Uuid::new_v4())
                .bind(*model_version_id)
                .bind(*parameter_set_id)
                .bind(request.competition_id)
                .bind(metric_window_start)
                .bind(metric_window_end)
                .bind(comparison.sample_size as i64)
                .bind(metrics)
                .bind(&calculation.calculation_version)
                .bind(calculation.generated_at)
                .execute(&mut *tx)
                .await?;
            }
        }

        write_audit_event(
            &mut tx,
            "analytics_refreshed",
            "analysis_snapshot",
            Some(snapshot_id.to_string()),
            json!({"sample_size": calculation.sample_size, "competition_id": request.competition_id}),
        )
        .await?;
        tx.commit().await?;
        self.analytics_overview().await
    }
}
