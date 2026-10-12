use super::{sha256_json, write_audit_event, PersistenceError, PersistenceResult, PostgresStore};
use football_analytics_engine::calculate_analytics;
use football_domain::{
    EvaluationSample, EvidenceScoringDecisionDraft, EvidenceScoringItemRecord, EvidenceVerdict,
    PostmatchDriftFindingRecord, PostmatchDriftRunRecord, PostmatchMonitoringRequest,
    PostmatchOverview, ProviderScoreSnapshotRecord, POSTMATCH_MONITORING_VERSION,
};
use serde_json::{json, Value};
use sqlx::Row;
use std::collections::BTreeMap;
use uuid::Uuid;

pub(crate) const FORMAL_HORIZONS: [&str; 4] = ["T-N", "T-24h", "T-6h", "T-1h"];

impl PostgresStore {
    pub async fn list_evidence_scoring_items(
        &self,
        status: Option<&str>,
        limit: u32,
    ) -> PersistenceResult<Vec<EvidenceScoringItemRecord>> {
        if let Some(value) = status {
            if !matches!(value, "pending" | "scored" | "not_verifiable") {
                return Err(PersistenceError::InvalidState(
                    "证据评分状态必须为 pending、scored 或 not_verifiable".to_string(),
                ));
            }
        }
        let rows = sqlx::query(
            r#"
            SELECT item.id, item.settlement_id, item.evidence_id, item.provider_id,
                   provider.name AS provider_name, item.source_document_id, item.field_key,
                   item.verification_state, item.source_tier, claim.source_title,
                   claim.source_domain, item.published_at, item.retrieved_at,
                   item.data_cutoff_at, item.timeliness_score, item.created_at,
                   decision.id AS decision_id, decision.verdict, decision.accuracy_score,
                   decision.reliability_score, decision.decided_by, decision.decision_note,
                   decision.decided_at,
                   CASE
                       WHEN decision.id IS NULL THEN 'pending'
                       WHEN decision.verdict = 'not_verifiable' THEN 'not_verifiable'
                       ELSE 'scored'
                   END AS status
            FROM review.evidence_scoring_items item
            JOIN research.evidence_claims claim ON claim.id = item.evidence_id
            LEFT JOIN catalog.data_providers provider ON provider.id = item.provider_id
            LEFT JOIN review.evidence_scoring_decisions decision ON decision.item_id = item.id
            WHERE $1::text IS NULL OR
                  ($1 = 'pending' AND decision.id IS NULL) OR
                  ($1 = 'scored' AND decision.id IS NOT NULL AND decision.verdict <> 'not_verifiable') OR
                  ($1 = 'not_verifiable' AND decision.verdict = 'not_verifiable')
            ORDER BY CASE WHEN decision.id IS NULL THEN 0 ELSE 1 END,
                     item.created_at DESC, item.id DESC
            LIMIT $2
            "#,
        )
        .bind(status)
        .bind(i64::from(limit.clamp(1, 1000)))
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(evidence_item_from_row).collect()
    }

    pub async fn decide_evidence_scoring_item(
        &self,
        draft: &EvidenceScoringDecisionDraft,
    ) -> PersistenceResult<EvidenceScoringItemRecord> {
        if draft.decision_note.trim().len() < 8 {
            return Err(PersistenceError::InvalidState(
                "证据判定说明至少需要 8 个字符".to_string(),
            ));
        }
        let item = sqlx::query(
            r#"
            SELECT item.id, item.verification_state, item.source_tier,
                   EXISTS (SELECT 1 FROM review.evidence_scoring_decisions decision WHERE decision.item_id=item.id) AS decided
            FROM review.evidence_scoring_items item
            WHERE item.id=$1
            "#,
        )
        .bind(draft.item_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("证据评分项不存在".to_string()))?;
        if item.try_get::<bool, _>("decided")? {
            return Err(PersistenceError::InvalidState(
                "该证据评分项已经完成判定，历史决定不可覆盖".to_string(),
            ));
        }
        let verification_state: String = item.try_get("verification_state")?;
        let source_tier: String = item.try_get("source_tier")?;
        let (accuracy_score, reliability_score) =
            verdict_scores(draft.verdict, &verification_state, &source_tier);
        let decision_fingerprint = sha256_json(&json!({
            "item_id": draft.item_id,
            "verdict": draft.verdict.as_str(),
            "accuracy_score": accuracy_score,
            "reliability_score": reliability_score,
            "decided_by": draft.decided_by.as_deref(),
            "decision_note": draft.decision_note.trim(),
        }))?;
        let decision_id = Uuid::new_v4();
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            r#"
            INSERT INTO review.evidence_scoring_decisions (
                id, item_id, verdict, accuracy_score, reliability_score,
                decided_by, decision_note, decision_fingerprint
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
            "#,
        )
        .bind(decision_id)
        .bind(draft.item_id)
        .bind(draft.verdict.as_str())
        .bind(accuracy_score)
        .bind(reliability_score)
        .bind(draft.decided_by.as_deref())
        .bind(draft.decision_note.trim())
        .bind(&decision_fingerprint)
        .execute(&mut *tx)
        .await?;
        write_audit_event(
            &mut tx,
            "evidence_scoring_decided",
            "evidence_scoring_item",
            Some(draft.item_id.to_string()),
            json!({
                "decision_id": decision_id,
                "verdict": draft.verdict.as_str(),
                "accuracy_score": accuracy_score,
                "reliability_score": reliability_score,
            }),
        )
        .await?;
        tx.commit().await?;
        self.read_evidence_scoring_item(draft.item_id).await
    }

    pub async fn refresh_postmatch_monitoring(
        &self,
        request: &PostmatchMonitoringRequest,
    ) -> PersistenceResult<PostmatchOverview> {
        validate_horizon(&request.horizon)?;
        if request.baseline_size < 5 || request.current_size < 5 {
            return Err(PersistenceError::InvalidState(
                "漂移基线窗口和当前窗口都不能少于 5 场".to_string(),
            ));
        }
        let partition = self
            .active_postmatch_partition(request.competition_id, &request.horizon)
            .await?;
        self.refresh_provider_scores_for_partition(&partition)
            .await?;
        self.refresh_drift_for_partition(&partition, request.baseline_size, request.current_size)
            .await?;
        self.postmatch_overview(100).await
    }

    pub async fn postmatch_overview(&self, limit: u32) -> PersistenceResult<PostmatchOverview> {
        let settlements = self.list_postmatch_settlements(limit).await?;
        let evidence_queue = self.list_evidence_scoring_items(None, limit).await?;
        let provider_scores = self.list_provider_scores(limit).await?;
        let drift_runs = self.list_postmatch_drift_runs(limit).await?;
        let settlement_count = non_negative_u64(
            sqlx::query_scalar::<_, i64>(
                "SELECT count(*)::bigint FROM review.postmatch_settlements",
            )
            .fetch_one(&self.pool)
            .await?,
            "赛后结算数量",
        )?;
        let pending_evidence_count = non_negative_u64(
            sqlx::query_scalar::<_, i64>(
                "SELECT count(*)::bigint FROM review.evidence_scoring_items item WHERE NOT EXISTS (SELECT 1 FROM review.evidence_scoring_decisions decision WHERE decision.item_id=item.id)",
            )
            .fetch_one(&self.pool)
            .await?,
            "待评分证据数量",
        )?;
        let scored_evidence_count = non_negative_u64(
            sqlx::query_scalar::<_, i64>(
                "SELECT count(*)::bigint FROM review.evidence_scoring_decisions WHERE verdict <> 'not_verifiable'",
            )
            .fetch_one(&self.pool)
            .await?,
            "已评分证据数量",
        )?;
        Ok(PostmatchOverview {
            settlement_count,
            pending_evidence_count,
            scored_evidence_count,
            settlements,
            evidence_queue,
            provider_scores,
            drift_runs,
        })
    }

    async fn read_evidence_scoring_item(
        &self,
        item_id: Uuid,
    ) -> PersistenceResult<EvidenceScoringItemRecord> {
        let row = sqlx::query(
            r#"
            SELECT item.id, item.settlement_id, item.evidence_id, item.provider_id,
                   provider.name AS provider_name, item.source_document_id, item.field_key,
                   item.verification_state, item.source_tier, claim.source_title,
                   claim.source_domain, item.published_at, item.retrieved_at,
                   item.data_cutoff_at, item.timeliness_score, item.created_at,
                   decision.id AS decision_id, decision.verdict, decision.accuracy_score,
                   decision.reliability_score, decision.decided_by, decision.decision_note,
                   decision.decided_at,
                   CASE
                       WHEN decision.id IS NULL THEN 'pending'
                       WHEN decision.verdict = 'not_verifiable' THEN 'not_verifiable'
                       ELSE 'scored'
                   END AS status
            FROM review.evidence_scoring_items item
            JOIN research.evidence_claims claim ON claim.id = item.evidence_id
            LEFT JOIN catalog.data_providers provider ON provider.id = item.provider_id
            LEFT JOIN review.evidence_scoring_decisions decision ON decision.item_id = item.id
            WHERE item.id=$1
            "#,
        )
        .bind(item_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("证据评分项不存在".to_string()))?;
        evidence_item_from_row(&row)
    }

    async fn active_postmatch_partition(
        &self,
        competition_id: Uuid,
        horizon: &str,
    ) -> PersistenceResult<PostmatchPartition> {
        let row = sqlx::query(
            r#"
            SELECT competition.id AS competition_id, package.competition_profile_id,
                   binding.model_version_id, binding.parameter_set_id
            FROM model.competition_bindings binding
            JOIN football.competitions competition ON competition.id=binding.competition_id
            JOIN model.rule_packages package ON package.id=binding.rule_package_id
            WHERE binding.competition_id=$1 AND binding.is_active=true
              AND package.status='active'
            ORDER BY binding.priority DESC, binding.id DESC
            LIMIT 1
            "#,
        )
        .bind(competition_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("未找到该赛事的活动模型绑定".to_string()))?;
        let model_version_id: Uuid = row.try_get("model_version_id")?;
        let competition_profile_id: Uuid = row.try_get("competition_profile_id")?;
        let parameter_set_id: Uuid = row.try_get("parameter_set_id")?;
        Ok(PostmatchPartition {
            competition_id: row.try_get("competition_id")?,
            competition_profile_id,
            model_version_id,
            parameter_set_id,
            horizon: horizon.to_string(),
            partition_key: format!(
                "{model_version_id}:{competition_profile_id}:{parameter_set_id}:{horizon}"
            ),
        })
    }

    async fn refresh_provider_scores_for_partition(
        &self,
        partition: &PostmatchPartition,
    ) -> PersistenceResult<()> {
        let rows = sqlx::query(
            r#"
            SELECT item.provider_id, decision.id AS decision_id, decision.verdict,
                   decision.accuracy_score, decision.reliability_score,
                   item.timeliness_score, decision.decision_fingerprint
            FROM review.evidence_scoring_items item
            JOIN review.postmatch_settlements settlement ON settlement.id=item.settlement_id
            JOIN review.evidence_scoring_decisions decision ON decision.item_id=item.id
            WHERE settlement.competition_id=$1
              AND settlement.competition_profile_id=$2
              AND settlement.model_version_id=$3
              AND settlement.parameter_set_id=$4
              AND settlement.horizon=$5
              AND item.provider_id IS NOT NULL
            ORDER BY item.provider_id, decision.id
            "#,
        )
        .bind(partition.competition_id)
        .bind(partition.competition_profile_id)
        .bind(partition.model_version_id)
        .bind(partition.parameter_set_id)
        .bind(&partition.horizon)
        .fetch_all(&self.pool)
        .await?;
        let mut groups: BTreeMap<Uuid, ProviderAggregate> = BTreeMap::new();
        for row in rows {
            let provider_id: Uuid = row.try_get("provider_id")?;
            let entry = groups.entry(provider_id).or_default();
            let verdict: String = row.try_get("verdict")?;
            match verdict.as_str() {
                "correct" => entry.correct += 1,
                "partial" => entry.partial += 1,
                "incorrect" => entry.incorrect += 1,
                _ => entry.not_verifiable += 1,
            }
            if let Some(value) = row.try_get::<Option<f64>, _>("accuracy_score")? {
                entry.accuracy.push(value);
            }
            if let Some(value) = row.try_get::<Option<f64>, _>("reliability_score")? {
                entry.reliability.push(value);
            }
            entry.timeliness.push(row.try_get("timeliness_score")?);
            entry
                .decision_fingerprints
                .push(row.try_get("decision_fingerprint")?);
        }
        for (provider_id, aggregate) in groups {
            let accuracy_mean = mean(&aggregate.accuracy).unwrap_or(0.0);
            let timeliness_mean = mean(&aggregate.timeliness).unwrap_or(0.0);
            let reliability_mean = mean(&aggregate.reliability).unwrap_or(0.0);
            let weighted_score = if aggregate.accuracy.is_empty() {
                0.0
            } else {
                round6(accuracy_mean * 0.60 + timeliness_mean * 0.15 + reliability_mean * 0.25)
            };
            let decision_set_sha256 = sha256_json(&aggregate.decision_fingerprints)?;
            let snapshot_key = format!(
                "{}:{}:{}:{}",
                provider_id,
                partition.partition_key,
                decision_set_sha256,
                POSTMATCH_MONITORING_VERSION
            );
            sqlx::query(
                r#"
                INSERT INTO analytics.provider_score_snapshots (
                    id, provider_id, scope_key, competition_id, competition_profile_id,
                    model_version_id, parameter_set_id, horizon, sample_size,
                    correct_count, partial_count, incorrect_count, not_verifiable_count,
                    accuracy_mean, timeliness_mean, reliability_mean, weighted_score,
                    decision_set_sha256, snapshot_key, calculation_version
                ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20)
                ON CONFLICT (snapshot_key) DO NOTHING
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(provider_id)
            .bind(&partition.partition_key)
            .bind(partition.competition_id)
            .bind(partition.competition_profile_id)
            .bind(partition.model_version_id)
            .bind(partition.parameter_set_id)
            .bind(&partition.horizon)
            .bind(aggregate.total() as i64)
            .bind(aggregate.correct as i64)
            .bind(aggregate.partial as i64)
            .bind(aggregate.incorrect as i64)
            .bind(aggregate.not_verifiable as i64)
            .bind(round6(accuracy_mean))
            .bind(round6(timeliness_mean))
            .bind(round6(reliability_mean))
            .bind(weighted_score)
            .bind(&decision_set_sha256)
            .bind(snapshot_key)
            .bind(POSTMATCH_MONITORING_VERSION)
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    async fn refresh_drift_for_partition(
        &self,
        partition: &PostmatchPartition,
        baseline_size: usize,
        current_size: usize,
    ) -> PersistenceResult<()> {
        let rows = sqlx::query(
            r#"
            SELECT sample.review_id, sample.run_id, sample.model_version_id,
                   sample.parameter_set_id, definition.model_key,
                   version.version AS model_version, parameter.parameter_version,
                   sample.competition_id, competition.name AS competition_name,
                   sample.season_id, sample.stage_id, sample.snapshot_type,
                   sample.kickoff_time, sample.actual_outcome, sample.home_win,
                   sample.draw, sample.away_win, sample.log_loss, sample.brier,
                   sample.scoreline_nll, sample.data_coverage, settlement.id AS settlement_id
            FROM analytics.evaluation_samples sample
            JOIN review.postmatch_settlements settlement ON settlement.id=sample.settlement_id
            JOIN model.versions version ON version.id=sample.model_version_id
            JOIN model.definitions definition ON definition.id=version.model_id
            JOIN model.parameter_sets parameter ON parameter.id=sample.parameter_set_id
            LEFT JOIN football.competitions competition ON competition.id=sample.competition_id
            WHERE sample.competition_id=$1
              AND sample.competition_profile_id=$2
              AND sample.model_version_id=$3
              AND sample.parameter_set_id=$4
              AND sample.snapshot_type=$5
              AND sample.calculation_version=$6
            ORDER BY sample.kickoff_time, sample.run_id
            "#,
        )
        .bind(partition.competition_id)
        .bind(partition.competition_profile_id)
        .bind(partition.model_version_id)
        .bind(partition.parameter_set_id)
        .bind(&partition.horizon)
        .bind(POSTMATCH_MONITORING_VERSION)
        .fetch_all(&self.pool)
        .await?;
        let mut samples = Vec::with_capacity(rows.len());
        let mut settlement_ids = Vec::with_capacity(rows.len());
        for row in rows {
            settlement_ids.push(row.try_get::<Uuid, _>("settlement_id")?);
            samples.push(EvaluationSample {
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
            });
        }
        let calculation = calculate_analytics(&samples, 10, baseline_size, current_size);
        let enough = samples.len() >= baseline_size + current_size;
        let status = if !enough {
            "insufficient"
        } else if calculation
            .drift
            .iter()
            .any(|item| item.severity == "critical")
        {
            "critical"
        } else if calculation
            .drift
            .iter()
            .any(|item| item.severity == "warning")
        {
            "warning"
        } else {
            "stable"
        };
        let current_start = samples.len().saturating_sub(current_size);
        let baseline_start = current_start.saturating_sub(baseline_size);
        let baseline_slice = &samples[baseline_start..current_start];
        let current_slice = &samples[current_start..];
        let baseline_window = window_json(baseline_slice);
        let current_window = window_json(current_slice);
        let run_payload = json!({
            "partition_key": &partition.partition_key,
            "baseline_size": baseline_size,
            "current_size": current_size,
            "settlement_ids": settlement_ids,
            "calculation_version": POSTMATCH_MONITORING_VERSION,
        });
        let run_key = sha256_json(&run_payload)?;
        let run_id = Uuid::new_v4();
        let mut tx = self.pool.begin().await?;
        let inserted = sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO analytics.postmatch_drift_runs (
                id, competition_id, competition_profile_id, model_version_id,
                parameter_set_id, horizon, partition_key, baseline_size,
                current_size, baseline_window, current_window, status,
                run_key, calculation_version
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14)
            ON CONFLICT (run_key) DO NOTHING
            RETURNING id
            "#,
        )
        .bind(run_id)
        .bind(partition.competition_id)
        .bind(partition.competition_profile_id)
        .bind(partition.model_version_id)
        .bind(partition.parameter_set_id)
        .bind(&partition.horizon)
        .bind(&partition.partition_key)
        .bind(baseline_slice.len() as i64)
        .bind(current_slice.len() as i64)
        .bind(&baseline_window)
        .bind(&current_window)
        .bind(status)
        .bind(&run_key)
        .bind(POSTMATCH_MONITORING_VERSION)
        .fetch_optional(&mut *tx)
        .await?;
        if let Some(id) = inserted {
            for finding in &calculation.drift {
                sqlx::query(
                    r#"
                    INSERT INTO analytics.postmatch_drift_findings (
                        run_id, metric_name, baseline_mean, current_mean,
                        absolute_delta, relative_delta, severity, direction
                    ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
                    "#,
                )
                .bind(id)
                .bind(&finding.metric_name)
                .bind(finding.baseline_mean)
                .bind(finding.current_mean)
                .bind(finding.absolute_delta)
                .bind(finding.relative_delta)
                .bind(&finding.severity)
                .bind(&finding.direction)
                .execute(&mut *tx)
                .await?;
            }
            write_audit_event(
                &mut tx,
                "postmatch_drift_refreshed",
                "postmatch_drift_run",
                Some(id.to_string()),
                json!({
                    "partition_key": &partition.partition_key,
                    "status": status,
                    "baseline_size": baseline_slice.len(),
                    "current_size": current_slice.len(),
                }),
            )
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    async fn list_provider_scores(
        &self,
        limit: u32,
    ) -> PersistenceResult<Vec<ProviderScoreSnapshotRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT DISTINCT ON (score.provider_id, score.scope_key)
                   score.id, score.provider_id, provider.name AS provider_name,
                   score.scope_key, score.competition_id, score.competition_profile_id,
                   score.model_version_id, score.parameter_set_id, score.horizon,
                   score.sample_size, score.correct_count,
                   score.partial_count, score.incorrect_count, score.not_verifiable_count,
                   score.accuracy_mean, score.timeliness_mean, score.reliability_mean,
                   score.weighted_score, score.decision_set_sha256,
                   score.calculation_version, score.generated_at
            FROM analytics.provider_score_snapshots score
            JOIN catalog.data_providers provider ON provider.id=score.provider_id
            ORDER BY score.provider_id, score.scope_key, score.generated_at DESC, score.id DESC
            LIMIT $1
            "#,
        )
        .bind(i64::from(limit.clamp(1, 500)))
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(provider_score_from_row).collect()
    }

    async fn list_postmatch_drift_runs(
        &self,
        limit: u32,
    ) -> PersistenceResult<Vec<PostmatchDriftRunRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT run.id, run.competition_id, competition.name AS competition_name,
                   run.competition_profile_id, run.model_version_id,
                   version.version AS model_version, run.parameter_set_id,
                   parameter.parameter_version, run.horizon, run.partition_key,
                   run.baseline_size, run.current_size, run.baseline_window,
                   run.current_window, run.status, run.run_key,
                   run.calculation_version, run.generated_at
            FROM analytics.postmatch_drift_runs run
            JOIN football.competitions competition ON competition.id=run.competition_id
            JOIN model.versions version ON version.id=run.model_version_id
            JOIN model.parameter_sets parameter ON parameter.id=run.parameter_set_id
            ORDER BY run.generated_at DESC, run.id DESC
            LIMIT $1
            "#,
        )
        .bind(i64::from(limit.clamp(1, 500)))
        .fetch_all(&self.pool)
        .await?;
        let mut output = Vec::with_capacity(rows.len());
        for row in rows {
            let run_id: Uuid = row.try_get("id")?;
            let finding_rows = sqlx::query(
                "SELECT metric_name,baseline_mean,current_mean,absolute_delta,relative_delta,severity,direction FROM analytics.postmatch_drift_findings WHERE run_id=$1 ORDER BY metric_name",
            )
            .bind(run_id)
            .fetch_all(&self.pool)
            .await?;
            let findings = finding_rows
                .iter()
                .map(drift_finding_from_row)
                .collect::<PersistenceResult<Vec<_>>>()?;
            output.push(PostmatchDriftRunRecord {
                id: run_id,
                competition_id: row.try_get("competition_id")?,
                competition_name: row.try_get("competition_name")?,
                competition_profile_id: row.try_get("competition_profile_id")?,
                model_version_id: row.try_get("model_version_id")?,
                model_version: row.try_get("model_version")?,
                parameter_set_id: row.try_get("parameter_set_id")?,
                parameter_version: row.try_get("parameter_version")?,
                horizon: row.try_get("horizon")?,
                partition_key: row.try_get("partition_key")?,
                baseline_size: non_negative_u64(row.try_get("baseline_size")?, "漂移基线样本")?,
                current_size: non_negative_u64(row.try_get("current_size")?, "漂移当前样本")?,
                baseline_window: row.try_get("baseline_window")?,
                current_window: row.try_get("current_window")?,
                status: row.try_get("status")?,
                run_key: row.try_get("run_key")?,
                calculation_version: row.try_get("calculation_version")?,
                findings,
                generated_at: row.try_get("generated_at")?,
            });
        }
        Ok(output)
    }
}

#[derive(Debug)]
struct PostmatchPartition {
    competition_id: Uuid,
    competition_profile_id: Uuid,
    model_version_id: Uuid,
    parameter_set_id: Uuid,
    horizon: String,
    partition_key: String,
}

#[derive(Debug, Default)]
struct ProviderAggregate {
    correct: usize,
    partial: usize,
    incorrect: usize,
    not_verifiable: usize,
    accuracy: Vec<f64>,
    timeliness: Vec<f64>,
    reliability: Vec<f64>,
    decision_fingerprints: Vec<String>,
}

impl ProviderAggregate {
    fn total(&self) -> usize {
        self.correct + self.partial + self.incorrect + self.not_verifiable
    }
}

fn evidence_item_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<EvidenceScoringItemRecord> {
    Ok(EvidenceScoringItemRecord {
        id: row.try_get("id")?,
        settlement_id: row.try_get("settlement_id")?,
        evidence_id: row.try_get("evidence_id")?,
        provider_id: row.try_get("provider_id")?,
        provider_name: row.try_get("provider_name")?,
        source_document_id: row.try_get("source_document_id")?,
        field_key: row.try_get("field_key")?,
        verification_state: row.try_get("verification_state")?,
        source_tier: row.try_get("source_tier")?,
        source_title: row.try_get("source_title")?,
        source_domain: row.try_get("source_domain")?,
        published_at: row.try_get("published_at")?,
        retrieved_at: row.try_get("retrieved_at")?,
        data_cutoff_at: row.try_get("data_cutoff_at")?,
        timeliness_score: row.try_get("timeliness_score")?,
        decision_id: row.try_get("decision_id")?,
        verdict: row.try_get("verdict")?,
        accuracy_score: row.try_get("accuracy_score")?,
        reliability_score: row.try_get("reliability_score")?,
        decided_by: row.try_get("decided_by")?,
        decision_note: row.try_get("decision_note")?,
        decided_at: row.try_get("decided_at")?,
        status: row.try_get("status")?,
        created_at: row.try_get("created_at")?,
    })
}

fn provider_score_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<ProviderScoreSnapshotRecord> {
    Ok(ProviderScoreSnapshotRecord {
        id: row.try_get("id")?,
        provider_id: row.try_get("provider_id")?,
        provider_name: row.try_get("provider_name")?,
        scope_key: row.try_get("scope_key")?,
        competition_id: row.try_get("competition_id")?,
        competition_profile_id: row.try_get("competition_profile_id")?,
        model_version_id: row.try_get("model_version_id")?,
        parameter_set_id: row.try_get("parameter_set_id")?,
        horizon: row.try_get("horizon")?,
        sample_size: non_negative_u64(row.try_get("sample_size")?, "供应商样本")?,
        correct_count: non_negative_u64(row.try_get("correct_count")?, "正确证据数量")?,
        partial_count: non_negative_u64(row.try_get("partial_count")?, "部分正确证据数量")?,
        incorrect_count: non_negative_u64(row.try_get("incorrect_count")?, "错误证据数量")?,
        not_verifiable_count: non_negative_u64(
            row.try_get("not_verifiable_count")?,
            "不可验证证据数量",
        )?,
        accuracy_mean: row.try_get("accuracy_mean")?,
        timeliness_mean: row.try_get("timeliness_mean")?,
        reliability_mean: row.try_get("reliability_mean")?,
        weighted_score: row.try_get("weighted_score")?,
        decision_set_sha256: row.try_get("decision_set_sha256")?,
        calculation_version: row.try_get("calculation_version")?,
        generated_at: row.try_get("generated_at")?,
    })
}

fn drift_finding_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<PostmatchDriftFindingRecord> {
    Ok(PostmatchDriftFindingRecord {
        metric_name: row.try_get("metric_name")?,
        baseline_mean: row.try_get("baseline_mean")?,
        current_mean: row.try_get("current_mean")?,
        absolute_delta: row.try_get("absolute_delta")?,
        relative_delta: row.try_get("relative_delta")?,
        severity: row.try_get("severity")?,
        direction: row.try_get("direction")?,
    })
}

fn verdict_scores(
    verdict: EvidenceVerdict,
    verification_state: &str,
    source_tier: &str,
) -> (Option<f64>, Option<f64>) {
    if verdict == EvidenceVerdict::NotVerifiable {
        return (None, None);
    }
    let accuracy: f64 = match verdict {
        EvidenceVerdict::Correct => 1.0,
        EvidenceVerdict::Partial => 0.5,
        EvidenceVerdict::Incorrect => 0.0,
        EvidenceVerdict::NotVerifiable => unreachable!(),
    };
    let verification_quality: f64 = match verification_state {
        "CONFIRMED" => 1.0,
        "PROBABLE" => 0.8,
        "CONFLICT" => 0.35,
        "STALE" => 0.25,
        "NOT_FOUND" | "NOT_APPLICABLE" => 0.5,
        _ => 0.5,
    };
    let source_quality: f64 = match source_tier.to_ascii_lowercase().as_str() {
        "official" | "tier_1" | "tier1" => 1.0,
        "reliable" | "tier_2" | "tier2" => 0.8,
        "secondary" | "tier_3" | "tier3" => 0.6,
        _ => 0.5,
    };
    let reliability =
        (accuracy * 0.70 + verification_quality * 0.15 + source_quality * 0.15).clamp(0.0, 1.0);
    (Some(round6(accuracy)), Some(round6(reliability)))
}

fn window_json(samples: &[EvaluationSample]) -> Value {
    json!({
        "sample_size": samples.len(),
        "start": samples.first().map(|item| item.kickoff_time),
        "end": samples.last().map(|item| item.kickoff_time),
        "run_ids": samples.iter().map(|item| item.run_id).collect::<Vec<_>>(),
    })
}

fn validate_horizon(value: &str) -> PersistenceResult<()> {
    if FORMAL_HORIZONS.contains(&value) {
        Ok(())
    } else {
        Err(PersistenceError::InvalidState(
            "正式赛后监控只支持 T-N、T-24h、T-6h、T-1h".to_string(),
        ))
    }
}

pub(crate) fn non_negative_u64(value: i64, label: &str) -> PersistenceResult<u64> {
    u64::try_from(value).map_err(|_| PersistenceError::InvalidState(format!("{label}不能为负数")))
}

fn mean(values: &[f64]) -> Option<f64> {
    (!values.is_empty()).then(|| values.iter().sum::<f64>() / values.len() as f64)
}

pub(crate) fn round6(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_verifiable_does_not_fake_accuracy() {
        assert_eq!(
            verdict_scores(EvidenceVerdict::NotVerifiable, "CONFIRMED", "official"),
            (None, None)
        );
        let (accuracy, reliability) =
            verdict_scores(EvidenceVerdict::Correct, "CONFIRMED", "official");
        assert_eq!(accuracy, Some(1.0));
        assert_eq!(reliability, Some(1.0));
    }
}
