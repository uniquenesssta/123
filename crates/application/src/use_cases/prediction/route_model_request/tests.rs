use super::*;
use crate::{ApplicationError, ApplicationService};

#[test]
fn model_selection_supports_family_and_exact_ids() {
    let p4 = normalize_model_selection("p4").expect("P4 系列必须有效");
    assert_eq!(p4.family, "p4");
    assert!(p4.exact_model_id.is_none());

    let p7 = normalize_model_selection("P7_KNOCKOUT_90").expect("P7 精确模型必须有效");
    assert_eq!(p7.family, "p7");
    assert_eq!(p7.exact_model_id.as_deref(), Some("p7_knockout_90"));
}

#[test]
fn exact_model_must_exist_in_registry() {
    let service = ApplicationService::new();
    let registered = normalize_model_selection("p7_league").expect("内置模型必须有效");
    ensure_model_selection_registered(&service.registry, &registered).expect("已注册模型必须通过");

    let missing = normalize_model_selection("p7_not_registered").expect("格式合法");
    assert!(matches!(
        ensure_model_selection_registered(&service.registry, &missing),
        Err(ApplicationError::ModelNotFound(_))
    ));
}

use crate::use_cases::prediction::tests::{command_fixture, probe_registry, route_fixture, Probe};
use crate::{p4_default_match, p4_default_parameters, RoutePreviewCommand};
use football_domain::{CompetitionKind, ResolvedCompetitionContext};
use serde_json::{json, Value};
use std::sync::Arc;
use uuid::Uuid;

fn scope_fixture() -> ResolvedCompetitionContext {
    ResolvedCompetitionContext {
        competition_id: Some(Uuid::from_u128(31)),
        season_id: Some(Uuid::from_u128(32)),
        stage_id: Some(Uuid::from_u128(33)),
        competition_kind: CompetitionKind::League,
    }
}

#[test]
fn selection_defaults_whitespace_case_and_errors_keep_existing_policy() {
    let registry = crate::model_registry::ModelRegistry::new();
    for (raw, family, exact) in [
        (" \t", "p4", None),
        (" P7 ", "p7", None),
        (" P4_LEAGUE ", "p4", Some("p4_league")),
        ("p7_", "p7", Some("p7_")),
    ] {
        let selection = normalize_model_selection(raw).unwrap();
        assert_eq!(selection.family, family);
        assert_eq!(selection.exact_model_id.as_deref(), exact);
        if exact.is_none() {
            ensure_model_selection_registered(&registry, &selection).unwrap();
        } else {
            assert!(matches!(
                ensure_model_selection_registered(&registry, &selection),
                Err(ApplicationError::ModelNotFound(_))
            ));
        }
    }
    for raw in ["p40", "p7-other", " P8 "] {
        assert!(
            matches!(normalize_model_selection(raw), Err(ApplicationError::Validation(message)) if message == format!("不支持的模型：{}；请选择已注册的 P4 或 P7 模型", raw.trim().to_ascii_lowercase()))
        );
    }
}

#[test]
fn manual_context_preserves_utc_scope_names_and_input_identity() {
    let mut command = command_fixture();
    command.match_input = json!({"kickoff_time": "2026-09-30T20:01:59.123456789+08:00", "team_a": {"name": " abc-def_1234567890 "}, "team_b": {"name": "中文"}});
    let scope = scope_fixture();
    let context = match_context_from_command(&command, &scope).unwrap();
    assert_eq!(context.match_key, "SIM-20260930T1201Z-ABCDEF123456-TEAM");
    assert_eq!(
        context.kickoff_time.to_rfc3339(),
        "2026-09-30T12:01:59.123456789+00:00"
    );
    assert_eq!(context.home_team_name, " abc-def_1234567890 ");
    assert_eq!(context.away_team_name, "中文");
    assert_eq!(context.competition_id, scope.competition_id);
    assert_eq!(context.season_id, scope.season_id);
    assert_eq!(context.stage_id, scope.stage_id);
    assert_eq!(
        context.metadata,
        json!({"routing_mode": "automatic", "requested_model_family": "p4", "requested_model_id": "p4_league"})
    );
    command.match_input["match_id"] = json!("  existing-key  ");
    let context = match_context_from_command(&command, &scope).unwrap();
    assert_eq!(context.match_key, "existing-key");
    let request = build_model_request(command.clone(), context, &route_fixture()).unwrap();
    assert_eq!(request.input, command.match_input);
    for invalid_id in [Value::Null, json!(17), json!(" \t")] {
        let mut input = command.match_input.clone();
        input["match_id"] = invalid_id;
        assert_eq!(
            ensure_match_input_id(input, "generated").unwrap()["match_id"],
            "generated"
        );
    }
    assert!(
        matches!(ensure_match_input_id(json!([]), "generated"), Err(ApplicationError::Model(message)) if message == "模型输入必须是 JSON 对象")
    );
}

#[test]
fn context_failure_priority_keeps_input_fields_before_model_selection() {
    let mut command = command_fixture();
    command.model_family = "invalid".into();
    command.match_input = json!({});
    assert!(
        matches!(match_context_from_command(&command, &scope_fixture()), Err(ApplicationError::Model(message)) if message == "缺少字符串字段：kickoff_time")
    );
    command.match_input["kickoff_time"] = json!("invalid");
    assert!(matches!(
        match_context_from_command(&command, &scope_fixture()),
        Err(ApplicationError::InvalidKickoff(_))
    ));
    command.match_input["kickoff_time"] = json!("2026-09-30T12:00:00Z");
    assert!(
        matches!(match_context_from_command(&command, &scope_fixture()), Err(ApplicationError::Model(message)) if message == "缺少字符串字段：team_a.name")
    );
    command.match_input["team_a"] = json!({"name": "Home"});
    assert!(
        matches!(match_context_from_command(&command, &scope_fixture()), Err(ApplicationError::Model(message)) if message == "缺少字符串字段：team_b.name")
    );
    command.match_input["team_b"] = json!({"name": "Away"});
    assert!(matches!(
        match_context_from_command(&command, &scope_fixture()),
        Err(ApplicationError::Validation(_))
    ));
}

#[test]
fn explicit_route_override_and_request_keep_routed_identity_and_parameters() {
    let mut command = command_fixture();
    let mut scope = scope_fixture();
    scope.competition_kind = CompetitionKind::Custom;
    let decision = route_fixture();
    let mut context = match_context_from_command(&command, &scope).unwrap();
    let original = serde_json::to_value(&context).unwrap();
    assert!(
        matches!(apply_route_context(&command, &mut context, &scope, &decision), Err(ApplicationError::Validation(message)) if message == "自动规则包赛事类型 league 与当前赛事类型 custom 不一致")
    );
    assert_eq!(serde_json::to_value(&context).unwrap(), original);
    command.explicit_rule_package_id = Some(decision.rule_package_id);
    let mut context = match_context_from_command(&command, &scope).unwrap();
    apply_route_context(&command, &mut context, &scope, &decision).unwrap();
    assert_eq!(context.competition_kind, CompetitionKind::League);
    assert_eq!(context.metadata["routing_mode"], "explicit_rule_package");
    assert_eq!(
        context.metadata["explicit_competition_kind_override"],
        json!({"catalog_kind": "custom", "rule_package_kind": "league"})
    );
    let request = build_model_request(command.clone(), context, &decision).unwrap();
    assert_eq!(request.identity.model_id, decision.model_id);
    assert_eq!(request.identity.model_version, decision.model_version);
    assert_eq!(
        request.identity.parameter_version,
        decision.parameter_version
    );
    assert_eq!(
        request.identity.rule_package_version,
        Some(decision.package_version)
    );
    assert_eq!(request.parameters, decision.parameters);
    assert_eq!(request.snapshot_type, command.snapshot_type);
    assert_eq!(request.input["match_id"], request.context.match_key);
    assert_eq!(request.input["team_a"], command.match_input["team_a"]);
    // Non-object metadata was previously left untouched by an explicit override.
    let mut context = request.context;
    context.metadata = Value::Null;
    apply_route_context(&command, &mut context, &scope, &route_fixture()).unwrap();
    assert!(context.metadata.is_null());
}

#[test]
fn snapshot_and_audited_route_preserve_exact_membership_and_all_identity_fields() {
    let decision = route_fixture();
    validate_snapshot_type("T-1h", &decision.routing).unwrap();
    for raw in ["", " ", " T-1h", "T-1h ", "t-1h"] {
        assert!(matches!(
            validate_snapshot_type(raw, &decision.routing),
            Err(ApplicationError::Validation(_))
        ));
    }
    let mut routing = decision.routing.clone();
    routing.supported_snapshot_types.clear();
    validate_snapshot_type(" arbitrary ", &routing).unwrap();
    assert!(validate_snapshot_type(" ", &routing).is_err());
    let identity = route_identity_manifest(&decision);
    assert_eq!(identity.as_object().unwrap().len(), 11);
    for absent in [
        json!({}),
        json!({"input_audit": null}),
        json!({"input_audit": {"manifest": {}}}),
    ] {
        verify_route_identity_matches_input_audit(&decision, &absent).unwrap();
    }
    let input = json!({"input_audit": {"manifest": {"route_identity": identity}}});
    verify_route_identity_matches_input_audit(&decision, &input).unwrap();
    for key in [
        "source",
        "binding_id",
        "rule_package_id",
        "rule_package_key",
        "rule_package_version",
        "model_id",
        "model_version_id",
        "model_version",
        "parameter_set_id",
        "parameter_version",
        "competition_profile_id",
    ] {
        let mut changed = input.clone();
        changed["input_audit"]["manifest"]["route_identity"][key] = json!("changed");
        assert!(
            matches!(verify_route_identity_matches_input_audit(&decision, &changed), Err(ApplicationError::Validation(message)) if message == "模型、参数或规则路由在完整度检查后发生变化，请重新检查后再运行"),
            "{key}"
        );
    }
    assert!(verify_route_identity_matches_input_audit(
        &decision,
        &json!({"input_audit": {"manifest": {"route_identity": null}}})
    )
    .is_err());
}

#[test]
fn default_fixture_request_keeps_public_shell_payload_and_no_route_identity() {
    let request = default_fixture_request().unwrap();
    let input = p4_default_match();
    let parameters = p4_default_parameters();
    assert_eq!(request.input, input);
    assert_eq!(request.parameters, parameters);
    assert_eq!(
        request.context.match_key,
        input["match_id"].as_str().unwrap()
    );
    assert_eq!(request.context.competition_kind, CompetitionKind::Custom);
    assert!(request.context.competition_id.is_none());
    assert!(request.context.season_id.is_none());
    assert!(request.context.stage_id.is_none());
    assert!(request.context.metadata.is_null());
    assert_eq!(request.identity.model_id, crate::model_shell::P4_MODEL_ID);
    assert_eq!(
        request.identity.model_version,
        parameters["model_version"].as_str().unwrap()
    );
    assert_eq!(
        request.identity.parameter_version,
        parameters["parameter_version"].as_str().unwrap()
    );
    assert!(request.identity.rule_package_version.is_none());
    assert_eq!(request.snapshot_type, "T-1h");
}

fn preview_fixture() -> RoutePreviewCommand {
    let command = command_fixture();
    RoutePreviewCommand {
        kickoff_time: "2026-09-30T20:00:00+08:00".into(),
        competition_id: command.competition_id,
        season_id: command.season_id,
        stage_id: command.stage_id,
        competition_kind: command.competition_kind,
        model_family: command.model_family,
        explicit_rule_package_id: command.explicit_rule_package_id,
    }
}

#[tokio::test]
async fn route_preview_is_read_only_and_preserves_resolution_error_order() {
    for (kickoff, family, failure, expected) in [
        ("bad", "bad", Some("scope"), vec![]),
        ("2026-09-30T12:00:00Z", "bad", Some("scope"), vec!["scope"]),
        ("2026-09-30T12:00:00Z", "bad", None, vec!["scope"]),
        ("2026-09-30T12:00:00Z", "p4_missing", None, vec!["scope"]),
        (
            "2026-09-30T12:00:00Z",
            "p4",
            Some("route"),
            vec!["scope", "route"],
        ),
    ] {
        let port = Arc::new(Probe::new());
        port.fail_at(failure);
        let mut command = preview_fixture();
        command.kickoff_time = kickoff.into();
        command.model_family = family.into();
        let error =
            super::super::preview_route::execute(port.as_ref(), &probe_registry(&port), command)
                .await
                .unwrap_err();
        match (kickoff, family, failure) {
            ("bad", _, _) => assert!(matches!(error, ApplicationError::InvalidKickoff(_))),
            (_, _, Some(_)) => assert!(matches!(error, ApplicationError::Port(_))),
            (_, "p4_missing", _) => assert!(matches!(error, ApplicationError::ModelNotFound(_))),
            _ => assert!(matches!(error, ApplicationError::Validation(_))),
        }
        assert_eq!(port.calls(), expected);
    }
    let port = Arc::new(Probe::new());
    let result = super::super::preview_route::execute(
        port.as_ref(),
        &probe_registry(&port),
        preview_fixture(),
    )
    .await
    .unwrap();
    assert_eq!(result.rule_package_id, route_fixture().rule_package_id);
    assert_eq!(port.calls(), ["scope", "route"]);
    let state = port.state.lock().unwrap();
    let request = &state.route_requests[0];
    assert_eq!(request.preferred_model_family.as_deref(), Some("p4"));
    assert_eq!(request.preferred_model_id.as_deref(), Some("p4_league"));
    assert_eq!(
        request.kickoff_time.to_rfc3339(),
        "2026-09-30T12:00:00+00:00"
    );
    assert_eq!(request.season_id, Some(Uuid::from_u128(22)));
    assert_eq!(request.stage_id, Some(Uuid::from_u128(23)));
}

#[tokio::test]
async fn preview_and_executor_preserve_explicit_kind_policy_before_later_checks() {
    for explicit in [false, true] {
        let port = Arc::new(Probe::new());
        port.state.lock().unwrap().scope_kind = Some(CompetitionKind::Custom);
        let registry = probe_registry(&port);
        let mut preview = preview_fixture();
        preview.explicit_rule_package_id = explicit.then_some(Uuid::from_u128(10));
        let result = super::super::preview_route::execute(port.as_ref(), &registry, preview).await;
        if explicit {
            assert!(result.is_ok());
        } else {
            assert!(matches!(result, Err(ApplicationError::Validation(_))));
        }
        assert_eq!(port.calls(), ["scope", "route"]);
        let mut command = command_fixture();
        command.explicit_rule_package_id = explicit.then_some(Uuid::from_u128(10));
        if !explicit {
            command.snapshot_type = "invalid".into();
        }
        let result = super::super::execute_prediction::execute_internal(
            port.as_ref(),
            &registry,
            command,
            false,
        )
        .await;
        if explicit {
            assert!(result.unwrap().run_id.is_nil());
            assert_eq!(
                port.calls(),
                [
                    "scope",
                    "route",
                    "scope",
                    "route",
                    "model_supports",
                    "predict"
                ]
            );
        } else {
            assert!(
                matches!(result, Err(ApplicationError::Validation(message)) if message == "自动规则包赛事类型 league 与当前赛事类型 custom 不一致")
            );
            assert_eq!(port.calls(), ["scope", "route", "scope", "route"]);
        }
    }
}
