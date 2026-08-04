use super::{sha256_json, write_audit_event, PersistenceError, PersistenceResult, PostgresStore};
use chrono::{DateTime, Utc};
use football_analytics_engine::calculate_analytics;
use football_domain::{
    EvidenceScoringDecisionDraft, EvidenceScoringItemRecord, EvidenceVerdict, EvaluationSample,
    PostmatchDriftFindingRecord, PostmatchDriftRunRecord, PostmatchMonitoringRequest,
    PostmatchOverview, PostmatchSettlementDraft, PostmatchSettlementReadiness,
    PostmatchSettlementRecord, ProviderScoreSnapshotRecord, POSTMATCH_MONITORING_VERSION,
    POSTMATCH_SETTLEMENT_VERSION,
};
use serde::Serialize;
use serde_json::{json, Value};
use sqlx::{Postgres, Row, Transaction};
use std::collections::BTreeMap;
use uuid::Uuid;

const FORMAL_HORIZONS: [&str; 4] = ["T-N", "T-24h", "T-6h", "T-1h"];

#[derive(Debug)]
struct SettlementContext {
    match_review_id: Uuid,
    match_id: Uuid,
    match_key: String,
    home_team_name: String,
    away_team_name: String,
    competition_id: Option<Uuid>,
    review_status: String,
    data_coverage: f64,
    prediction_evaluation: Value,
    model_run_id: Option<Uuid>,
    run_status: Option<String>,
    feature_snapshot_id: Option<Uuid>,
    competition_profile_id: Option<Uuid>,
    model_version_id: Option<Uuid>,
    parameter_set_id: Option<Uuid>,
    rule_package_id: Option<Uuid>,
    horizon: Option<String>,
    kickoff_time: DateTime<Utc>,
    home_goals_90: Option<i16>,
    away_goals_90: Option<i16>,
    result_finalized_at: Option<DateTime<Utc>>,
    snapshot_cutoff_at: Option<DateTime<Utc>>,
    snapshot_match_id: Option<Uuid>,
    snapshot_model_version_id: Option<Uuid>,
    snapshot_parameter_set_id: Option<Uuid>,
    snapshot_competition_profile_id: Option<Uuid>,
    snapshot_source_kind: Option<String>,
    snapshot_evidence_scope: Option<String>,
    existing_settlement_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
struct SettlementFingerprint<'a> {
    match_id: Uuid,
    match_review_id: Uuid,
    model_run_id: Uuid,
    feature_snapshot_id: Uuid,
    competition_profile_id: Uuid,
    model_version_id: Uuid,
    parameter_set_id: Uuid,
    rule_package_id: Uuid,
    horizon: &'a str,
    home_goals_90: i16,
    away_goals_90: i16,
    result_finalized_at: DateTime<Utc>,
    prediction_evaluation: &'a Value,
}

impl PostgresStore {
    pub async fn postmatch_settlement_readiness(
        &self,
        match_review_id: Uuid,
    ) -> PersistenceResult<PostmatchSettlementReadiness> {
        let context = self.load_settlement_context(match_review_id).await?;
        Ok(readiness_from_context(&context))
    }

    pub async fn settle_postmatch_review(
        &self,
        draft: &PostmatchSettlementDraft,
    ) -> PersistenceResult<PostmatchSettlementRecord> {
        let context = self.load_settlement_context(draft.match_review_id).await?;
        let readiness = readiness_from_context(&context);
        if !readiness.ready {
            return Err(PersistenceError::InvalidState(format!(
                "赛后结算门禁未通过：{}",
                readiness.blocked_reasons.join("；")
            )));
        }
        if let Some(existing_id) = context.existing_settlement_id {
            return self.read_postmatch_settlement(existing_id).await;
        }

        let competition_id = required(context.competition_id, "比赛未绑定赛事")?;
        let competition_profile_id = required(
            context.competition_profile_id,
            "推演规则包未绑定赛事 Profile",
        )?;
        let model_run_id = required(context.model_run_id, "复盘未绑定成功推演")?;
        let feature_snapshot_id = required(context.feature_snapshot_id, "推演未绑定冻结快照")?;
        let model_version_id = required(context.model_version_id, "推演模型版本缺失")?;
        let parameter_set_id = required(context.parameter_set_id, "推演参数版本缺失")?;
        let rule_package_id = required(context.rule_package_id, "推演规则包缺失")?;
        let horizon = required(context.horizon.clone(), "推演冻结时点缺失")?;
        let home_goals_90 = required(context.home_goals_90, "正式赛果缺失")?;
        let away_goals_90 = required(context.away_goals_90, "正式赛果缺失")?;
        let result_finalized_at = required(context.result_finalized_at.clone(), "正式赛果时间缺失")?;
        let fingerprint = sha256_json(&SettlementFingerprint {
            match_id: context.match_id,
            match_review_id: context.match_review_id,
            model_run_id,
            feature_snapshot_id,
            competition_profile_id,
            model_version_id,
            parameter_set_id,
            rule_package_id,
            horizon: &horizon,
            home_goals_90,
            away_goals_90,
            result_finalized_at: result_finalized_at.clone(),
            prediction_evaluation: &context.prediction_evaluation,
        })?;
        let settlement_key = format!(
            "{}:{}:{}",
            context.match_review_id, model_run_id, POSTMATCH_SETTLEMENT_VERSION
        );
        let settlement_id = Uuid::new_v4();
        let metadata = json!({
            "formal_partition": format!(
                "{}:{}:{}:{}",
                model_version_id, competition_profile_id, parameter_set_id, horizon
            ),
            "result_finalized_at": &result_finalized_at,
            "kickoff_time": &context.kickoff_time,
            "data_coverage": context.data_coverage,
            "provider_state": "NOT_BUNDLED",
            "automatic_parameter_promotion": false,
        });

        let mut tx = self.pool.begin().await?;
        let inserted = sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO review.postmatch_settlements (
                id, match_id, match_review_id, model_run_id, feature_snapshot_id,
                competition_id, competition_profile_id, model_version_id,
                parameter_set_id, rule_package_id, horizon, home_goals_90,
                away_goals_90, result_finalized_at, result_fingerprint,
                settlement_key, settlement_version, settled_by, settlement_note, metadata
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20)
            ON CONFLICT (settlement_key) DO NOTHING
            RETURNING id
            "#,
        )
        .bind(settlement_id)
        .bind(context.match_id)
        .bind(context.match_review_id)
        .bind(model_run_id)
        .bind(feature_snapshot_id)
        .bind(competition_id)
        .bind(competition_profile_id)
        .bind(model_version_id)
        .bind(parameter_set_id)
        .bind(rule_package_id)
        .bind(&horizon)
        .bind(home_goals_90)
        .bind(away_goals_90)
        .bind(&result_finalized_at)
        .bind(&fingerprint)
        .bind(&settlement_key)
        .bind(POSTMATCH_SETTLEMENT_VERSION)
        .bind(draft.settled_by.as_deref())
        .bind(draft.settlement_note.as_deref())
        .bind(&metadata)
        .fetch_optional(&mut *tx)
        .await?;

        let persisted_id = if let Some(id) = inserted {
            self.insert_evidence_scoring_items_in_tx(
                &mut tx,
                id,
                feature_snapshot_id,
                required(context.snapshot_cutoff_at.clone(), "冻结快照截止时间缺失")?,
            )
            .await?;
            self.insert_postmatch_evaluation_sample_in_tx(
                &mut tx,
                id,
                &context,
                competition_profile_id,
                model_version_id,
                parameter_set_id,
                &horizon,
            )
            .await?;
            write_audit_event(
                &mut tx,
                "postmatch_settlement_created",
                "postmatch_settlement",
                Some(id.to_string()),
                json!({
                    "match_id": context.match_id,
                    "match_review_id": context.match_review_id,
                    "model_run_id": model_run_id,
                    "competition_profile_id": competition_profile_id,
                    "horizon": &horizon,
                    "result_fingerprint": &fingerprint,
                }),
            )
            .await?;
            id
        } else {
            let row = sqlx::query(
                "SELECT id, result_fingerprint FROM review.postmatch_settlements WHERE settlement_key=$1",
            )
            .bind(&settlement_key)
            .fetch_one(&mut *tx)
            .await?;
            let existing_fingerprint: String = row.try_get("result_fingerprint")?;
            if existing_fingerprint != fingerprint {
                return Err(PersistenceError::InvalidState(
                    "相同赛后结算键对应的正式赛果或预测评价已经发生变化".to_string(),
                ));
            }
            row.try_get("id")?
        };
        tx.commit().await?;
        self.read_postmatch_settlement(persisted_id).await
    }

    pub async fn list_postmatch_settlements(
        &self,
        limit: u32,
    ) -> PersistenceResult<Vec<PostmatchSettlementRecord>> {
        let rows = sqlx::query(&settlement_select_sql("ORDER BY settlement.settled_at DESC, settlement.id DESC LIMIT $1"))
            .bind(i64::from(limit.clamp(1, 500)))
            .fetch_all(&self.pool)
            .await?;
        rows.iter().map(settlement_from_row).collect()
    }

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
        let (accuracy_score, reliability_score) = verdict_scores(
            draft.verdict,
            &verification_state,
            &source_tier,
        );
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
        let partition = self.active_postmatch_partition(request.competition_id, &request.horizon).await?;
        self.refresh_provider_scores_for_partition(&partition).await?;
        self.refresh_drift_for_partition(
            &partition,
            request.baseline_size,
            request.current_size,
        )
        .await?;
        self.postmatch_overview(100).await
    }

    pub async fn postmatch_overview(&self, limit: u32) -> PersistenceResult<PostmatchOverview> {
        let settlements = self.list_postmatch_settlements(limit).await?;
        let evidence_queue = self.list_evidence_scoring_items(None, limit).await?;
        let provider_scores = self.list_provider_scores(limit).await?;
        let drift_runs = self.list_postmatch_drift_runs(limit).await?;
        let settlement_count = non_negative_u64(
            sqlx::query_scalar::<_, i64>("SELECT count(*)::bigint FROM review.postmatch_settlements")
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

    async fn load_settlement_context(
        &self,
        match_review_id: Uuid,
    ) -> PersistenceResult<SettlementContext> {
        let row = sqlx::query(
            r#"
            SELECT review.id AS match_review_id, review.match_id, fixture.external_key AS match_key,
                   home.canonical_name AS home_team_name, away.canonical_name AS away_team_name,
                   fixture.competition_id,
                   review.status AS review_status, review.data_coverage::double precision AS data_coverage,
                   review.prediction_evaluation, review.source_run_id AS model_run_id,
                   run.status AS run_status, run.feature_snapshot_id,
                   package.competition_profile_id, run.model_version_id,
                   run.parameter_set_id, run.rule_package_id,
                   run.snapshot_type AS horizon, fixture.kickoff_time,
                   result.home_goals_90, result.away_goals_90, result.finalized_at AS result_finalized_at,
                   snapshot.data_cutoff_time AS snapshot_cutoff_at,
                   snapshot.match_id AS snapshot_match_id,
                   snapshot.model_version_id AS snapshot_model_version_id,
                   snapshot.parameter_set_id AS snapshot_parameter_set_id,
                   snapshot.competition_profile_id AS snapshot_competition_profile_id,
                   snapshot.source_kind AS snapshot_source_kind,
                   snapshot.evidence_scope AS snapshot_evidence_scope,
                   settlement.id AS existing_settlement_id
            FROM review.match_reviews review
            JOIN football.matches fixture ON fixture.id = review.match_id
            JOIN football.teams home ON home.id = fixture.home_team_id
            JOIN football.teams away ON away.id = fixture.away_team_id
            LEFT JOIN football.match_results result ON result.match_id = fixture.id
            LEFT JOIN model.runs run ON run.id = review.source_run_id
            LEFT JOIN feature.snapshots snapshot ON snapshot.id = run.feature_snapshot_id
            LEFT JOIN model.rule_packages package ON package.id = run.rule_package_id
            LEFT JOIN review.postmatch_settlements settlement
              ON settlement.match_review_id = review.id AND settlement.model_run_id = run.id
            WHERE review.id=$1
            "#,
        )
        .bind(match_review_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("复盘记录不存在".to_string()))?;
        Ok(SettlementContext {
            match_review_id: row.try_get("match_review_id")?,
            match_id: row.try_get("match_id")?,
            match_key: row.try_get("match_key")?,
            home_team_name: row.try_get("home_team_name")?,
            away_team_name: row.try_get("away_team_name")?,
            competition_id: row.try_get("competition_id")?,
            review_status: row.try_get("review_status")?,
            data_coverage: row.try_get("data_coverage")?,
            prediction_evaluation: row.try_get("prediction_evaluation")?,
            model_run_id: row.try_get("model_run_id")?,
            run_status: row.try_get("run_status")?,
            feature_snapshot_id: row.try_get("feature_snapshot_id")?,
            competition_profile_id: row.try_get("competition_profile_id")?,
            model_version_id: row.try_get("model_version_id")?,
            parameter_set_id: row.try_get("parameter_set_id")?,
            rule_package_id: row.try_get("rule_package_id")?,
            horizon: row.try_get("horizon")?,
            kickoff_time: row.try_get("kickoff_time")?,
            home_goals_90: row.try_get("home_goals_90")?,
            away_goals_90: row.try_get("away_goals_90")?,
            result_finalized_at: row.try_get("result_finalized_at")?,
            snapshot_cutoff_at: row.try_get("snapshot_cutoff_at")?,
            snapshot_match_id: row.try_get("snapshot_match_id")?,
            snapshot_model_version_id: row.try_get("snapshot_model_version_id")?,
            snapshot_parameter_set_id: row.try_get("snapshot_parameter_set_id")?,
            snapshot_competition_profile_id: row.try_get("snapshot_competition_profile_id")?,
            snapshot_source_kind: row.try_get("snapshot_source_kind")?,
            snapshot_evidence_scope: row.try_get("snapshot_evidence_scope")?,
            existing_settlement_id: row.try_get("existing_settlement_id")?,
        })
    }

    async fn insert_evidence_scoring_items_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        settlement_id: Uuid,
        feature_snapshot_id: Uuid,
        data_cutoff_at: DateTime<Utc>,
    ) -> PersistenceResult<()> {
        let rows = sqlx::query(
            r#"
            SELECT DISTINCT claim.id AS evidence_id, document.provider_id,
                   claim.source_document_id, link.field_key, claim.verification_state,
                   claim.source_tier, claim.published_at, claim.retrieved_at
            FROM feature.snapshot_evidence link
            JOIN research.evidence_claims claim ON claim.id = link.evidence_id
            LEFT JOIN catalog.source_documents document ON document.id = claim.source_document_id
            WHERE link.snapshot_id=$1
            ORDER BY claim.id
            "#,
        )
        .bind(feature_snapshot_id)
        .fetch_all(&mut **tx)
        .await?;
        for row in rows {
            let evidence_id: Uuid = row.try_get("evidence_id")?;
            let provider_id: Option<Uuid> = row.try_get("provider_id")?;
            let source_document_id: Option<Uuid> = row.try_get("source_document_id")?;
            let field_key: String = row.try_get("field_key")?;
            let verification_state: String = row.try_get("verification_state")?;
            let source_tier: String = row.try_get("source_tier")?;
            let published_at: Option<DateTime<Utc>> = row.try_get("published_at")?;
            let retrieved_at: DateTime<Utc> = row.try_get("retrieved_at")?;
            if retrieved_at > data_cutoff_at
                || published_at
                    .as_ref()
                    .is_some_and(|published| published > &data_cutoff_at)
            {
                return Err(PersistenceError::InvalidState(format!(
                    "证据 {evidence_id} 超出冻结快照截止时间，不能进入正式评分队列"
                )));
            }
            let timeliness_score = calculate_timeliness_score(
                published_at.as_ref(),
                &retrieved_at,
                &data_cutoff_at,
            );
            let item_fingerprint = sha256_json(&json!({
                "settlement_id": settlement_id,
                "evidence_id": evidence_id,
                "provider_id": provider_id,
                "field_key": &field_key,
                "verification_state": &verification_state,
                "source_tier": &source_tier,
                "published_at": published_at.as_ref(),
                "retrieved_at": &retrieved_at,
                "data_cutoff_at": &data_cutoff_at,
                "timeliness_score": timeliness_score,
            }))?;
            sqlx::query(
                r#"
                INSERT INTO review.evidence_scoring_items (
                    id, settlement_id, evidence_id, provider_id, source_document_id,
                    field_key, verification_state, source_tier, published_at,
                    retrieved_at, data_cutoff_at, timeliness_score, item_fingerprint
                ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)
                ON CONFLICT (settlement_id, evidence_id, field_key) DO NOTHING
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(settlement_id)
            .bind(evidence_id)
            .bind(provider_id)
            .bind(source_document_id)
            .bind(&field_key)
            .bind(&verification_state)
            .bind(&source_tier)
            .bind(published_at.as_ref())
            .bind(&retrieved_at)
            .bind(&data_cutoff_at)
            .bind(timeliness_score)
            .bind(&item_fingerprint)
            .execute(&mut **tx)
            .await?;
        }
        Ok(())
    }

    async fn insert_postmatch_evaluation_sample_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        settlement_id: Uuid,
        context: &SettlementContext,
        competition_profile_id: Uuid,
        model_version_id: Uuid,
        parameter_set_id: Uuid,
        horizon: &str,
    ) -> PersistenceResult<()> {
        let evaluation = &context.prediction_evaluation;
        if !evaluation
            .get("available")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        {
            return Err(PersistenceError::InvalidState(
                "最终复盘没有可用预测评价，不能进入正式结算样本".to_string(),
            ));
        }
        let run_id = required(context.model_run_id, "复盘未绑定成功推演")?;
        let actual_outcome = json_text(evaluation, "actual_outcome")?;
        let expected_actual_outcome = match (
            required(context.home_goals_90, "正式赛果缺失")?,
            required(context.away_goals_90, "正式赛果缺失")?,
        ) {
            (home, away) if home > away => "home_win",
            (home, away) if home < away => "away_win",
            _ => "draw",
        };
        if actual_outcome != expected_actual_outcome {
            return Err(PersistenceError::InvalidState(
                "最终复盘的实际赛果标签与冻结比分不一致".to_string(),
            ));
        }
        let probabilities = evaluation
            .get("predicted_probabilities")
            .ok_or_else(|| PersistenceError::InvalidState("预测概率快照缺失".to_string()))?;
        let home_win = json_f64(probabilities, "home_win")?;
        let draw = json_f64(probabilities, "draw")?;
        let away_win = json_f64(probabilities, "away_win")?;
        validate_probabilities(home_win, draw, away_win)?;
        let log_loss = json_f64(evaluation, "log_loss")?;
        let brier = json_f64(evaluation, "brier")?;
        let scoreline_nll = evaluation.get("scoreline_nll").and_then(Value::as_f64);
        if log_loss < 0.0
            || !(0.0..=2.0).contains(&brier)
            || scoreline_nll.is_some_and(|value| !value.is_finite() || value < 0.0)
            || !context.data_coverage.is_finite()
            || !(0.0..=1.0).contains(&context.data_coverage)
        {
            return Err(PersistenceError::InvalidState(
                "最终复盘评价指标或数据覆盖率超出合法范围".to_string(),
            ));
        }
        sqlx::query(
            r#"
            INSERT INTO analytics.evaluation_samples (
                id, review_id, run_id, model_version_id, parameter_set_id,
                competition_id, season_id, stage_id, snapshot_type, kickoff_time,
                actual_outcome, home_win, draw, away_win, log_loss, brier,
                scoreline_nll, data_coverage, calculation_version, settlement_id,
                competition_profile_id, calculated_at
            )
            SELECT $1,$2,$3,$4,$5,fixture.competition_id,fixture.season_id,fixture.stage_id,
                   $6,fixture.kickoff_time,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,now()
            FROM football.matches fixture WHERE fixture.id=$18
            ON CONFLICT (run_id, kickoff_time, calculation_version) DO NOTHING
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(context.match_review_id)
        .bind(run_id)
        .bind(model_version_id)
        .bind(parameter_set_id)
        .bind(horizon)
        .bind(actual_outcome)
        .bind(home_win)
        .bind(draw)
        .bind(away_win)
        .bind(log_loss)
        .bind(brier)
        .bind(scoreline_nll)
        .bind(context.data_coverage)
        .bind(POSTMATCH_MONITORING_VERSION)
        .bind(settlement_id)
        .bind(competition_profile_id)
        .bind(context.match_id)
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    async fn read_postmatch_settlement(
        &self,
        settlement_id: Uuid,
    ) -> PersistenceResult<PostmatchSettlementRecord> {
        let row = sqlx::query(&settlement_select_sql("WHERE settlement.id=$1"))
            .bind(settlement_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| PersistenceError::InvalidState("赛后结算记录不存在".to_string()))?;
        settlement_from_row(&row)
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
                "{}:{}:{}:{}",
                model_version_id, competition_profile_id, parameter_set_id, horizon
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
            entry.decision_fingerprints.push(row.try_get("decision_fingerprint")?);
        }
        for (provider_id, aggregate) in groups {
            let accuracy_mean = mean(&aggregate.accuracy).unwrap_or(0.0);
            let timeliness_mean = mean(&aggregate.timeliness).unwrap_or(0.0);
            let reliability_mean = mean(&aggregate.reliability).unwrap_or(0.0);
            let weighted_score = if aggregate.accuracy.is_empty() {
                0.0
            } else {
                round6(
                    accuracy_mean * 0.60
                        + timeliness_mean * 0.15
                        + reliability_mean * 0.25,
                )
            };
            let decision_set_sha256 = sha256_json(&aggregate.decision_fingerprints)?;
            let snapshot_key = format!(
                "{}:{}:{}:{}",
                provider_id, partition.partition_key, decision_set_sha256, POSTMATCH_MONITORING_VERSION
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
        } else if calculation.drift.iter().any(|item| item.severity == "critical") {
            "critical"
        } else if calculation.drift.iter().any(|item| item.severity == "warning") {
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
                baseline_size: non_negative_u64(
                    row.try_get("baseline_size")?,
                    "漂移基线样本",
                )?,
                current_size: non_negative_u64(
                    row.try_get("current_size")?,
                    "漂移当前样本",
                )?,
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

fn readiness_from_context(context: &SettlementContext) -> PostmatchSettlementReadiness {
    let result_ready = context.home_goals_90.is_some()
        && context.away_goals_90.is_some()
        && context.result_finalized_at.is_some();
    let finalized_review_ready = context.review_status == "finalized"
        && context
            .prediction_evaluation
            .get("available")
            .and_then(Value::as_bool)
            .unwrap_or(false);
    let successful_run_ready = context.model_run_id.is_some()
        && context.run_status.as_deref() == Some("succeeded")
        && context.model_version_id.is_some()
        && context.parameter_set_id.is_some()
        && context.rule_package_id.is_some();
    let frozen_snapshot_ready = context.feature_snapshot_id.is_some()
        && context.snapshot_cutoff_at.is_some();
    let snapshot_identity_ready = context.snapshot_match_id == Some(context.match_id)
        && context.snapshot_model_version_id == context.model_version_id
        && context.snapshot_parameter_set_id == context.parameter_set_id
        && context.snapshot_competition_profile_id == context.competition_profile_id;
    let real_evidence_snapshot_ready = matches!(
        context.snapshot_source_kind.as_deref(),
        Some("real" | "manual")
    ) && context.snapshot_evidence_scope.as_deref() == Some("real");
    let competition_profile_ready = context.competition_id.is_some()
        && context.competition_profile_id.is_some();
    let formal_horizon_ready = context
        .horizon
        .as_deref()
        .is_some_and(|value| FORMAL_HORIZONS.contains(&value));
    let mut blocked_reasons = Vec::new();
    if !result_ready {
        blocked_reasons.push("缺少已最终确认的 90 分钟正式赛果".to_string());
    }
    if !finalized_review_ready {
        blocked_reasons.push("复盘未最终完成或没有可用预测评价".to_string());
    }
    if !successful_run_ready {
        blocked_reasons.push("复盘未绑定完整的成功推演版本".to_string());
    }
    if !frozen_snapshot_ready {
        blocked_reasons.push("成功推演未绑定不可变冻结快照".to_string());
    }
    if !snapshot_identity_ready {
        blocked_reasons.push("冻结快照与比赛、模型、参数或赛事 Profile 身份不一致".to_string());
    }
    if !real_evidence_snapshot_ready {
        blocked_reasons.push("只有真实或人工冻结、且证据域为 real 的快照才能进入正式赛后结算".to_string());
    }
    if !competition_profile_ready {
        blocked_reasons.push("比赛赛事或赛事 Profile 缺失".to_string());
    }
    if !formal_horizon_ready {
        blocked_reasons.push("只有 T-N、T-24h、T-6h、T-1h 可进入正式结算".to_string());
    }
    PostmatchSettlementReadiness {
        match_review_id: context.match_review_id,
        match_id: context.match_id,
        match_key: context.match_key.clone(),
        home_team_name: context.home_team_name.clone(),
        away_team_name: context.away_team_name.clone(),
        result_ready,
        finalized_review_ready,
        successful_run_ready,
        frozen_snapshot_ready,
        snapshot_identity_ready,
        real_evidence_snapshot_ready,
        competition_profile_ready,
        formal_horizon_ready,
        existing_settlement_id: context.existing_settlement_id,
        ready: blocked_reasons.is_empty(),
        blocked_reasons,
    }
}

fn settlement_select_sql(suffix: &str) -> String {
    format!(
        r#"
        SELECT settlement.id, settlement.match_id, settlement.match_review_id,
               settlement.model_run_id, settlement.feature_snapshot_id,
               settlement.competition_id, competition.name AS competition_name,
               settlement.competition_profile_id, settlement.model_version_id,
               version.version AS model_version, settlement.parameter_set_id,
               parameter.parameter_version, settlement.rule_package_id,
               settlement.horizon, fixture.external_key AS match_key,
               home.canonical_name AS home_team_name,
               away.canonical_name AS away_team_name,
               settlement.home_goals_90, settlement.away_goals_90,
               settlement.result_finalized_at, settlement.result_fingerprint, settlement.settlement_key,
               settlement.settlement_version, settlement.status,
               settlement.settled_by, settlement.settlement_note,
               settlement.metadata, settlement.settled_at,
               COALESCE(evidence_counts.item_count,0)::bigint AS evidence_item_count,
               COALESCE(evidence_counts.scored_count,0)::bigint AS scored_evidence_count,
               drift.status AS drift_status
        FROM review.postmatch_settlements settlement
        JOIN football.matches fixture ON fixture.id=settlement.match_id
        JOIN football.teams home ON home.id=fixture.home_team_id
        JOIN football.teams away ON away.id=fixture.away_team_id
        JOIN football.competitions competition ON competition.id=settlement.competition_id
        JOIN model.versions version ON version.id=settlement.model_version_id
        JOIN model.parameter_sets parameter ON parameter.id=settlement.parameter_set_id
        LEFT JOIN LATERAL (
            SELECT count(*) AS item_count,
                   count(decision.id) FILTER (WHERE decision.verdict <> 'not_verifiable') AS scored_count
            FROM review.evidence_scoring_items item
            LEFT JOIN review.evidence_scoring_decisions decision ON decision.item_id=item.id
            WHERE item.settlement_id=settlement.id
        ) evidence_counts ON true
        LEFT JOIN LATERAL (
            SELECT run.status
            FROM analytics.postmatch_drift_runs run
            WHERE run.competition_id=settlement.competition_id
              AND run.competition_profile_id=settlement.competition_profile_id
              AND run.model_version_id=settlement.model_version_id
              AND run.parameter_set_id=settlement.parameter_set_id
              AND run.horizon=settlement.horizon
            ORDER BY run.generated_at DESC, run.id DESC LIMIT 1
        ) drift ON true
        {suffix}
        "#
    )
}

fn settlement_from_row(row: &sqlx::postgres::PgRow) -> PersistenceResult<PostmatchSettlementRecord> {
    Ok(PostmatchSettlementRecord {
        id: row.try_get("id")?,
        match_id: row.try_get("match_id")?,
        match_review_id: row.try_get("match_review_id")?,
        model_run_id: row.try_get("model_run_id")?,
        feature_snapshot_id: row.try_get("feature_snapshot_id")?,
        competition_id: row.try_get("competition_id")?,
        competition_name: row.try_get("competition_name")?,
        competition_profile_id: row.try_get("competition_profile_id")?,
        model_version_id: row.try_get("model_version_id")?,
        model_version: row.try_get("model_version")?,
        parameter_set_id: row.try_get("parameter_set_id")?,
        parameter_version: row.try_get("parameter_version")?,
        rule_package_id: row.try_get("rule_package_id")?,
        horizon: row.try_get("horizon")?,
        match_key: row.try_get("match_key")?,
        home_team_name: row.try_get("home_team_name")?,
        away_team_name: row.try_get("away_team_name")?,
        home_goals_90: row.try_get("home_goals_90")?,
        away_goals_90: row.try_get("away_goals_90")?,
        result_finalized_at: row.try_get("result_finalized_at")?,
        result_fingerprint: row.try_get("result_fingerprint")?,
        settlement_key: row.try_get("settlement_key")?,
        settlement_version: row.try_get("settlement_version")?,
        status: row.try_get("status")?,
        evidence_item_count: non_negative_u64(row.try_get("evidence_item_count")?, "证据队列数量")?,
        scored_evidence_count: non_negative_u64(row.try_get("scored_evidence_count")?, "已评分证据数量")?,
        drift_status: row.try_get("drift_status")?,
        settled_by: row.try_get("settled_by")?,
        settlement_note: row.try_get("settlement_note")?,
        metadata: row.try_get("metadata")?,
        settled_at: row.try_get("settled_at")?,
    })
}

fn evidence_item_from_row(row: &sqlx::postgres::PgRow) -> PersistenceResult<EvidenceScoringItemRecord> {
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

fn provider_score_from_row(row: &sqlx::postgres::PgRow) -> PersistenceResult<ProviderScoreSnapshotRecord> {
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
        not_verifiable_count: non_negative_u64(row.try_get("not_verifiable_count")?, "不可验证证据数量")?,
        accuracy_mean: row.try_get("accuracy_mean")?,
        timeliness_mean: row.try_get("timeliness_mean")?,
        reliability_mean: row.try_get("reliability_mean")?,
        weighted_score: row.try_get("weighted_score")?,
        decision_set_sha256: row.try_get("decision_set_sha256")?,
        calculation_version: row.try_get("calculation_version")?,
        generated_at: row.try_get("generated_at")?,
    })
}

fn drift_finding_from_row(row: &sqlx::postgres::PgRow) -> PersistenceResult<PostmatchDriftFindingRecord> {
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

fn calculate_timeliness_score(
    published_at: Option<&DateTime<Utc>>,
    retrieved_at: &DateTime<Utc>,
    cutoff_at: &DateTime<Utc>,
) -> f64 {
    let reference = published_at.unwrap_or(retrieved_at);
    if reference > cutoff_at || retrieved_at > cutoff_at {
        return 0.0;
    }
    let age_hours = cutoff_at
        .signed_duration_since(reference.clone())
        .num_minutes()
        .max(0) as f64
        / 60.0;
    let score = if age_hours <= 6.0 {
        1.0
    } else if age_hours <= 24.0 {
        0.9
    } else if age_hours <= 72.0 {
        0.75
    } else if age_hours <= 168.0 {
        0.5
    } else {
        0.25
    };
    round6(score)
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
    let reliability = (accuracy * 0.70 + verification_quality * 0.15 + source_quality * 0.15)
        .clamp(0.0, 1.0);
    (Some(round6(accuracy)), Some(round6(reliability)))
}

fn window_json(samples: &[EvaluationSample]) -> Value {
    json!({
        "sample_size": samples.len(),
        "start": samples.first().map(|item| item.kickoff_time.clone()),
        "end": samples.last().map(|item| item.kickoff_time.clone()),
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

fn validate_probabilities(home: f64, draw: f64, away: f64) -> PersistenceResult<()> {
    let values = [home, draw, away];
    let sum = values.iter().sum::<f64>();
    if values
        .iter()
        .all(|value| value.is_finite() && (0.0..=1.0).contains(value))
        && sum.is_finite()
        && (sum - 1.0).abs() <= 0.0001
    {
        Ok(())
    } else {
        Err(PersistenceError::InvalidState(
            "正式结算样本的胜平负概率非法或未归一化".to_string(),
        ))
    }
}

fn json_text(value: &Value, key: &str) -> PersistenceResult<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| PersistenceError::InvalidState(format!("预测评价缺少字段 {key}")))
}

fn json_f64(value: &Value, key: &str) -> PersistenceResult<f64> {
    value
        .get(key)
        .and_then(Value::as_f64)
        .filter(|item| item.is_finite())
        .ok_or_else(|| PersistenceError::InvalidState(format!("预测评价字段 {key} 非法")))
}

fn required<T>(value: Option<T>, message: &str) -> PersistenceResult<T> {
    value.ok_or_else(|| PersistenceError::InvalidState(message.to_string()))
}

fn non_negative_u64(value: i64, label: &str) -> PersistenceResult<u64> {
    u64::try_from(value).map_err(|_| PersistenceError::InvalidState(format!("{label}不能为负数")))
}

fn mean(values: &[f64]) -> Option<f64> {
    (!values.is_empty()).then(|| values.iter().sum::<f64>() / values.len() as f64)
}

fn round6(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn timeliness_is_bounded_and_rewards_fresh_sources() {
        let cutoff = Utc.with_ymd_and_hms(2026, 1, 2, 12, 0, 0).unwrap();
        let fresh = Utc.with_ymd_and_hms(2026, 1, 2, 9, 0, 0).unwrap();
        let old = Utc.with_ymd_and_hms(2025, 12, 20, 9, 0, 0).unwrap();
        assert_eq!(calculate_timeliness_score(Some(&fresh), &fresh, &cutoff), 1.0);
        assert_eq!(calculate_timeliness_score(Some(&old), &old, &cutoff), 0.25);
        assert_eq!(calculate_timeliness_score(Some(&cutoff), &cutoff, &cutoff), 1.0);
    }

    #[test]
    fn not_verifiable_does_not_fake_accuracy() {
        assert_eq!(
            verdict_scores(EvidenceVerdict::NotVerifiable, "CONFIRMED", "official"),
            (None, None)
        );
        let (accuracy, reliability) = verdict_scores(EvidenceVerdict::Correct, "CONFIRMED", "official");
        assert_eq!(accuracy, Some(1.0));
        assert_eq!(reliability, Some(1.0));
    }
}
