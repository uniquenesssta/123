use super::*;
use crate::{
    model_registry::ModelRegistry,
    ports::{
        analytics::{ParameterDefinition, ParameterLifecyclePort},
        PortError, PortErrorKind, PortResult,
    },
    ApplicationError,
};
use async_trait::async_trait;
use football_domain::{
    ParameterLifecycleReadiness, ParameterLifecycleReadinessRequest,
    ParameterPromotionDecisionRecord, ParameterPromotionRequest, ParameterReplayFixture,
    ParameterRollbackRequest, ParameterShadowValidationRecord, ParameterShadowValidationRequest,
    ParameterTuningCandidateRecord, ParameterTuningDecision, ParameterTuningDecisionDraft,
    ParameterTuningDraft,
};
use serde_json::{json, Value};
use std::sync::Mutex;
use uuid::Uuid;

fn candidate() -> ParameterTuningCandidateRecord {
    serde_json::from_value(json!({
        "id": Uuid::from_u128(1), "competition_id": Uuid::from_u128(2),
        "competition_name": "original", "competition_profile_id": Uuid::from_u128(3),
        "partition_key": "original partition", "model_key": "external-provider",
        "model_version": "baseline model", "parameter_version": "baseline parameter",
        "snapshot_type": "T-1h", "target_module": "provider-owned", "sample_size": 50,
        "baseline_model_version_id": Uuid::from_u128(4), "baseline_parameter_set_id": Uuid::from_u128(5),
        "candidate_model_version_id": Uuid::from_u128(6), "candidate_parameter_set_id": Uuid::from_u128(7),
        "candidate_model_version": "candidate model", "candidate_parameter_version": "candidate parameter",
        "candidate_definition_sha256": "original definition", "baseline_metrics": {"opaque": "baseline"},
        "calibration_bias": {}, "proposed_adjustments": {}, "constraints": {"minimum_sample_size": 50},
        "training_window": {}, "validation_window": {}, "holdout_window": {}, "rationale": "original",
        "status": "accepted_for_backtest", "created_at": "2026-10-14T00:00:00Z",
        "decided_at": null, "decision_note": null
    })).unwrap()
}
fn readiness() -> ParameterLifecycleReadiness {
    serde_json::from_value(json!({
        "partition_key": "original ready partition", "competition_id": Uuid::from_u128(2),
        "competition_name": "original", "competition_profile_id": Uuid::from_u128(3),
        "snapshot_type": "T-1h", "h_contract_ready": true, "h_contract_version": "original H",
        "settled_sample_count": 70, "eligible_sample_count": 50, "minimum_sample_size": 50,
        "active_model_version_id": Uuid::from_u128(4), "active_parameter_set_id": Uuid::from_u128(5),
        "active_model_version": "baseline model", "active_parameter_version": "baseline parameter",
        "blocked_reasons": [], "ready_for_shadow_validation": true, "ready_for_promotion": true
    })).unwrap()
}
fn decision() -> ParameterPromotionDecisionRecord {
    serde_json::from_value(json!({"id": Uuid::from_u128(8), "candidate_id": Uuid::from_u128(1),
        "decision": "original ledger receipt", "previous_binding_state": [{"opaque": "before"}],
        "new_binding_state": [{"opaque": "after"}], "decided_by": null, "decision_note": "original note",
        "created_at": "2026-10-14T00:00:00Z"})).unwrap()
}
fn replay_fixture(id: u128) -> ParameterReplayFixture {
    serde_json::from_value(
        json!({"review_id": Uuid::from_u128(id+100), "run_id": Uuid::from_u128(id),
        "match_id": Uuid::from_u128(id+200), "match_key": "original fixture",
        "competition_id": Uuid::from_u128(2), "season_id": null, "stage_id": null,
        "competition_kind": "league", "competition_profile_id": Uuid::from_u128(3),
        "kickoff_time": "2026-10-14T00:00:00Z", "home_team_name": "home", "away_team_name": "away",
        "snapshot_type": "T-1h", "input_payload": {}, "rule_package_version": "original rules",
        "actual_home_goals": 1, "actual_away_goals": 0, "baseline_home_win": 0.5,
        "baseline_draw": 0.25, "baseline_away_win": 0.25, "baseline_scoreline_probability": 0.25,
        "data_coverage": 1.0}),
    )
    .unwrap()
}
struct Fixture {
    candidate: ParameterTuningCandidateRecord,
    readiness: ParameterLifecycleReadiness,
    fixtures: Vec<ParameterReplayFixture>,
    calls: Mutex<Vec<Value>>,
    fail_at: Option<&'static str>,
    error_kind: PortErrorKind,
}
impl Fixture {
    fn new() -> Self {
        Self {
            candidate: candidate(),
            readiness: readiness(),
            fixtures: Vec::new(),
            calls: Mutex::new(Vec::new()),
            fail_at: None,
            error_kind: PortErrorKind::Infrastructure,
        }
    }
    fn note(&self, method: &'static str, value: Value) -> PortResult<()> {
        self.calls
            .lock()
            .unwrap()
            .push(json!({"method": method, "data": value}));
        if self.fail_at == Some(method) {
            return Err(PortError::new(
                self.error_kind,
                format!("injected {method}"),
            ));
        }
        Ok(())
    }
    fn calls(&self) -> Vec<Value> {
        self.calls.lock().unwrap().clone()
    }
    fn names(&self) -> Vec<String> {
        self.calls()
            .iter()
            .map(|call| call["method"].as_str().unwrap().to_string())
            .collect()
    }
}
#[async_trait]
impl ParameterLifecyclePort for Fixture {
    async fn readiness(
        &self,
        request: &ParameterLifecycleReadinessRequest,
    ) -> PortResult<ParameterLifecycleReadiness> {
        self.note("readiness", json!(request))?;
        Ok(self.readiness.clone())
    }
    async fn list_tuning_candidates(
        &self,
        limit: u32,
    ) -> PortResult<Vec<ParameterTuningCandidateRecord>> {
        self.note("list", json!({"limit": limit}))?;
        Ok(vec![self.candidate.clone()])
    }
    async fn read_tuning_candidate(
        &self,
        candidate_id: Uuid,
    ) -> PortResult<ParameterTuningCandidateRecord> {
        self.note("read", json!({"id": candidate_id}))?;
        Ok(self.candidate.clone())
    }
    async fn decide_tuning_candidate(
        &self,
        draft: &ParameterTuningDecisionDraft,
    ) -> PortResult<ParameterTuningCandidateRecord> {
        self.note("decide", json!(draft))?;
        Ok(self.candidate.clone())
    }
    async fn load_replay_fixtures(
        &self,
        competition_id: Uuid,
        competition_profile_id: Uuid,
        snapshot_type: &str,
        baseline_model_version_id: Uuid,
        baseline_parameter_set_id: Uuid,
    ) -> PortResult<Vec<ParameterReplayFixture>> {
        self.note("fixtures", json!({"competition": competition_id, "profile": competition_profile_id,
            "snapshot": snapshot_type, "model": baseline_model_version_id, "parameter": baseline_parameter_set_id}))?;
        Ok(self.fixtures.clone())
    }
    async fn read_parameter_set_definition(
        &self,
        parameter_set_id: Uuid,
    ) -> PortResult<ParameterDefinition> {
        self.note("definition", json!({"id": parameter_set_id}))?;
        Ok(ParameterDefinition::from_value(
            json!({"opaque": "private parameters"}),
        ))
    }
    async fn save_shadow_validation(
        &self,
        record: &ParameterShadowValidationRecord,
    ) -> PortResult<ParameterShadowValidationRecord> {
        self.note("save", json!(record))?;
        Ok(record.clone())
    }
    async fn list_shadow_validations(
        &self,
        candidate_id: Uuid,
    ) -> PortResult<Vec<ParameterShadowValidationRecord>> {
        self.note("shadows", json!({"id": candidate_id}))?;
        Ok(Vec::new())
    }
    async fn promote(
        &self,
        request: &ParameterPromotionRequest,
    ) -> PortResult<ParameterPromotionDecisionRecord> {
        self.note("promote", json!(request))?;
        Ok(decision())
    }
    async fn rollback(
        &self,
        request: &ParameterRollbackRequest,
    ) -> PortResult<ParameterPromotionDecisionRecord> {
        self.note("rollback", json!(request))?;
        Ok(decision())
    }
    async fn list_promotion_decisions(
        &self,
        candidate_id: Uuid,
    ) -> PortResult<Vec<ParameterPromotionDecisionRecord>> {
        self.note("decisions", json!({"id": candidate_id}))?;
        Ok(vec![decision()])
    }
}
fn promotion(note: &str) -> ParameterPromotionRequest {
    ParameterPromotionRequest {
        candidate_id: Uuid::from_u128(1),
        decided_by: Some(" reviewer ".into()),
        decision_note: note.into(),
    }
}
fn validation() -> ParameterShadowValidationRequest {
    ParameterShadowValidationRequest {
        candidate_id: Uuid::from_u128(1),
    }
}

#[tokio::test]
async fn generation_keeps_public_provider_unavailable_for_every_draft() {
    for draft in [
        json!({"target_module": "provider-owned"}),
        json!({"target_module": "", "snapshot_type": "unknown", "minimum_sample_size": 0}),
    ] {
        assert!(
            matches!(generate_parameter_tuning_candidate(serde_json::from_value::<ParameterTuningDraft>(draft).unwrap()).await.unwrap_err(),
            ApplicationError::Model(message) if message == "公开仓库未捆绑模型提供器，不能生成提供器私有参数候选")
        );
    }
}

#[tokio::test]
async fn readiness_preserves_four_horizons_minimum_boundary_and_original_result() {
    for snapshot in ["T-N", "T-24h", "T-6h", "T-1h"] {
        let fixture = Fixture::new();
        let request = ParameterLifecycleReadinessRequest {
            competition_id: None,
            snapshot_type: snapshot.into(),
            minimum_sample_size: 20,
        };
        let result = parameter_lifecycle_readiness(&fixture, request.clone())
            .await
            .unwrap();
        assert_eq!(fixture.names(), ["readiness"]);
        assert_eq!(fixture.calls()[0]["data"], json!(request));
        assert_eq!(json!(result), json!(fixture.readiness));
    }
}

#[tokio::test]
async fn readiness_rejects_invalid_horizon_before_sample_size_or_port() {
    for snapshot in ["", "T-1h ", "t-1h", "T-2h"] {
        let fixture = Fixture::new();
        assert!(
            matches!(parameter_lifecycle_readiness(&fixture, ParameterLifecycleReadinessRequest {
            competition_id: None, snapshot_type: snapshot.into(), minimum_sample_size: 0 }).await.unwrap_err(),
            ApplicationError::Validation(message) if message.ends_with(&format!("收到：{snapshot}")))
        );
        assert!(fixture.calls().is_empty());
    }
    let fixture = Fixture::new();
    assert!(
        matches!(parameter_lifecycle_readiness(&fixture, ParameterLifecycleReadinessRequest {
        competition_id: None, snapshot_type: "T-N".into(), minimum_sample_size: 19 }).await.unwrap_err(),
        ApplicationError::Validation(message) if message == "最低样本量不能少于 20 场")
    );
    assert!(fixture.calls().is_empty());
}

#[tokio::test]
async fn candidate_list_and_manual_decisions_forward_limits_notes_and_records() {
    for limit in [0, u32::MAX] {
        let fixture = Fixture::new();
        assert_eq!(
            json!(list_parameter_tuning_candidates(&fixture, limit)
                .await
                .unwrap()),
            json!([fixture.candidate])
        );
        assert_eq!(fixture.calls()[0]["data"], json!({"limit": limit}));
    }
    for decision in [
        ParameterTuningDecision::AcceptForBacktest,
        ParameterTuningDecision::Reject,
    ] {
        for note in [None, Some("".to_string()), Some(" 原说明 ".to_string())] {
            let fixture = Fixture::new();
            let draft = ParameterTuningDecisionDraft {
                candidate_id: Uuid::nil(),
                decision,
                decision_note: note,
            };
            assert_eq!(
                json!(decide_parameter_tuning_candidate(&fixture, draft.clone())
                    .await
                    .unwrap()),
                json!(fixture.candidate)
            );
            assert_eq!(fixture.names(), ["decide"]);
            assert_eq!(fixture.calls()[0]["data"], json!(draft));
        }
    }
}

#[tokio::test]
async fn shadow_and_promotion_ledger_queries_keep_candidate_identity_and_result() {
    let fixture = Fixture::new();
    assert!(list_parameter_shadow_validations(&fixture, Uuid::nil())
        .await
        .unwrap()
        .is_empty());
    assert_eq!(
        json!(list_parameter_promotion_decisions(&fixture, Uuid::nil())
            .await
            .unwrap()),
        json!([decision()])
    );
    assert_eq!(fixture.names(), ["shadows", "decisions"]);
    assert!(fixture
        .calls()
        .iter()
        .all(|call| call["data"] == json!({"id": Uuid::nil()})));
}

#[tokio::test]
async fn promotion_note_check_precedes_port_and_keeps_original_utf8_byte_rule() {
    for note in ["", "       ", " 1234567 "] {
        let fixture = Fixture::new();
        assert!(
            matches!(promote_parameter_candidate(&fixture, promotion(note)).await.unwrap_err(),
            ApplicationError::Validation(message) if message == "人工晋升说明至少需要 8 个字符")
        );
        assert!(fixture.calls().is_empty());
    }
    let fixture = Fixture::new();
    promote_parameter_candidate(&fixture, promotion("确认通过"))
        .await
        .unwrap();
    assert_eq!(fixture.names(), ["read", "readiness", "promote"]);
}

#[tokio::test]
async fn promotion_checks_exact_baseline_and_forwards_original_manual_request() {
    let mut fixture = Fixture::new();
    fixture.candidate.constraints = json!({"minimum_sample_size": 73});
    let request = promotion(" 12345678 ");
    assert_eq!(
        json!(promote_parameter_candidate(&fixture, request.clone())
            .await
            .unwrap()),
        json!(decision())
    );
    assert_eq!(fixture.names(), ["read", "readiness", "promote"]);
    assert_eq!(
        fixture.calls()[1]["data"],
        json!({"competition_id": fixture.candidate.competition_id, "snapshot_type": "T-1h", "minimum_sample_size": 73})
    );
    assert_eq!(fixture.calls()[2]["data"], json!(request));
}

#[tokio::test]
async fn promotion_missing_scope_and_blocked_reasons_stop_before_binding_write() {
    let mut fixture = Fixture::new();
    fixture.candidate.competition_id = None;
    assert!(
        matches!(promote_parameter_candidate(&fixture, promotion("12345678")).await.unwrap_err(),
        ApplicationError::Validation(message) if message == "候选缺少赛事范围")
    );
    assert_eq!(fixture.names(), ["read"]);
    let mut fixture = Fixture::new();
    fixture.readiness.ready_for_promotion = false;
    fixture.readiness.blocked_reasons = vec!["first".into(), " second ".into()];
    fixture.readiness.active_model_version_id = None;
    assert!(
        matches!(promote_parameter_candidate(&fixture, promotion("12345678")).await.unwrap_err(),
        ApplicationError::Validation(message) if message == "晋升门禁未通过：first； second ")
    );
    assert_eq!(fixture.names(), ["read", "readiness"]);
}

#[tokio::test]
async fn promotion_rejects_either_changed_baseline_binding_without_overwrite() {
    for model_changed in [true, false] {
        let mut fixture = Fixture::new();
        if model_changed {
            fixture.readiness.active_model_version_id = None;
        } else {
            fixture.readiness.active_parameter_set_id = None;
        }
        assert!(
            matches!(promote_parameter_candidate(&fixture, promotion("12345678")).await.unwrap_err(),
            ApplicationError::Validation(message) if message == "正式绑定已发生变化，候选必须重新校准和影子验证")
        );
        assert_eq!(fixture.names(), ["read", "readiness"]);
    }
}

#[tokio::test]
async fn rollback_requires_original_note_and_only_forwards_explicit_manual_request() {
    let fixture = Fixture::new();
    let mut request = ParameterRollbackRequest {
        candidate_id: Uuid::nil(),
        decided_by: None,
        decision_note: " 1234567 ".into(),
    };
    assert!(
        matches!(rollback_parameter_candidate(&fixture, request.clone()).await.unwrap_err(),
        ApplicationError::Validation(message) if message == "回滚说明至少需要 8 个字符")
    );
    assert!(fixture.calls().is_empty());
    request.decision_note = " 确认回滚 ".into();
    assert_eq!(
        json!(rollback_parameter_candidate(&fixture, request.clone())
            .await
            .unwrap()),
        json!(decision())
    );
    assert_eq!(fixture.names(), ["rollback"]);
    assert_eq!(fixture.calls()[0]["data"], json!(request));
}

#[tokio::test]
async fn shadow_requires_manual_queue_status_before_readiness_or_model() {
    for status in [
        "pending",
        "shadow_running",
        "shadow_passed",
        "promoted",
        "rejected",
        "accepted_for_backtest ",
    ] {
        let mut fixture = Fixture::new();
        fixture.candidate.status = status.into();
        assert!(
            matches!(run_parameter_shadow_validation(&fixture, &ModelRegistry::new(), validation()).await.unwrap_err(),
            ApplicationError::Validation(message) if message == "候选必须先由用户确认进入影子验证队列")
        );
        assert_eq!(fixture.names(), ["read"]);
    }
}

#[tokio::test]
async fn shadow_checks_five_required_identities_in_original_order_before_readiness() {
    for index in 0..5 {
        let mut fixture = Fixture::new();
        let fields = [
            &mut fixture.candidate.competition_id,
            &mut fixture.candidate.competition_profile_id,
            &mut fixture.candidate.baseline_model_version_id,
            &mut fixture.candidate.baseline_parameter_set_id,
            &mut fixture.candidate.candidate_parameter_set_id,
        ];
        *fields.into_iter().nth(index).unwrap() = None;
        let label = [
            "赛事范围",
            "赛事 Profile",
            "基线模型版本",
            "基线参数版本",
            "候选参数版本",
        ][index];
        assert!(
            matches!(run_parameter_shadow_validation(&fixture, &ModelRegistry::new(), validation()).await.unwrap_err(),
            ApplicationError::Validation(message) if message == format!("候选缺少{label}"))
        );
        assert_eq!(fixture.names(), ["read"]);
    }
}

#[tokio::test]
async fn shadow_blocked_h_records_original_report_without_replay_or_promotion() {
    for status in ["accepted_for_backtest", "blocked_by_h", "shadow_failed"] {
        let mut fixture = Fixture::new();
        fixture.candidate.status = status.into();
        fixture.readiness.ready_for_shadow_validation = false;
        fixture.readiness.h_contract_ready = false;
        fixture.readiness.blocked_reasons = vec!["original H blocker".into()];
        let result = run_parameter_shadow_validation(&fixture, &ModelRegistry::new(), validation())
            .await
            .unwrap();
        assert_eq!(fixture.names(), ["read", "readiness", "save"]);
        assert_eq!(json!(result), fixture.calls()[2]["data"]);
        assert_eq!(result.status, "blocked");
        assert_eq!(result.sample_count, 50);
        assert_eq!(result.partition_key, fixture.readiness.partition_key);
        assert_eq!(result.baseline_metrics, fixture.candidate.baseline_metrics);
        assert_eq!(
            result.gate_results,
            json!({"passed": false, "h_contract_ready": false, "blocked_reasons": ["original H blocker"], "automatic_promotion": false})
        );
        assert_eq!(result.validation_key.len(), 64);
    }
}

#[tokio::test]
async fn shadow_baseline_and_sample_guards_precede_parameter_and_provider_reads() {
    let mut fixture = Fixture::new();
    fixture.fixtures = vec![replay_fixture(20)];
    fixture.fixtures[0].baseline_home_win = 2.0;
    assert!(
        matches!(run_parameter_shadow_validation(&fixture, &ModelRegistry::new(), validation()).await.unwrap_err(),
        ApplicationError::Validation(message) if message.contains("1X2 概率非法或未归一化"))
    );
    assert_eq!(fixture.names(), ["read", "readiness", "fixtures"]);
    let mut fixture = Fixture::new();
    fixture.fixtures = vec![replay_fixture(20)];
    assert!(
        matches!(run_parameter_shadow_validation(&fixture, &ModelRegistry::new(), validation()).await.unwrap_err(),
        ApplicationError::Validation(message) if message == "精确分区样本在验证前发生变化，已拒绝继续")
    );
    assert_eq!(fixture.names(), ["read", "readiness", "fixtures"]);
    let mut fixture = Fixture::new();
    fixture.fixtures = (1..=50).rev().map(replay_fixture).collect();
    assert!(
        matches!(run_parameter_shadow_validation(&fixture, &ModelRegistry::new(), validation()).await.unwrap_err(),
        ApplicationError::ModelNotFound(key) if key == "external-provider")
    );
    assert_eq!(
        fixture.names(),
        ["read", "readiness", "fixtures", "definition"]
    );
    assert_eq!(
        fixture.calls()[2]["data"],
        json!({"competition": Uuid::from_u128(2), "profile": Uuid::from_u128(3),
        "snapshot": "T-1h", "model": Uuid::from_u128(4), "parameter": Uuid::from_u128(5)})
    );
    assert_eq!(
        fixture.calls()[3]["data"],
        json!({"id": Uuid::from_u128(7)})
    );
}

#[tokio::test]
async fn lifecycle_port_errors_preserve_all_kinds_and_stop_without_retry() {
    for kind in [
        PortErrorKind::Unavailable,
        PortErrorKind::NotFound,
        PortErrorKind::Conflict,
        PortErrorKind::InvalidState,
        PortErrorKind::Serialization,
        PortErrorKind::Infrastructure,
    ] {
        for method in ["read", "readiness", "fixtures", "definition", "save"] {
            let mut fixture = Fixture::new();
            fixture.fail_at = Some(method);
            fixture.error_kind = kind;
            fixture.fixtures = (1..=50).map(replay_fixture).collect();
            if method == "save" {
                fixture.readiness.ready_for_shadow_validation = false;
            }
            assert!(
                matches!(run_parameter_shadow_validation(&fixture, &ModelRegistry::new(), validation()).await.unwrap_err(),
                ApplicationError::Port(error) if error.kind == kind && error.message == format!("injected {method}"))
            );
            assert_eq!(fixture.names().last().unwrap(), method);
            assert_eq!(
                fixture
                    .names()
                    .iter()
                    .filter(|name| name.as_str() == method)
                    .count(),
                1
            );
        }
        let mut fixture = Fixture::new();
        fixture.fail_at = Some("promote");
        fixture.error_kind = kind;
        assert!(
            matches!(promote_parameter_candidate(&fixture, promotion("12345678")).await.unwrap_err(), ApplicationError::Port(error) if error.kind==kind)
        );
        assert_eq!(fixture.names(), ["read", "readiness", "promote"]);
        let mut fixture = Fixture::new();
        fixture.fail_at = Some("rollback");
        fixture.error_kind = kind;
        assert!(
            matches!(rollback_parameter_candidate(&fixture, ParameterRollbackRequest { candidate_id: Uuid::nil(), decided_by: None, decision_note: "12345678".into() }).await.unwrap_err(), ApplicationError::Port(error) if error.kind==kind)
        );
        assert_eq!(fixture.names(), ["rollback"]);
    }
}
