use super::{
    evaluation_sample::EvaluationSamplePartition, readiness::readiness_from_context,
    validation::required,
};
use crate::{sha256_json, write_audit_event, PersistenceError, PersistenceResult, PostgresStore};
use chrono::{DateTime, Utc};
use football_domain::{
    PostmatchSettlementDraft, PostmatchSettlementRecord, POSTMATCH_SETTLEMENT_VERSION,
};
use serde::Serialize;
use serde_json::{json, Value};
use sqlx::Row;
use uuid::Uuid;

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
        let result_finalized_at = required(context.result_finalized_at, "正式赛果时间缺失")?;
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
            result_finalized_at,
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
        .bind(result_finalized_at)
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
                required(context.snapshot_cutoff_at, "冻结快照截止时间缺失")?,
            )
            .await?;
            self.insert_postmatch_evaluation_sample_in_tx(
                &mut tx,
                id,
                &context,
                EvaluationSamplePartition {
                    competition_profile_id,
                    model_version_id,
                    parameter_set_id,
                    horizon: &horizon,
                },
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
}
