use super::context::SettlementContext;
use crate::{postmatch::FORMAL_HORIZONS, PersistenceResult, PostgresStore};
use football_domain::PostmatchSettlementReadiness;
use serde_json::Value;
use uuid::Uuid;

impl PostgresStore {
    pub async fn postmatch_settlement_readiness(
        &self,
        match_review_id: Uuid,
    ) -> PersistenceResult<PostmatchSettlementReadiness> {
        let context = self.load_settlement_context(match_review_id).await?;
        Ok(readiness_from_context(&context))
    }
}

pub(super) fn readiness_from_context(context: &SettlementContext) -> PostmatchSettlementReadiness {
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
    let frozen_snapshot_ready =
        context.feature_snapshot_id.is_some() && context.snapshot_cutoff_at.is_some();
    let snapshot_identity_ready = context.snapshot_match_id == Some(context.match_id)
        && context.snapshot_model_version_id == context.model_version_id
        && context.snapshot_parameter_set_id == context.parameter_set_id
        && context.snapshot_competition_profile_id == context.competition_profile_id;
    let real_evidence_snapshot_ready = matches!(
        context.snapshot_source_kind.as_deref(),
        Some("real" | "manual")
    ) && context.snapshot_evidence_scope.as_deref()
        == Some("real");
    let competition_profile_ready =
        context.competition_id.is_some() && context.competition_profile_id.is_some();
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
        blocked_reasons
            .push("只有真实或人工冻结、且证据域为 real 的快照才能进入正式赛后结算".to_string());
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
