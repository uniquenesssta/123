use super::{context::SettlementContext, readiness::readiness_from_context};
use chrono::{TimeZone, Utc};
use serde_json::json;
use uuid::Uuid;

fn id(value: u128) -> Uuid {
    Uuid::from_u128(value)
}

fn context() -> SettlementContext {
    let time = Utc.with_ymd_and_hms(2026, 10, 13, 0, 0, 0).unwrap();
    SettlementContext {
        match_review_id: id(1),
        match_id: id(2),
        match_key: "fixture".into(),
        home_team_name: "home".into(),
        away_team_name: "away".into(),
        competition_id: Some(id(3)),
        review_status: "finalized".into(),
        data_coverage: 0.75,
        prediction_evaluation: json!({"available": true}),
        model_run_id: Some(id(4)),
        run_status: Some("succeeded".into()),
        feature_snapshot_id: Some(id(5)),
        competition_profile_id: Some(id(6)),
        model_version_id: Some(id(7)),
        parameter_set_id: Some(id(8)),
        rule_package_id: Some(id(9)),
        horizon: Some("T-6h".into()),
        kickoff_time: time,
        home_goals_90: Some(1),
        away_goals_90: Some(0),
        result_finalized_at: Some(time),
        snapshot_cutoff_at: Some(time),
        snapshot_match_id: Some(id(2)),
        snapshot_model_version_id: Some(id(7)),
        snapshot_parameter_set_id: Some(id(8)),
        snapshot_competition_profile_id: Some(id(6)),
        snapshot_source_kind: Some("real".into()),
        snapshot_evidence_scope: Some("real".into()),
        existing_settlement_id: None,
    }
}

#[test]
fn readiness_preserves_formal_horizons_and_real_manual_snapshot_sources() {
    for horizon in ["T-N", "T-24h", "T-6h", "T-1h"] {
        for source in ["real", "manual"] {
            let mut input = context();
            input.horizon = Some(horizon.into());
            input.snapshot_source_kind = Some(source.into());
            let result = readiness_from_context(&input);
            assert!(result.ready);
            assert!(result.blocked_reasons.is_empty());
            assert_eq!(result.match_review_id, id(1));
            assert_eq!(result.match_id, id(2));
            assert_eq!(result.match_key, "fixture");
            assert_eq!(result.home_team_name, "home");
            assert_eq!(result.away_team_name, "away");
        }
    }
}

#[test]
fn readiness_accumulates_original_eight_blockers_in_order() {
    let mut input = context();
    input.home_goals_90 = None;
    input.away_goals_90 = None;
    input.result_finalized_at = None;
    input.review_status = "superseded".into();
    input.prediction_evaluation = json!({"available": false});
    input.model_run_id = None;
    input.run_status = None;
    input.model_version_id = None;
    input.parameter_set_id = None;
    input.rule_package_id = None;
    input.feature_snapshot_id = None;
    input.snapshot_cutoff_at = None;
    input.snapshot_match_id = None;
    input.snapshot_model_version_id = None;
    input.snapshot_parameter_set_id = None;
    input.snapshot_competition_profile_id = None;
    input.snapshot_source_kind = None;
    input.snapshot_evidence_scope = None;
    input.competition_id = None;
    input.competition_profile_id = None;
    input.horizon = None;
    let result = readiness_from_context(&input);
    assert!(!result.ready);
    assert_eq!(
        result.blocked_reasons,
        [
            "缺少已最终确认的 90 分钟正式赛果",
            "复盘未最终完成或没有可用预测评价",
            "复盘未绑定完整的成功推演版本",
            "成功推演未绑定不可变冻结快照",
            "冻结快照与比赛、模型、参数或赛事 Profile 身份不一致",
            "只有真实或人工冻结、且证据域为 real 的快照才能进入正式赛后结算",
            "比赛赛事或赛事 Profile 缺失",
            "只有 T-N、T-24h、T-6h、T-1h 可进入正式结算",
        ]
    );
}

#[test]
fn readiness_rejects_each_snapshot_identity_axis_and_nonformal_scope() {
    for axis in 0..4 {
        let mut input = context();
        match axis {
            0 => input.snapshot_match_id = Some(id(99)),
            1 => input.snapshot_model_version_id = Some(id(99)),
            2 => input.snapshot_parameter_set_id = Some(id(99)),
            _ => input.snapshot_competition_profile_id = Some(id(99)),
        }
        let result = readiness_from_context(&input);
        assert!(!result.snapshot_identity_ready);
        assert_eq!(result.blocked_reasons.len(), 1);
    }
    for source in ["synthetic", "REAL", ""] {
        let mut input = context();
        input.snapshot_source_kind = Some(source.into());
        assert!(!readiness_from_context(&input).real_evidence_snapshot_ready);
    }
    let mut input = context();
    input.snapshot_evidence_scope = Some("synthetic".into());
    input.horizon = Some("T-2h".into());
    let result = readiness_from_context(&input);
    assert!(!result.real_evidence_snapshot_ready);
    assert!(!result.formal_horizon_ready);
    assert_eq!(result.blocked_reasons.len(), 2);
}

#[test]
fn existing_settlement_identity_does_not_override_current_readiness_guards() {
    let mut input = context();
    input.existing_settlement_id = Some(id(42));
    assert!(readiness_from_context(&input).ready);
    input.home_goals_90 = None;
    let result = readiness_from_context(&input);
    assert!(!result.ready);
    assert_eq!(result.existing_settlement_id, Some(id(42)));
    assert!(!result.result_ready);
    assert_eq!(result.blocked_reasons.len(), 1);
}
