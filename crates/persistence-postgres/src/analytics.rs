use super::{write_audit_event, PersistenceError, PersistenceResult, PostgresStore};
use chrono::{DateTime, Utc};
use football_analytics_engine::calculate_analytics;
use football_domain::{
    AiAnalysisPackageData, AiAnalysisPackageSummary, AiAnalysisResponsePreview,
    AiAnalysisSuggestionRecord, AiSuggestionDecision, AiSuggestionDecisionDraft, AnalyticsOverview,
    AnalyticsRefreshRequest, CalibrationBucket, DataQualityDecisionDraft, DataQualityFinding,
    DataQualitySummary, DriftFinding, EvaluationSample, ParameterTuningCandidateRecord,
    ParameterTuningDecision, ParameterTuningDecisionDraft, QueryPerformanceFinding,
    QueryPerformanceSummary, ANALYTICS_CALCULATION_VERSION,
};
use serde_json::{json, Value};
use sqlx::postgres::PgPoolCopyExt;
use sqlx::{Postgres, QueryBuilder, Row};
use std::{collections::HashMap, time::Instant};
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

        let window_start = request
            .window_start
            .or_else(|| samples.iter().map(|item| item.kickoff_time).min());
        let window_end = request
            .window_end
            .or_else(|| samples.iter().map(|item| item.kickoff_time).max());
        let metric_window_start = window_start.unwrap_or(calculation.generated_at);
        let metric_window_end = window_end.unwrap_or(calculation.generated_at);
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
        if let Some(calculation) = calculation {
            Ok(AnalyticsOverview {
                generated_at: Some(calculation.generated_at),
                calculation_version: calculation.calculation_version,
                sample_size: calculation.sample_size,
                average_log_loss: calculation.average_log_loss,
                average_brier: calculation.average_brier,
                average_scoreline_nll: calculation.average_scoreline_nll,
                expected_calibration_error: calculation.expected_calibration_error,
                comparisons: calculation.comparisons,
                calibration: calculation.calibration,
                drift: calculation.drift,
                data_quality: quality,
                query_performance,
            })
        } else {
            Ok(AnalyticsOverview {
                generated_at: None,
                calculation_version: ANALYTICS_CALCULATION_VERSION.to_string(),
                sample_size: 0,
                average_log_loss: None,
                average_brier: None,
                average_scoreline_nll: None,
                expected_calibration_error: None,
                comparisons: Vec::new(),
                calibration: Vec::new(),
                drift: Vec::new(),
                data_quality: quality,
                query_performance,
            })
        }
    }

    pub async fn run_data_quality_scan(&self) -> PersistenceResult<DataQualitySummary> {
        let scan_id = Uuid::new_v4();
        sqlx::query("INSERT INTO analytics.data_quality_scans (id,status,scope) VALUES ($1,'running','{}'::jsonb)")
            .bind(scan_id)
            .execute(&self.pool)
            .await?;
        let result = self.collect_data_quality_findings(scan_id).await;
        match result {
            Ok(findings) => {
                let critical = findings
                    .iter()
                    .filter(|item| item.severity == "critical")
                    .count() as i64;
                let warning = findings
                    .iter()
                    .filter(|item| item.severity == "warning")
                    .count() as i64;
                let info = findings
                    .iter()
                    .filter(|item| item.severity == "info")
                    .count() as i64;
                let summary = json!({"critical": critical, "warning": warning, "info": info, "open_total": findings.len()});
                sqlx::query("UPDATE analytics.data_quality_scans SET status='succeeded', summary=$2, finished_at=now() WHERE id=$1")
                    .bind(scan_id)
                    .bind(&summary)
                    .execute(&self.pool)
                    .await?;
                Ok(DataQualitySummary {
                    scan_id: Some(scan_id),
                    generated_at: Some(Utc::now()),
                    critical,
                    warning,
                    info,
                    open_total: findings.len() as i64,
                    findings,
                })
            }
            Err(error) => {
                sqlx::query("UPDATE analytics.data_quality_scans SET status='failed', error_message=$2, finished_at=now() WHERE id=$1")
                    .bind(scan_id)
                    .bind(error.to_string())
                    .execute(&self.pool)
                    .await?;
                Err(error)
            }
        }
    }

    pub async fn capture_query_performance(&self) -> PersistenceResult<QueryPerformanceSummary> {
        let database_size_bytes: i64 =
            sqlx::query_scalar("SELECT pg_database_size(current_database())::bigint")
                .fetch_one(&self.pool)
                .await?;
        let rows = sqlx::query(
            r#"
            SELECT
                schemaname, relname,
                COALESCE(n_live_tup,0)::bigint AS estimated_rows,
                pg_total_relation_size(format('%I.%I', schemaname, relname)::regclass)::bigint AS table_size_bytes,
                COALESCE(seq_scan,0)::bigint AS sequential_scans,
                COALESCE(idx_scan,0)::bigint AS index_scans,
                COALESCE(n_dead_tup,0)::bigint AS dead_rows,
                last_analyze
            FROM pg_stat_user_tables
            WHERE schemaname IN ('football','feature','model','review','analytics','catalog','audit','platform')
            ORDER BY pg_total_relation_size(format('%I.%I', schemaname, relname)::regclass) DESC
            LIMIT 100
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        let tables = rows
            .iter()
            .map(|row| {
                let estimated_rows: i64 = row.try_get("estimated_rows")?;
                let sequential_scans: i64 = row.try_get("sequential_scans")?;
                let index_scans: i64 = row.try_get("index_scans")?;
                let dead_rows: i64 = row.try_get("dead_rows")?;
                let ratio = if estimated_rows > 0 {
                    dead_rows as f64 / estimated_rows as f64
                } else {
                    0.0
                };
                let (severity, recommendation) = if ratio > 0.20 {
                    (
                        "warning",
                        Some("死元组比例较高，安排 ANALYZE/VACUUM 并检查更新频率".to_string()),
                    )
                } else if estimated_rows > 10_000
                    && sequential_scans > index_scans.saturating_mul(4)
                {
                    (
                        "warning",
                        Some("顺序扫描明显高于索引扫描，请结合实际查询检查组合索引".to_string()),
                    )
                } else {
                    ("stable", None)
                };
                Ok(QueryPerformanceFinding {
                    schema_name: row.try_get("schemaname")?,
                    table_name: row.try_get("relname")?,
                    estimated_rows,
                    table_size_bytes: row.try_get("table_size_bytes")?,
                    sequential_scans,
                    index_scans,
                    dead_rows,
                    last_analyze: row.try_get("last_analyze")?,
                    severity: severity.to_string(),
                    recommendation,
                })
            })
            .collect::<PersistenceResult<Vec<_>>>()?;
        let summary = QueryPerformanceSummary {
            captured_at: Some(Utc::now()),
            database_size_bytes,
            tables,
        };
        sqlx::query("INSERT INTO analytics.query_performance_snapshots (id,database_size_bytes,tables,recommendations,captured_at) VALUES ($1,$2,$3,$4,$5)")
            .bind(Uuid::new_v4())
            .bind(database_size_bytes)
            .bind(serde_json::to_value(&summary.tables)?)
            .bind(serde_json::to_value(summary.tables.iter().filter_map(|item| item.recommendation.clone()).collect::<Vec<_>>())?)
            .bind(summary.captured_at)
            .execute(&self.pool)
            .await?;
        Ok(summary)
    }

    pub async fn build_ai_analysis_data(&self) -> PersistenceResult<AiAnalysisPackageData> {
        let overview = self.analytics_overview().await?;
        let database_summary = self.database_summary_json().await?;
        let player_review_summary = self.player_review_summary_json().await?;
        let team_review_summary = self.team_review_summary_json().await?;
        let ability_candidates = self.ability_candidates_json().await?;
        let schema_summary = self.schema_summary_json().await?;
        Ok(AiAnalysisPackageData {
            overview,
            database_summary,
            player_review_summary,
            team_review_summary,
            ability_candidates,
            schema_summary,
        })
    }

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

    pub async fn import_ai_response(
        &self,
        input_path: &str,
        preview: &AiAnalysisResponsePreview,
    ) -> PersistenceResult<Vec<AiAnalysisSuggestionRecord>> {
        if !preview.blocking_errors.is_empty() {
            return Err(PersistenceError::InvalidState(format!(
                "AI 回包存在阻断错误：{}",
                preview.blocking_errors.join("；")
            )));
        }
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
            .bind(if suggestion.severity.trim().is_empty() { "info" } else { suggestion.severity.as_str() })
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

    pub async fn decide_ai_suggestion(
        &self,
        draft: &AiSuggestionDecisionDraft,
    ) -> PersistenceResult<AiAnalysisSuggestionRecord> {
        let mut tx = self.pool.begin().await?;
        let row = sqlx::query("SELECT suggestion_type,payload,status FROM analytics.ai_suggestions WHERE id=$1 FOR UPDATE")
            .bind(draft.suggestion_id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(|| PersistenceError::InvalidState("AI 建议不存在".to_string()))?;
        let status: String = row.try_get("status")?;
        if status != "pending" {
            return Err(PersistenceError::InvalidState(
                "该 AI 建议已处理".to_string(),
            ));
        }
        let suggestion_type: String = row.try_get("suggestion_type")?;
        let payload: Value = row.try_get("payload")?;
        let decided_by = draft.decided_by.as_deref().unwrap_or("desktop-user");
        let mut linked_candidate_id = None;
        let new_status = match draft.decision {
            AiSuggestionDecision::Reject => "rejected",
            AiSuggestionDecision::Accept if suggestion_type == "ability_update" => {
                let player_id = json_uuid(&payload, "player_id")?;
                let dimension_code = json_string(&payload, "dimension_code")?;
                let identity_valid: bool = sqlx::query_scalar(
                    r#"
                    SELECT EXISTS(
                        SELECT 1
                        FROM football.players player
                        CROSS JOIN feature.player_ability_dimensions dimension
                        WHERE player.id=$1 AND dimension.code=$2
                    )
                    "#,
                )
                .bind(player_id)
                .bind(&dimension_code)
                .fetch_one(&mut *tx)
                .await?;
                if !identity_valid {
                    return Err(PersistenceError::InvalidState(
                        "AI 能力建议引用了不存在的球员或能力维度".to_string(),
                    ));
                }
                let proposed_value = json_number(&payload, "proposed_value")?.clamp(0.0, 100.0);
                let confidence = payload
                    .get("confidence")
                    .and_then(Value::as_f64)
                    .unwrap_or(0.5)
                    .clamp(0.0, 1.0);
                let sample_size = payload
                    .get("sample_size")
                    .and_then(Value::as_i64)
                    .unwrap_or(0)
                    .clamp(0, i32::MAX as i64) as i32;
                let current_value: Option<f64> = sqlx::query_scalar(
                    "SELECT value FROM feature.player_current_abilities WHERE player_id=$1 AND dimension_code=$2",
                )
                .bind(player_id)
                .bind(&dimension_code)
                .fetch_optional(&mut *tx)
                .await?;
                let candidate_id = Uuid::new_v4();
                sqlx::query(
                    r#"
                    INSERT INTO review.ability_update_candidates (
                        id,player_id,dimension_code,current_value,proposed_value,confidence,sample_size,
                        evidence,calculation_version,status
                    ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,'ai-analysis-response-v1','pending')
                    "#,
                )
                .bind(candidate_id)
                .bind(player_id)
                .bind(&dimension_code)
                .bind(current_value)
                .bind(proposed_value)
                .bind(confidence)
                .bind(sample_size)
                .bind(json!({
                    "source": "ai_analysis_response",
                    "suggestion_id": draft.suggestion_id,
                    "accepted_by": decided_by,
                    "payload": payload
                }))
                .execute(&mut *tx)
                .await?;
                linked_candidate_id = Some(candidate_id);
                "accepted"
            }
            AiSuggestionDecision::Accept => "accepted",
        };
        sqlx::query("UPDATE analytics.ai_suggestions SET status=$2,linked_candidate_id=$3,decided_at=now(),decided_by=$4,decision_note=$5 WHERE id=$1")
            .bind(draft.suggestion_id)
            .bind(new_status)
            .bind(linked_candidate_id)
            .bind(decided_by)
            .bind(&draft.decision_note)
            .execute(&mut *tx)
            .await?;
        write_audit_event(
            &mut tx,
            "ai_suggestion_decided",
            "ai_suggestion",
            Some(draft.suggestion_id.to_string()),
            json!({"decision": new_status, "linked_candidate_id": linked_candidate_id}),
        )
        .await?;
        tx.commit().await?;
        self.read_ai_suggestion(draft.suggestion_id).await
    }

    pub async fn decide_data_quality_finding(
        &self,
        draft: &DataQualityDecisionDraft,
    ) -> PersistenceResult<DataQualityFinding> {
        let mut tx = self.pool.begin().await?;
        let status = draft.decision.as_status();
        let row = sqlx::query(
            r#"
            UPDATE analytics.data_quality_findings
            SET status=$2,resolved_at=now(),resolution_note=$3
            WHERE id=$1 AND status='open'
            RETURNING id,scan_id,severity,category,finding_code,entity_type,entity_id,
                      message,evidence,status,detected_at
            "#,
        )
        .bind(draft.finding_id)
        .bind(status)
        .bind(&draft.resolution_note)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| {
            PersistenceError::InvalidState("数据质量问题不存在，或已经被处理".to_string())
        })?;
        write_audit_event(
            &mut tx,
            "data_quality_finding_decided",
            "data_quality_finding",
            Some(draft.finding_id.to_string()),
            json!({"status": status, "resolution_note": draft.resolution_note}),
        )
        .await?;
        tx.commit().await?;
        data_quality_finding_from_row(&row)
    }

    pub async fn stage_bulk_json_rows_copy(
        &self,
        batch_id: Uuid,
        entity_type: &str,
        rows: &[Value],
    ) -> PersistenceResult<u64> {
        if rows.is_empty() {
            return Ok(0);
        }
        let started = Instant::now();
        sqlx::query("DELETE FROM catalog.bulk_import_staging WHERE batch_id=$1")
            .bind(batch_id)
            .execute(&self.pool)
            .await?;
        sqlx::query(
            r#"
            INSERT INTO catalog.bulk_import_runs (
                id,batch_id,import_type,row_count,status,validation_summary
            ) VALUES ($1,$2,$3,0,'staged','{}'::jsonb)
            ON CONFLICT (batch_id) DO UPDATE SET
                import_type=EXCLUDED.import_type,row_count=0,copy_duration_ms=NULL,
                validation_summary='{}'::jsonb,status='staged',finished_at=NULL
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(batch_id)
        .bind(entity_type)
        .execute(&self.pool)
        .await?;

        let mut csv = String::new();
        for (index, payload) in rows.iter().enumerate() {
            let payload_text = serde_json::to_string(payload)?;
            csv.push_str(&batch_id.to_string());
            csv.push('\t');
            csv.push_str(&(index + 1).to_string());
            csv.push('\t');
            csv.push_str(&escape_copy_field(entity_type));
            csv.push('\t');
            csv.push_str(&escape_copy_field(&payload_text));
            csv.push('\t');
            csv.push_str("\\N\n");
        }
        let statement = "COPY catalog.bulk_import_staging (batch_id,row_number,entity_type,payload,payload_sha256) FROM STDIN WITH (FORMAT csv, DELIMITER E'\\t', NULL '\\N')";
        let copy_result: Result<u64, sqlx::Error> = async {
            let mut copy = self.pool.copy_in_raw(statement).await?;
            copy.send(csv.as_bytes()).await?;
            copy.finish().await
        }
        .await;
        match copy_result {
            Ok(count) => {
                sqlx::query(
                    "UPDATE catalog.bulk_import_runs SET row_count=$2,copy_duration_ms=$3,status='staged',finished_at=now() WHERE batch_id=$1",
                )
                .bind(batch_id)
                .bind(count as i64)
                .bind(started.elapsed().as_millis().min(i64::MAX as u128) as i64)
                .execute(&self.pool)
                .await?;
                Ok(count)
            }
            Err(error) => {
                let _ = sqlx::query(
                    "UPDATE catalog.bulk_import_runs SET status='failed',validation_summary=$2,finished_at=now() WHERE batch_id=$1",
                )
                .bind(batch_id)
                .bind(json!({"error": error.to_string()}))
                .execute(&self.pool)
                .await;
                Err(error.into())
            }
        }
    }

    async fn load_evaluation_samples(
        &self,
        request: &AnalyticsRefreshRequest,
    ) -> PersistenceResult<Vec<EvaluationSample>> {
        let rows = sqlx::query(
            r#"
            SELECT DISTINCT ON (run.id)
                review.id AS review_id, run.id AS run_id,
                run.model_version_id, run.parameter_set_id,
                definition.model_key, version.version AS model_version,
                parameter.parameter_version,
                fixture.competition_id, competition.name AS competition_name,
                fixture.season_id, fixture.stage_id, run.snapshot_type, fixture.kickoff_time,
                review.prediction_evaluation->>'actual_outcome' AS actual_outcome,
                (review.prediction_evaluation->'predicted_probabilities'->>'home_win')::double precision AS home_win,
                (review.prediction_evaluation->'predicted_probabilities'->>'draw')::double precision AS draw,
                (review.prediction_evaluation->'predicted_probabilities'->>'away_win')::double precision AS away_win,
                (review.prediction_evaluation->>'log_loss')::double precision AS log_loss,
                (review.prediction_evaluation->>'brier')::double precision AS brier,
                NULLIF(review.prediction_evaluation->>'scoreline_nll','')::double precision AS scoreline_nll,
                review.data_coverage::double precision AS data_coverage
            FROM review.match_reviews review
            JOIN model.runs run ON run.id=review.source_run_id AND run.status='succeeded'
            JOIN model.versions version ON version.id=run.model_version_id
            JOIN model.definitions definition ON definition.id=version.model_id
            JOIN model.parameter_sets parameter ON parameter.id=run.parameter_set_id
            JOIN football.matches fixture ON fixture.id=review.match_id
            LEFT JOIN football.competitions competition ON competition.id=fixture.competition_id
            WHERE review.status='finalized'
              AND COALESCE((review.prediction_evaluation->>'available')::boolean,false)=true
              AND ($1::uuid IS NULL OR fixture.competition_id=$1)
              AND ($2::timestamptz IS NULL OR fixture.kickoff_time >= $2)
              AND ($3::timestamptz IS NULL OR fixture.kickoff_time <= $3)
            ORDER BY run.id, review.created_at DESC, review.id DESC
            "#,
        )
        .bind(request.competition_id)
        .bind(request.window_start)
        .bind(request.window_end)
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(evaluation_sample_from_row).collect()
    }

    async fn collect_data_quality_findings(
        &self,
        scan_id: Uuid,
    ) -> PersistenceResult<Vec<DataQualityFinding>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM (
                SELECT 'warning'::text severity,'identity'::text category,'duplicate_player_name'::text finding_code,
                       'player'::text entity_type,NULL::text entity_id,
                       format('规范化姓名 %s 存在 %s 名球员', normalized_name, count(*)) message,
                       jsonb_build_object('normalized_name',normalized_name,'count',count(*)) evidence
                FROM football.players WHERE status <> 'retired' GROUP BY normalized_name HAVING count(*) > 1
                UNION ALL
                SELECT 'critical','match','finished_without_result','match',fixture.id::text,
                       format('已结束比赛 %s 尚无正式赛果',fixture.external_key),
                       jsonb_build_object('match_key',fixture.external_key,'kickoff_time',fixture.kickoff_time)
                FROM football.matches fixture LEFT JOIN football.match_results result ON result.match_id=fixture.id
                WHERE fixture.status='finished' AND result.match_id IS NULL
                UNION ALL
                SELECT 'warning','lineup','invalid_starter_count','lineup',lineup.id::text,
                       format('阵容 %s 的首发人数为 %s',lineup.id,count(player.player_id)),
                       jsonb_build_object('match_id',lineup.match_id,'team_id',lineup.team_id,'starter_count',count(player.player_id))
                FROM football.lineups lineup JOIN football.lineup_players player ON player.lineup_id=lineup.id AND player.is_starter
                WHERE lineup.status='active' GROUP BY lineup.id,lineup.match_id,lineup.team_id HAVING count(player.player_id) <> 11
                UNION ALL
                SELECT 'warning','model','probability_sum','model_run',run.id::text,
                       format('推演 %s 的 1X2 概率和偏离 1',run.id),
                       jsonb_build_object('probability_sum',
                           COALESCE((run.summary->>'home_win')::double precision,0)+COALESCE((run.summary->>'draw')::double precision,0)+COALESCE((run.summary->>'away_win')::double precision,0))
                FROM model.runs run WHERE run.status='succeeded' AND abs(
                    COALESCE((run.summary->>'home_win')::double precision,0)+COALESCE((run.summary->>'draw')::double precision,0)+COALESCE((run.summary->>'away_win')::double precision,0)-1.0
                ) > 0.0001
                UNION ALL
                SELECT 'info','player','missing_current_ability','player',player.id::text,
                       format('球员 %s 尚无当前能力投影',player.canonical_name),
                       jsonb_build_object('player_name',player.canonical_name)
                FROM football.players player WHERE player.status='active' AND NOT EXISTS (
                    SELECT 1 FROM feature.player_current_abilities ability WHERE ability.player_id=player.id
                )
                UNION ALL
                SELECT 'warning','review','low_data_coverage','match_review',review.id::text,
                       format('复盘 %s 的数据覆盖率低于 70%%',review.id),
                       jsonb_build_object('data_coverage',review.data_coverage,'match_id',review.match_id)
                FROM review.match_reviews review WHERE review.status='finalized' AND review.data_coverage < 0.7
                UNION ALL
                SELECT 'info','ability','stale_pending_candidate','ability_candidate',candidate.id::text,
                       format('能力候选 %s 已待审核超过 90 天',candidate.id),
                       jsonb_build_object('player_id',candidate.player_id,'created_at',candidate.created_at)
                FROM review.ability_update_candidates candidate WHERE candidate.status='pending' AND candidate.created_at < now()-interval '90 days'
            ) finding
            ORDER BY CASE severity WHEN 'critical' THEN 1 WHEN 'warning' THEN 2 ELSE 3 END, finding_code
            LIMIT 2000
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        let mut findings = Vec::with_capacity(rows.len());
        let mut tx = self.pool.begin().await?;
        for row in rows {
            let finding = DataQualityFinding {
                id: Uuid::new_v4(),
                scan_id,
                severity: row.try_get("severity")?,
                category: row.try_get("category")?,
                finding_code: row.try_get("finding_code")?,
                entity_type: row.try_get("entity_type")?,
                entity_id: row.try_get("entity_id")?,
                message: row.try_get("message")?,
                evidence: row.try_get("evidence")?,
                status: "open".to_string(),
                detected_at: Utc::now(),
            };
            sqlx::query("INSERT INTO analytics.data_quality_findings (id,scan_id,severity,category,finding_code,entity_type,entity_id,message,evidence,status,detected_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,'open',$10)")
                .bind(finding.id).bind(scan_id).bind(&finding.severity).bind(&finding.category)
                .bind(&finding.finding_code).bind(&finding.entity_type).bind(&finding.entity_id)
                .bind(&finding.message).bind(&finding.evidence).bind(finding.detected_at)
                .execute(&mut *tx).await?;
            findings.push(finding);
        }
        tx.commit().await?;
        Ok(findings)
    }

    async fn latest_data_quality_summary(
        &self,
        limit: u32,
    ) -> PersistenceResult<DataQualitySummary> {
        let scan = sqlx::query("SELECT id,finished_at,summary FROM analytics.data_quality_scans WHERE status='succeeded' ORDER BY finished_at DESC,id DESC LIMIT 1")
            .fetch_optional(&self.pool).await?;
        let Some(scan) = scan else {
            return Ok(DataQualitySummary {
                scan_id: None,
                generated_at: None,
                critical: 0,
                warning: 0,
                info: 0,
                open_total: 0,
                findings: Vec::new(),
            });
        };
        let scan_id: Uuid = scan.try_get("id")?;
        let counts = sqlx::query(
            r#"
            SELECT
                count(*) FILTER (WHERE severity='critical')::bigint AS critical,
                count(*) FILTER (WHERE severity='warning')::bigint AS warning,
                count(*) FILTER (WHERE severity='info')::bigint AS info,
                count(*)::bigint AS open_total
            FROM analytics.data_quality_findings
            WHERE scan_id=$1 AND status='open'
            "#,
        )
        .bind(scan_id)
        .fetch_one(&self.pool)
        .await?;
        let rows = sqlx::query(
            "SELECT id,scan_id,severity,category,finding_code,entity_type,entity_id,message,evidence,status,detected_at FROM analytics.data_quality_findings WHERE scan_id=$1 AND status='open' ORDER BY CASE severity WHEN 'critical' THEN 1 WHEN 'warning' THEN 2 ELSE 3 END, detected_at DESC LIMIT $2",
        )
        .bind(scan_id)
        .bind(i64::from(limit.clamp(1, 500)))
        .fetch_all(&self.pool)
        .await?;
        let findings = rows
            .iter()
            .map(data_quality_finding_from_row)
            .collect::<PersistenceResult<Vec<_>>>()?;
        Ok(DataQualitySummary {
            scan_id: Some(scan_id),
            generated_at: scan.try_get("finished_at")?,
            critical: counts.try_get("critical")?,
            warning: counts.try_get("warning")?,
            info: counts.try_get("info")?,
            open_total: counts.try_get("open_total")?,
            findings,
        })
    }

    async fn latest_query_performance(&self) -> PersistenceResult<QueryPerformanceSummary> {
        let row = sqlx::query("SELECT database_size_bytes,tables,captured_at FROM analytics.query_performance_snapshots ORDER BY captured_at DESC,id DESC LIMIT 1")
            .fetch_optional(&self.pool).await?;
        let Some(row) = row else {
            return Ok(QueryPerformanceSummary {
                captured_at: None,
                database_size_bytes: 0,
                tables: Vec::new(),
            });
        };
        Ok(QueryPerformanceSummary {
            captured_at: row.try_get("captured_at")?,
            database_size_bytes: row.try_get("database_size_bytes")?,
            tables: serde_json::from_value(row.try_get("tables")?)?,
        })
    }

    async fn database_summary_json(&self) -> PersistenceResult<Value> {
        Ok(sqlx::query_scalar(
            r#"SELECT jsonb_build_object(
                'generated_at',now(),'database_size_bytes',pg_database_size(current_database()),
                'counts',jsonb_build_object(
                    'competitions',(SELECT count(*) FROM football.competitions),
                    'teams',(SELECT count(*) FROM football.teams),
                    'players',(SELECT count(*) FROM football.players),
                    'matches',(SELECT count(*) FROM football.matches),
                    'model_runs',(SELECT count(*) FROM model.runs),
                    'match_reviews',(SELECT count(*) FROM review.match_reviews),
                    'ability_observations',(SELECT count(*) FROM feature.player_ability_observations),
                    'pending_ability_candidates',(SELECT count(*) FROM review.ability_update_candidates WHERE status='pending')
                )
            )"#,
        ).fetch_one(&self.pool).await?)
    }

    async fn player_review_summary_json(&self) -> PersistenceResult<Value> {
        let rows = sqlx::query(
            r#"SELECT player.id,player.canonical_name,count(*)::bigint sample_size,
                      avg(review.actual_performance)::double precision average_performance,
                      avg(review.realization_ratio)::double precision average_realization,
                      avg(review.confidence)::double precision average_confidence
               FROM review.player_match_reviews review JOIN football.players player ON player.id=review.player_id
               WHERE review.minutes_played >= 15 GROUP BY player.id,player.canonical_name
               ORDER BY avg(review.realization_ratio) DESC NULLS LAST LIMIT 200"#,
        ).fetch_all(&self.pool).await?;
        Ok(Value::Array(rows.iter().map(|row| json!({
            "player_id": row.try_get::<Uuid,_>("id").ok(),
            "player_name": row.try_get::<String,_>("canonical_name").ok(),
            "sample_size": row.try_get::<i64,_>("sample_size").ok(),
            "average_performance": row.try_get::<Option<f64>,_>("average_performance").ok().flatten(),
            "average_realization": row.try_get::<Option<f64>,_>("average_realization").ok().flatten(),
            "average_confidence": row.try_get::<Option<f64>,_>("average_confidence").ok().flatten(),
        })).collect()))
    }

    async fn team_review_summary_json(&self) -> PersistenceResult<Value> {
        let rows = sqlx::query(
            r#"SELECT team.id,team.canonical_name,count(*)::bigint sample_size,
                      avg(review.chemistry_score)::double precision chemistry,
                      avg(review.bench_strength)::double precision bench_strength,
                      avg(review.substitution_impact)::double precision substitution_impact,
                      avg(review.realization_score)::double precision realization
               FROM review.team_match_reviews review JOIN football.teams team ON team.id=review.team_id
               GROUP BY team.id,team.canonical_name ORDER BY count(*) DESC LIMIT 200"#,
        ).fetch_all(&self.pool).await?;
        Ok(Value::Array(rows.iter().map(|row| json!({
            "team_id": row.try_get::<Uuid,_>("id").ok(),
            "team_name": row.try_get::<String,_>("canonical_name").ok(),
            "sample_size": row.try_get::<i64,_>("sample_size").ok(),
            "chemistry": row.try_get::<Option<f64>,_>("chemistry").ok().flatten(),
            "bench_strength": row.try_get::<Option<f64>,_>("bench_strength").ok().flatten(),
            "substitution_impact": row.try_get::<Option<f64>,_>("substitution_impact").ok().flatten(),
            "realization": row.try_get::<Option<f64>,_>("realization").ok().flatten(),
        })).collect()))
    }

    async fn ability_candidates_json(&self) -> PersistenceResult<Value> {
        let rows = sqlx::query("SELECT candidate.id,candidate.player_id,player.canonical_name,candidate.dimension_code,candidate.current_value,candidate.proposed_value,candidate.confidence,candidate.sample_size,candidate.status,candidate.evidence,candidate.created_at FROM review.ability_update_candidates candidate JOIN football.players player ON player.id=candidate.player_id ORDER BY candidate.created_at DESC LIMIT 1000")
            .fetch_all(&self.pool).await?;
        Ok(Value::Array(rows.iter().map(|row| json!({
            "id": row.try_get::<Uuid,_>("id").ok(),"player_id": row.try_get::<Uuid,_>("player_id").ok(),
            "player_name": row.try_get::<String,_>("canonical_name").ok(),"dimension_code": row.try_get::<String,_>("dimension_code").ok(),
            "current_value": row.try_get::<Option<f64>,_>("current_value").ok().flatten(),"proposed_value": row.try_get::<f64,_>("proposed_value").ok(),
            "confidence": row.try_get::<f64,_>("confidence").ok(),"sample_size": row.try_get::<i32,_>("sample_size").ok(),
            "status": row.try_get::<String,_>("status").ok(),"evidence": row.try_get::<Value,_>("evidence").ok(),"created_at": row.try_get::<DateTime<Utc>,_>("created_at").ok()
        })).collect()))
    }

    async fn schema_summary_json(&self) -> PersistenceResult<Value> {
        let rows = sqlx::query("SELECT table_schema,table_name,count(*)::bigint column_count FROM information_schema.columns WHERE table_schema IN ('platform','catalog','football','feature','model','review','analytics','audit') GROUP BY table_schema,table_name ORDER BY table_schema,table_name")
            .fetch_all(&self.pool).await?;
        Ok(Value::Array(rows.iter().map(|row| json!({"schema":row.try_get::<String,_>("table_schema").ok(),"table":row.try_get::<String,_>("table_name").ok(),"columns":row.try_get::<i64,_>("column_count").ok()})).collect()))
    }

    pub async fn save_parameter_tuning_candidate(
        &self,
        candidate: &ParameterTuningCandidateRecord,
    ) -> PersistenceResult<ParameterTuningCandidateRecord> {
        sqlx::query(
            r#"
            INSERT INTO analytics.parameter_tuning_candidates (
                id, competition_id, competition_profile_id, partition_key,
                model_key, model_version, parameter_version, snapshot_type,
                target_module, sample_size,
                baseline_model_version_id, baseline_parameter_set_id,
                candidate_model_version_id, candidate_parameter_set_id,
                candidate_model_version, candidate_parameter_version, candidate_definition_sha256,
                baseline_metrics, calibration_bias, proposed_adjustments, constraints,
                training_window, validation_window, holdout_window,
                rationale, status
            ) VALUES (
                $1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,
                $14,$15,$16,$17,$18,$19,$20,$21,$22,$23,$24,$25,$26
            )
            "#,
        )
        .bind(candidate.id)
        .bind(candidate.competition_id)
        .bind(candidate.competition_profile_id)
        .bind(candidate.partition_key.as_deref())
        .bind(&candidate.model_key)
        .bind(&candidate.model_version)
        .bind(&candidate.parameter_version)
        .bind(&candidate.snapshot_type)
        .bind(&candidate.target_module)
        .bind(candidate.sample_size as i64)
        .bind(candidate.baseline_model_version_id)
        .bind(candidate.baseline_parameter_set_id)
        .bind(candidate.candidate_model_version_id)
        .bind(candidate.candidate_parameter_set_id)
        .bind(candidate.candidate_model_version.as_deref())
        .bind(candidate.candidate_parameter_version.as_deref())
        .bind(candidate.candidate_definition_sha256.as_deref())
        .bind(&candidate.baseline_metrics)
        .bind(&candidate.calibration_bias)
        .bind(&candidate.proposed_adjustments)
        .bind(&candidate.constraints)
        .bind(&candidate.training_window)
        .bind(&candidate.validation_window)
        .bind(&candidate.holdout_window)
        .bind(&candidate.rationale)
        .bind(&candidate.status)
        .execute(&self.pool)
        .await?;
        self.read_parameter_tuning_candidate(candidate.id).await
    }

    pub async fn list_parameter_tuning_candidates(
        &self,
        limit: u32,
    ) -> PersistenceResult<Vec<ParameterTuningCandidateRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT candidate.id, candidate.competition_id, competition.name AS competition_name,
                   candidate.competition_profile_id, candidate.partition_key,
                   candidate.model_key, candidate.model_version, candidate.parameter_version,
                   candidate.snapshot_type, candidate.target_module, candidate.sample_size,
                   candidate.baseline_model_version_id, candidate.baseline_parameter_set_id,
                   candidate.candidate_model_version_id, candidate.candidate_parameter_set_id,
                   candidate.candidate_model_version, candidate.candidate_parameter_version,
                   candidate.candidate_definition_sha256,
                   candidate.baseline_metrics, candidate.calibration_bias,
                   candidate.proposed_adjustments, candidate.constraints,
                   candidate.training_window, candidate.validation_window, candidate.holdout_window,
                   candidate.rationale, candidate.status, candidate.created_at,
                   candidate.decided_at, candidate.decision_note
            FROM analytics.parameter_tuning_candidates candidate
            LEFT JOIN football.competitions competition ON competition.id = candidate.competition_id
            ORDER BY candidate.created_at DESC
            LIMIT $1
            "#,
        )
        .bind(i64::from(limit.clamp(1, 500)))
        .fetch_all(&self.pool)
        .await?;
        rows.iter()
            .map(parameter_tuning_candidate_from_row)
            .collect()
    }

    pub async fn decide_parameter_tuning_candidate(
        &self,
        draft: &ParameterTuningDecisionDraft,
    ) -> PersistenceResult<ParameterTuningCandidateRecord> {
        let status = match draft.decision {
            ParameterTuningDecision::AcceptForBacktest => "accepted_for_backtest",
            ParameterTuningDecision::Reject => "rejected",
        };
        let updated = sqlx::query_scalar::<_, Uuid>(
            r#"
            UPDATE analytics.parameter_tuning_candidates
            SET status=$2, decided_at=now(), decision_note=$3
            WHERE id=$1 AND status='pending'
            RETURNING id
            "#,
        )
        .bind(draft.candidate_id)
        .bind(status)
        .bind(draft.decision_note.as_deref())
        .fetch_optional(&self.pool)
        .await?;
        if updated.is_none() {
            return Err(PersistenceError::InvalidState(
                "参数候选不存在或已经完成审核".to_string(),
            ));
        }
        self.read_parameter_tuning_candidate(draft.candidate_id)
            .await
    }

    pub async fn read_parameter_tuning_candidate(
        &self,
        id: Uuid,
    ) -> PersistenceResult<ParameterTuningCandidateRecord> {
        let row = sqlx::query(
            r#"
            SELECT candidate.id, candidate.competition_id, competition.name AS competition_name,
                   candidate.competition_profile_id, candidate.partition_key,
                   candidate.model_key, candidate.model_version, candidate.parameter_version,
                   candidate.snapshot_type, candidate.target_module, candidate.sample_size,
                   candidate.baseline_model_version_id, candidate.baseline_parameter_set_id,
                   candidate.candidate_model_version_id, candidate.candidate_parameter_set_id,
                   candidate.candidate_model_version, candidate.candidate_parameter_version,
                   candidate.candidate_definition_sha256,
                   candidate.baseline_metrics, candidate.calibration_bias,
                   candidate.proposed_adjustments, candidate.constraints,
                   candidate.training_window, candidate.validation_window, candidate.holdout_window,
                   candidate.rationale, candidate.status, candidate.created_at,
                   candidate.decided_at, candidate.decision_note
            FROM analytics.parameter_tuning_candidates candidate
            LEFT JOIN football.competitions competition ON competition.id = candidate.competition_id
            WHERE candidate.id=$1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("参数候选不存在".to_string()))?;
        parameter_tuning_candidate_from_row(&row)
    }

    async fn read_ai_suggestion(&self, id: Uuid) -> PersistenceResult<AiAnalysisSuggestionRecord> {
        let row = sqlx::query("SELECT id,response_id,suggestion_type,title,summary,severity,scope,payload,evidence,status,created_at,decided_at,decision_note,linked_candidate_id FROM analytics.ai_suggestions WHERE id=$1")
            .bind(id).fetch_optional(&self.pool).await?.ok_or_else(|| PersistenceError::InvalidState("AI 建议不存在".to_string()))?;
        ai_suggestion_from_row(&row)
    }
}

fn sample_group_ids(
    samples: &[EvaluationSample],
) -> HashMap<(String, String, String, String), (Uuid, Uuid)> {
    samples
        .iter()
        .map(|sample| {
            (
                (
                    sample.model_key.clone(),
                    sample.model_version.clone(),
                    sample.parameter_version.clone(),
                    sample.snapshot_type.clone(),
                ),
                (sample.model_version_id, sample.parameter_set_id),
            )
        })
        .collect()
}

async fn insert_calibration_bucket(
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

async fn insert_drift_finding(
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

fn evaluation_sample_from_row(row: &sqlx::postgres::PgRow) -> PersistenceResult<EvaluationSample> {
    Ok(EvaluationSample {
        review_id: row.try_get("review_id")?,
        run_id: row.try_get("run_id")?,
        model_version_id: row.try_get("model_version_id")?,
        parameter_set_id: row.try_get("parameter_set_id")?,
        model_key: row.try_get("model_key")?,
        model_version: row.try_get("model_version")?,
        parameter_version: row.try_get("parameter_version")?,
        competition_id: row.try_get("competition_id")?,
        competition_name: row.try_get("competition_name")?,
        season_id: row.try_get("season_id")?,
        stage_id: row.try_get("stage_id")?,
        snapshot_type: row.try_get("snapshot_type")?,
        kickoff_time: row.try_get("kickoff_time")?,
        actual_outcome: row.try_get("actual_outcome")?,
        home_win: row.try_get("home_win")?,
        draw: row.try_get("draw")?,
        away_win: row.try_get("away_win")?,
        log_loss: row.try_get("log_loss")?,
        brier: row.try_get("brier")?,
        scoreline_nll: row.try_get("scoreline_nll")?,
        data_coverage: row.try_get("data_coverage")?,
    })
}

fn parameter_tuning_candidate_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<ParameterTuningCandidateRecord> {
    let sample_size = row.try_get::<i64, _>("sample_size")?;
    let sample_size = u64::try_from(sample_size)
        .map_err(|_| PersistenceError::InvalidState("参数候选样本量不能为负数".to_string()))?;
    Ok(ParameterTuningCandidateRecord {
        id: row.try_get("id")?,
        competition_id: row.try_get("competition_id")?,
        competition_name: row.try_get("competition_name")?,
        competition_profile_id: row.try_get("competition_profile_id")?,
        partition_key: row.try_get("partition_key")?,
        model_key: row.try_get("model_key")?,
        model_version: row.try_get("model_version")?,
        parameter_version: row.try_get("parameter_version")?,
        snapshot_type: row.try_get("snapshot_type")?,
        target_module: row.try_get("target_module")?,
        sample_size,
        baseline_model_version_id: row.try_get("baseline_model_version_id")?,
        baseline_parameter_set_id: row.try_get("baseline_parameter_set_id")?,
        candidate_model_version_id: row.try_get("candidate_model_version_id")?,
        candidate_parameter_set_id: row.try_get("candidate_parameter_set_id")?,
        candidate_model_version: row.try_get("candidate_model_version")?,
        candidate_parameter_version: row.try_get("candidate_parameter_version")?,
        candidate_definition_sha256: row.try_get("candidate_definition_sha256")?,
        baseline_metrics: row.try_get("baseline_metrics")?,
        calibration_bias: row.try_get("calibration_bias")?,
        proposed_adjustments: row.try_get("proposed_adjustments")?,
        constraints: row.try_get("constraints")?,
        training_window: row.try_get("training_window")?,
        validation_window: row.try_get("validation_window")?,
        holdout_window: row.try_get("holdout_window")?,
        rationale: row.try_get("rationale")?,
        status: row.try_get("status")?,
        created_at: row.try_get("created_at")?,
        decided_at: row.try_get("decided_at")?,
        decision_note: row.try_get("decision_note")?,
    })
}

fn data_quality_finding_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<DataQualityFinding> {
    Ok(DataQualityFinding {
        id: row.try_get("id")?,
        scan_id: row.try_get("scan_id")?,
        severity: row.try_get("severity")?,
        category: row.try_get("category")?,
        finding_code: row.try_get("finding_code")?,
        entity_type: row.try_get("entity_type")?,
        entity_id: row.try_get("entity_id")?,
        message: row.try_get("message")?,
        evidence: row.try_get("evidence")?,
        status: row.try_get("status")?,
        detected_at: row.try_get("detected_at")?,
    })
}

fn ai_suggestion_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<AiAnalysisSuggestionRecord> {
    Ok(AiAnalysisSuggestionRecord {
        id: row.try_get("id")?,
        response_id: row.try_get("response_id")?,
        suggestion_type: row.try_get("suggestion_type")?,
        title: row.try_get("title")?,
        summary: row.try_get("summary")?,
        severity: row.try_get("severity")?,
        scope: row.try_get("scope")?,
        payload: row.try_get("payload")?,
        evidence: row.try_get("evidence")?,
        status: row.try_get("status")?,
        created_at: row.try_get("created_at")?,
        decided_at: row.try_get("decided_at")?,
        decision_note: row.try_get("decision_note")?,
        linked_candidate_id: row.try_get("linked_candidate_id")?,
    })
}

fn json_string(value: &Value, key: &str) -> PersistenceResult<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| PersistenceError::InvalidState(format!("AI 建议缺少 {key}")))
}
fn json_number(value: &Value, key: &str) -> PersistenceResult<f64> {
    value
        .get(key)
        .and_then(Value::as_f64)
        .ok_or_else(|| PersistenceError::InvalidState(format!("AI 建议缺少数值 {key}")))
}
fn json_uuid(value: &Value, key: &str) -> PersistenceResult<Uuid> {
    Uuid::parse_str(&json_string(value, key)?)
        .map_err(|error| PersistenceError::InvalidState(format!("AI 建议 {key} 无效：{error}")))
}
fn escape_copy_field(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}
