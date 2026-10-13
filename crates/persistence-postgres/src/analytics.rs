use super::{write_audit_event, PersistenceError, PersistenceResult, PostgresStore};
use chrono::Utc;
use football_domain::{
    DataQualityDecisionDraft, DataQualityFinding, DataQualitySummary,
    ParameterTuningCandidateRecord, ParameterTuningDecision, ParameterTuningDecisionDraft,
    QueryPerformanceFinding, QueryPerformanceSummary,
};
use serde_json::{json, Value};
use sqlx::postgres::PgPoolCopyExt;
use sqlx::Row;
use std::time::Instant;
use uuid::Uuid;

impl PostgresStore {
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

    pub(crate) async fn latest_data_quality_summary(
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

    pub(crate) async fn latest_query_performance(
        &self,
    ) -> PersistenceResult<QueryPerformanceSummary> {
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

fn escape_copy_field(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}
