use super::{
    p4_default_match, p4_default_parameters, ApplicationError, ApplicationResult,
    ApplicationService, PredictionCommand, PredictionExecution, RoutePreviewCommand,
    StoredMatchPredictionCommand,
};
use crate::model_shell::P4_MODEL_ID;
use crate::{ModelRunListItem, PersistenceError};
use chrono::{DateTime, Utc};
use football_domain::{
    CompetitionKind, MatchContext, MatchLineupChain, MatchPredictionReadiness, ModelIdentity,
    PredictionInputAuditSummary, PredictionReadinessCheck, PredictionReadinessCheckStatus,
    PredictionReadinessLevel, ResolvedCompetitionContext, RouteDecision, RouteRequest, RuleRouting,
    PREDICTION_INPUT_AUDIT_VERSION,
};
use football_model_api::{ModelOutput, ModelRequest};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::time::Instant;
use uuid::Uuid;

impl ApplicationService {
    pub async fn execute_prediction(
        &self,
        command: PredictionCommand,
    ) -> ApplicationResult<PredictionExecution> {
        self.execute_prediction_internal(command, true).await
    }

    async fn execute_prediction_internal(
        &self,
        command: PredictionCommand,
        persist_run: bool,
    ) -> ApplicationResult<PredictionExecution> {
        let store = self.active_store().await?;
        let scope = store
            .resolve_competition_context(
                command.competition_id,
                command.season_id,
                command.stage_id,
                command.competition_kind,
            )
            .await?;
        let mut context = match_context_from_command(&command, &scope)?;
        let model_selection = normalize_model_selection(&command.model_family)?;
        ensure_model_selection_registered(&self.registry, &model_selection)?;
        let decision = store
            .resolve_route(&RouteRequest {
                competition_id: scope.competition_id,
                season_id: scope.season_id,
                stage_id: scope.stage_id,
                competition_kind: scope.competition_kind,
                kickoff_time: context.kickoff_time,
                preferred_model_family: Some(model_selection.family.to_string()),
                preferred_model_id: model_selection.exact_model_id.clone(),
                explicit_rule_package_id: command.explicit_rule_package_id,
            })
            .await?;
        if command.explicit_rule_package_id.is_some() {
            context.competition_kind = decision.competition_profile.competition_kind;
            if let Some(metadata) = context.metadata.as_object_mut() {
                metadata.insert(
                    "explicit_competition_kind_override".to_string(),
                    json!({
                        "catalog_kind": scope.competition_kind.as_str(),
                        "rule_package_kind": decision.competition_profile.competition_kind.as_str(),
                    }),
                );
            }
        } else if decision.competition_profile.competition_kind != scope.competition_kind {
            return Err(ApplicationError::Validation(format!(
                "自动规则包赛事类型 {} 与当前赛事类型 {} 不一致",
                decision.competition_profile.competition_kind.as_str(),
                scope.competition_kind.as_str()
            )));
        }
        validate_snapshot_type(&command.snapshot_type, &decision.routing)?;
        verify_route_identity_matches_input_audit(&decision, &command.match_input)?;
        let model = self
            .registry
            .get(&decision.model_id)
            .ok_or_else(|| ApplicationError::ModelNotFound(decision.model_id.clone()))?;
        if !model.supports(&context) {
            return Err(ApplicationError::Model(format!(
                "模型 {} 不支持赛事类型 {}",
                model.descriptor().display_name,
                scope.competition_kind.as_str()
            )));
        }

        let match_input = ensure_match_input_id(command.match_input, &context.match_key)?;
        let request = ModelRequest {
            context,
            identity: ModelIdentity {
                model_id: decision.model_id.clone(),
                model_version: decision.model_version.clone(),
                parameter_version: decision.parameter_version.clone(),
                rule_package_version: Some(decision.package_version.clone()),
            },
            snapshot_type: command.snapshot_type,
            input: match_input,
            parameters: decision.parameters.clone(),
        };
        let input_sha256 = sha256_value(&request.input)?;
        let input_audit = prediction_input_audit_summary(&request.input, &input_sha256)?;
        let started = Instant::now();
        let output = model
            .predict(&request)
            .map_err(|error| ApplicationError::Model(error.to_string()))?;
        let duration_ms = started.elapsed().as_millis().min(i64::MAX as u128) as i64;
        let run_id = if persist_run {
            store
                .save_successful_run(&decision, &request, &output, duration_ms)
                .await?
        } else {
            Uuid::nil()
        };
        Ok(PredictionExecution {
            run_id,
            duration_ms,
            route: decision,
            output,
            input_audit,
        })
    }

    pub async fn inspect_match_prediction_readiness(
        &self,
        command: StoredMatchPredictionCommand,
    ) -> ApplicationResult<MatchPredictionReadiness> {
        let assessed_at = Utc::now();
        let model_selection = normalize_model_selection(&command.model_family)?;
        ensure_model_selection_registered(&self.registry, &model_selection)?;
        let store = self.active_store().await?;
        let match_record = store.read_match(command.match_id).await?;
        let mut checks = Vec::<PredictionReadinessCheck>::new();
        let mut shadow_reasons = Vec::<String>::new();
        let mut route_identity = None;
        let mut prepared_input = None;
        let mut data_cutoff_at = None;

        let identity_details = if match_record.home_team_id == match_record.away_team_id {
            vec!["主客队不能是同一球队".to_string()]
        } else {
            Vec::new()
        };
        checks.push(readiness_check(
            ("match_identity", "比赛身份"),
            if identity_details.is_empty() {
                PredictionReadinessCheckStatus::Passed
            } else {
                PredictionReadinessCheckStatus::Blocked
            },
            10,
            if identity_details.is_empty() { 10 } else { 0 },
            if identity_details.is_empty() {
                "比赛、主客队与开球时间已确定"
            } else {
                "比赛身份不满足正式推演条件"
            },
            identity_details,
            json!({
                "match_key": match_record.external_key,
                "home_team_id": match_record.home_team_id,
                "away_team_id": match_record.away_team_id,
                "kickoff_time": match_record.kickoff_time,
                "status": match_record.status,
            }),
        ));

        let lineup_chain = match store
            .read_match_lineup_chain_at(command.match_id, &command.snapshot_type, assessed_at)
            .await
        {
            Ok(chain) => {
                data_cutoff_at = Some(chain.data_cutoff_time);
                checks.push(readiness_check(
                    ("data_window", "赛前数据窗口"),
                    PredictionReadinessCheckStatus::Passed,
                    10,
                    10,
                    "所选时间窗口已经开启，且截止时间早于开球",
                    Vec::new(),
                    json!({
                        "snapshot_type": chain.snapshot_type,
                        "window_start": chain.data_window_start_time,
                        "data_cutoff_time": chain.data_cutoff_time,
                    }),
                ));
                append_lineup_readiness_checks(&mut checks, &chain);
                Some(chain)
            }
            Err(PersistenceError::InvalidState(message)) => {
                checks.push(readiness_check(
                    ("data_window", "赛前数据窗口"),
                    PredictionReadinessCheckStatus::Blocked,
                    10,
                    0,
                    "所选时间窗口当前不可用",
                    vec![message.clone()],
                    json!({"snapshot_type": command.snapshot_type}),
                ));
                append_unavailable_lineup_readiness_checks(&mut checks, &message);
                None
            }
            Err(error) => return Err(error.into()),
        };

        let scope = store
            .resolve_competition_context(
                match_record.competition_id,
                match_record.season_id,
                match_record.stage_id,
                CompetitionKind::Custom,
            )
            .await?;
        let route_result = store
            .resolve_route(&RouteRequest {
                competition_id: scope.competition_id,
                season_id: scope.season_id,
                stage_id: scope.stage_id,
                competition_kind: scope.competition_kind,
                kickoff_time: match_record.kickoff_time,
                preferred_model_family: Some(model_selection.family.to_string()),
                preferred_model_id: model_selection.exact_model_id.clone(),
                explicit_rule_package_id: command.explicit_rule_package_id,
            })
            .await;
        match route_result {
            Ok(decision) => {
                let route_validation = (|| -> ApplicationResult<()> {
                    if command.explicit_rule_package_id.is_none()
                        && decision.competition_profile.competition_kind != scope.competition_kind
                    {
                        return Err(ApplicationError::Validation(format!(
                            "自动规则包赛事类型 {} 与当前赛事类型 {} 不一致",
                            decision.competition_profile.competition_kind.as_str(),
                            scope.competition_kind.as_str()
                        )));
                    }
                    validate_snapshot_type(&command.snapshot_type, &decision.routing)?;
                    let model = self.registry.get(&decision.model_id).ok_or_else(|| {
                        ApplicationError::ModelNotFound(decision.model_id.clone())
                    })?;
                    let context = MatchContext {
                        match_key: match_record.external_key.clone(),
                        kickoff_time: match_record.kickoff_time,
                        competition_id: match_record.competition_id,
                        season_id: match_record.season_id,
                        stage_id: match_record.stage_id,
                        competition_kind: scope.competition_kind,
                        home_team_name: match_record.home_team_name.clone(),
                        away_team_name: match_record.away_team_name.clone(),
                        metadata: Value::Null,
                    };
                    if !model.supports(&context) {
                        return Err(ApplicationError::Model(format!(
                            "模型 {} 不支持赛事类型 {}",
                            model.descriptor().display_name,
                            scope.competition_kind.as_str()
                        )));
                    }
                    Ok(())
                })();
                match route_validation {
                    Ok(()) => {
                        route_identity = Some(route_identity_manifest(&decision));
                        checks.push(readiness_check(
                            ("model_route", "模型与规则路由"),
                            PredictionReadinessCheckStatus::Passed,
                            15,
                            15,
                            "生产模型、参数与规则包已形成唯一可追溯路由",
                            Vec::new(),
                            route_identity.clone().unwrap_or(Value::Null),
                        ));
                    }
                    Err(error) => checks.push(readiness_check(
                        ("model_route", "模型与规则路由"),
                        PredictionReadinessCheckStatus::Blocked,
                        15,
                        0,
                        "模型或规则包路由不满足正式推演条件",
                        vec![error.to_string()],
                        Value::Null,
                    )),
                }
            }
            Err(PersistenceError::RouteNotFound) => checks.push(readiness_check(
                ("model_route", "模型与规则路由"),
                PredictionReadinessCheckStatus::Blocked,
                15,
                0,
                "没有匹配到可用的生产模型与规则包",
                vec!["请检查赛事类型、模型系列和生产规则绑定".to_string()],
                Value::Null,
            )),
            Err(error) => return Err(error.into()),
        }

        if lineup_chain
            .as_ref()
            .is_some_and(|chain| chain.ready_for_model)
        {
            match store
                .prepare_match_prediction_input_at(
                    command.match_id,
                    &command.snapshot_type,
                    model_selection.family,
                    assessed_at,
                )
                .await
            {
                Ok(prepared) => {
                    append_prepared_input_checks(&mut checks, &mut shadow_reasons, &prepared);
                    prepared_input = Some(prepared);
                }
                Err(PersistenceError::InvalidState(message)) => {
                    append_unavailable_prepared_input_checks(
                        &mut checks,
                        "模型输入构建失败，暂时无法评估球队历史样本",
                        &message,
                    );
                }
                Err(error) => return Err(error.into()),
            }
        } else {
            append_unavailable_prepared_input_checks(
                &mut checks,
                "阵容或时间窗口尚未通过，暂时无法评估球队历史样本",
                "阵容或时间窗口尚未通过，不能构建正式输入",
            );
        }

        let blockers = checks
            .iter()
            .filter(|check| check.status == PredictionReadinessCheckStatus::Blocked)
            .flat_map(|check| {
                if check.details.is_empty() {
                    vec![format!("{}：{}", check.label, check.summary)]
                } else {
                    check
                        .details
                        .iter()
                        .map(|detail| format!("{}：{detail}", check.label))
                        .collect()
                }
            })
            .collect::<Vec<_>>();
        let mut warnings = checks
            .iter()
            .filter(|check| check.status == PredictionReadinessCheckStatus::Warning)
            .flat_map(|check| {
                if check.details.is_empty() {
                    vec![format!("{}：{}", check.label, check.summary)]
                } else {
                    check
                        .details
                        .iter()
                        .map(|detail| format!("{}：{detail}", check.label))
                        .collect()
                }
            })
            .collect::<Vec<_>>();
        for reason in &shadow_reasons {
            if !warnings.contains(reason) {
                warnings.push(reason.clone());
            }
        }
        let score = checks
            .iter()
            .map(|check| u16::from(check.score))
            .sum::<u16>()
            .min(100) as u8;
        let level = if !blockers.is_empty() {
            PredictionReadinessLevel::Blocked
        } else if !shadow_reasons.is_empty() {
            PredictionReadinessLevel::ShadowOnly
        } else if !warnings.is_empty() {
            PredictionReadinessLevel::ReadyWithWarnings
        } else {
            PredictionReadinessLevel::FormalReady
        };
        let input_manifest = prepared_input.as_ref().map(|prepared| {
            build_prediction_input_manifest(
                &prepared.match_input,
                &prepared.data_quality,
                &match_record,
                &command.snapshot_type,
                route_identity.as_ref(),
            )
        });
        let input_manifest_sha256 = input_manifest.as_ref().map(sha256_value).transpose()?;

        Ok(MatchPredictionReadiness {
            audit_version: PREDICTION_INPUT_AUDIT_VERSION.to_string(),
            match_id: match_record.id,
            match_key: match_record.external_key,
            snapshot_type: command.snapshot_type,
            model_family: command.model_family,
            assessed_at,
            data_cutoff_at,
            level,
            score,
            can_run_formal: level.can_run_formal(),
            can_run_shadow: level.can_run_shadow(),
            blockers,
            warnings,
            checks,
            input_manifest,
            input_manifest_sha256,
            route_identity,
        })
    }

    pub async fn execute_prediction_from_match(
        &self,
        command: StoredMatchPredictionCommand,
    ) -> ApplicationResult<PredictionExecution> {
        self.execute_prediction_from_match_with_mode(command, true)
            .await
    }

    pub async fn execute_shadow_prediction_from_match(
        &self,
        command: StoredMatchPredictionCommand,
    ) -> ApplicationResult<PredictionExecution> {
        self.execute_prediction_from_match_with_mode(command, false)
            .await
    }

    async fn execute_prediction_from_match_with_mode(
        &self,
        command: StoredMatchPredictionCommand,
        persist_run: bool,
    ) -> ApplicationResult<PredictionExecution> {
        let readiness = self
            .inspect_match_prediction_readiness(command.clone())
            .await?;
        let allowed = if persist_run {
            readiness.can_run_formal
        } else {
            readiness.can_run_shadow
        };
        if !allowed {
            let reasons = if readiness.blockers.is_empty() {
                readiness.warnings.join("；")
            } else {
                readiness.blockers.join("；")
            };
            let mode = if persist_run { "正式" } else { "影子" };
            return Err(ApplicationError::Validation(format!(
                "赛前数据完整度门禁未允许{mode}推演（{}，{} 分）：{}",
                readiness.level.as_str(),
                readiness.score,
                reasons
            )));
        }
        let model_family = normalize_model_selection(&command.model_family)?
            .family
            .to_string();
        let store = self.active_store().await?;
        let mut prepared = store
            .prepare_match_prediction_input_at(
                command.match_id,
                &command.snapshot_type,
                &model_family,
                readiness.assessed_at,
            )
            .await?;
        verify_prepared_input_matches_readiness(&prepared, &readiness)?;
        attach_prediction_input_audit(&mut prepared.match_input, &readiness)?;
        self.execute_prediction_internal(
            PredictionCommand {
                match_input: prepared.match_input,
                snapshot_type: prepared.snapshot_type,
                competition_id: prepared.match_record.competition_id,
                season_id: prepared.match_record.season_id,
                stage_id: prepared.match_record.stage_id,
                competition_kind: prepared.competition_kind,
                model_family: command.model_family,
                explicit_rule_package_id: command.explicit_rule_package_id,
            },
            persist_run,
        )
        .await
    }

    pub async fn preview_route(
        &self,
        command: RoutePreviewCommand,
    ) -> ApplicationResult<RouteDecision> {
        let kickoff_time = parse_kickoff(&command.kickoff_time)?;
        let store = self.active_store().await?;
        let scope = store
            .resolve_competition_context(
                command.competition_id,
                command.season_id,
                command.stage_id,
                command.competition_kind,
            )
            .await?;
        let model_selection = normalize_model_selection(&command.model_family)?;
        ensure_model_selection_registered(&self.registry, &model_selection)?;
        let decision = store
            .resolve_route(&RouteRequest {
                competition_id: scope.competition_id,
                season_id: scope.season_id,
                stage_id: scope.stage_id,
                competition_kind: scope.competition_kind,
                kickoff_time,
                preferred_model_family: Some(model_selection.family.to_string()),
                preferred_model_id: model_selection.exact_model_id.clone(),
                explicit_rule_package_id: command.explicit_rule_package_id,
            })
            .await?;
        if command.explicit_rule_package_id.is_none()
            && decision.competition_profile.competition_kind != scope.competition_kind
        {
            return Err(ApplicationError::Validation(format!(
                "自动规则包赛事类型 {} 与当前赛事类型 {} 不一致",
                decision.competition_profile.competition_kind.as_str(),
                scope.competition_kind.as_str()
            )));
        }
        Ok(decision)
    }
    pub fn dry_run_default_fixture(&self) -> ApplicationResult<ModelOutput> {
        let model = self
            .registry
            .get(P4_MODEL_ID)
            .ok_or_else(|| ApplicationError::ModelNotFound(P4_MODEL_ID.to_string()))?;
        let parameters = p4_default_parameters();
        let match_input = p4_default_match();
        let request = ModelRequest {
            context: MatchContext {
                match_key: required_string(&match_input, "match_id")?,
                kickoff_time: parse_kickoff(&required_string(&match_input, "kickoff_time")?)?,
                competition_id: None,
                season_id: None,
                stage_id: None,
                competition_kind: CompetitionKind::Custom,
                home_team_name: nested_required_string(&match_input, "team_a", "name")?,
                away_team_name: nested_required_string(&match_input, "team_b", "name")?,
                metadata: Value::Null,
            },
            identity: ModelIdentity {
                model_id: P4_MODEL_ID.to_string(),
                model_version: required_string(&parameters, "model_version")?,
                parameter_version: required_string(&parameters, "parameter_version")?,
                rule_package_version: None,
            },
            snapshot_type: "T-1h".to_string(),
            input: match_input,
            parameters,
        };
        model
            .predict(&request)
            .map_err(|error| ApplicationError::Model(error.to_string()))
    }

    pub async fn list_recent_runs(&self, limit: i64) -> ApplicationResult<Vec<ModelRunListItem>> {
        let store = self.active_store().await?;
        Ok(store.list_recent_runs(limit).await?)
    }

    pub async fn hide_run_from_history(
        &self,
        run_id: Uuid,
        reason: Option<String>,
    ) -> ApplicationResult<()> {
        let store = self.active_store().await?;
        Ok(store
            .hide_run_from_history(run_id, reason.as_deref())
            .await?)
    }

    pub async fn read_run(&self, run_id: Uuid) -> ApplicationResult<Value> {
        let store = self.active_store().await?;
        Ok(store.read_run(run_id).await?)
    }
}

fn match_context_from_command(
    command: &PredictionCommand,
    scope: &ResolvedCompetitionContext,
) -> ApplicationResult<MatchContext> {
    let kickoff_time = parse_kickoff(&required_string(&command.match_input, "kickoff_time")?)?;
    let home_team_name = nested_required_string(&command.match_input, "team_a", "name")?;
    let away_team_name = nested_required_string(&command.match_input, "team_b", "name")?;
    let match_key = command
        .match_input
        .get("match_id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| {
            format!(
                "SIM-{}-{}-{}",
                kickoff_time.format("%Y%m%dT%H%MZ"),
                compact_key_part(&home_team_name),
                compact_key_part(&away_team_name)
            )
        });
    let model_selection = normalize_model_selection(&command.model_family)?;
    Ok(MatchContext {
        match_key,
        kickoff_time,
        competition_id: scope.competition_id,
        season_id: scope.season_id,
        stage_id: scope.stage_id,
        competition_kind: scope.competition_kind,
        home_team_name,
        away_team_name,
        metadata: json!({
            "routing_mode": if command.explicit_rule_package_id.is_some() { "explicit_rule_package" } else { "automatic" },
            "requested_model_family": model_selection.family,
            "requested_model_id": model_selection.exact_model_id,
        }),
    })
}

pub(super) fn ensure_match_input_id(mut input: Value, match_key: &str) -> ApplicationResult<Value> {
    let object = input
        .as_object_mut()
        .ok_or_else(|| ApplicationError::Model("模型输入必须是 JSON 对象".to_string()))?;
    let has_match_id = object
        .get("match_id")
        .and_then(Value::as_str)
        .is_some_and(|value| !value.trim().is_empty());
    if !has_match_id {
        object.insert("match_id".to_string(), Value::String(match_key.to_string()));
    }
    Ok(input)
}

fn compact_key_part(value: &str) -> String {
    let normalized: String = value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .map(|ch| ch.to_ascii_uppercase())
        .take(12)
        .collect();
    if normalized.is_empty() {
        "TEAM".to_string()
    } else {
        normalized
    }
}

fn validate_snapshot_type(snapshot_type: &str, routing: &RuleRouting) -> ApplicationResult<()> {
    if snapshot_type.trim().is_empty() {
        return Err(ApplicationError::Validation("快照类型不能为空".to_string()));
    }
    if !routing.supported_snapshot_types.is_empty()
        && !routing
            .supported_snapshot_types
            .iter()
            .any(|item| item == snapshot_type)
    {
        return Err(ApplicationError::Validation(format!(
            "规则包不支持快照类型 {snapshot_type}"
        )));
    }
    Ok(())
}

fn route_identity_manifest(decision: &RouteDecision) -> Value {
    json!({
        "source": decision.source,
        "binding_id": decision.binding_id,
        "rule_package_id": decision.rule_package_id,
        "rule_package_key": decision.package_key,
        "rule_package_version": decision.package_version,
        "model_id": decision.model_id,
        "model_version_id": decision.model_version_id,
        "model_version": decision.model_version,
        "parameter_set_id": decision.parameter_set_id,
        "parameter_version": decision.parameter_version,
        "competition_profile_id": decision.competition_profile_id,
    })
}

fn verify_route_identity_matches_input_audit(
    decision: &RouteDecision,
    input: &Value,
) -> ApplicationResult<()> {
    let Some(expected) = input
        .get("input_audit")
        .and_then(|audit| audit.get("manifest"))
        .and_then(|manifest| manifest.get("route_identity"))
    else {
        return Ok(());
    };
    let actual = route_identity_manifest(decision);
    if expected != &actual {
        return Err(ApplicationError::Validation(
            "模型、参数或规则路由在完整度检查后发生变化，请重新检查后再运行".to_string(),
        ));
    }
    Ok(())
}

fn readiness_check(
    (code, label): (&str, &str),
    status: PredictionReadinessCheckStatus,
    weight: u8,
    score: u8,
    summary: &str,
    details: Vec<String>,
    metadata: Value,
) -> PredictionReadinessCheck {
    PredictionReadinessCheck {
        code: code.to_string(),
        label: label.to_string(),
        status,
        weight,
        score: score.min(weight),
        summary: summary.to_string(),
        details,
        metadata,
    }
}

fn selected_lineup(
    chain: &football_domain::MatchLineupTeamChain,
) -> Option<&football_domain::LineupRecord> {
    let selected_id = chain.selected_lineup_id?;
    chain
        .versions
        .iter()
        .find(|lineup| lineup.id == selected_id)
}

fn append_unavailable_lineup_readiness_checks(
    checks: &mut Vec<PredictionReadinessCheck>,
    reason: &str,
) {
    for (code, label) in [("home_lineup", "主队阵容"), ("away_lineup", "客队阵容")] {
        checks.push(readiness_check(
            (code, label),
            PredictionReadinessCheckStatus::Blocked,
            15,
            0,
            "赛前数据窗口不可用，尚未选择有效阵容",
            vec![reason.to_string()],
            Value::Null,
        ));
    }
    checks.push(readiness_check(
        ("starting_goalkeepers", "首发门将"),
        PredictionReadinessCheckStatus::Blocked,
        10,
        0,
        "阵容不可用，无法确认双方首发门将",
        vec![reason.to_string()],
        Value::Null,
    ));
    checks.push(readiness_check(
        ("starter_context", "首发位置、角色与状态"),
        PredictionReadinessCheckStatus::Blocked,
        10,
        0,
        "阵容不可用，无法核对首发位置、角色与状态",
        vec![reason.to_string()],
        Value::Null,
    ));
}

fn append_lineup_readiness_checks(
    checks: &mut Vec<PredictionReadinessCheck>,
    chain: &MatchLineupChain,
) {
    for (code, label, side) in [
        ("home_lineup", "主队阵容", &chain.home),
        ("away_lineup", "客队阵容", &chain.away),
    ] {
        let Some(lineup) = selected_lineup(side) else {
            checks.push(readiness_check(
                (code, label),
                PredictionReadinessCheckStatus::Blocked,
                15,
                0,
                "当前时间窗口没有可进入模型的阵容",
                side.blocking_issues.clone(),
                json!({"team_id": side.team_id, "team_name": side.team_name}),
            ));
            continue;
        };
        let mut details = lineup.validation_warnings.clone();
        let status = if lineup.lineup_type.as_str() == "confirmed" && details.is_empty() {
            PredictionReadinessCheckStatus::Passed
        } else {
            if lineup.lineup_type.as_str() == "expected" {
                details.push("当前使用预计阵容，正式首发尚未确认".to_string());
            }
            PredictionReadinessCheckStatus::Warning
        };
        checks.push(readiness_check(
            (code, label),
            status,
            15,
            if status == PredictionReadinessCheckStatus::Passed {
                15
            } else {
                12
            },
            if status == PredictionReadinessCheckStatus::Passed {
                "确认阵容完整且通过模型资格校验"
            } else {
                "阵容可用，但仍有需要人工关注的信息"
            },
            details,
            json!({
                "team_id": side.team_id,
                "team_name": side.team_name,
                "lineup_id": lineup.id,
                "lineup_type": lineup.lineup_type.as_str(),
                "captured_at": lineup.captured_at,
                "formation_id": lineup.formation_id,
                "coach_id": lineup.coach_id,
                "player_count": lineup.player_count,
                "starter_count": lineup.starter_count,
                "quality_score": lineup.quality_score,
            }),
        ));
    }

    let selected = [selected_lineup(&chain.home), selected_lineup(&chain.away)];
    let mut goalkeeper_details = Vec::new();
    let mut missing_position_details = Vec::new();
    let mut missing_role_count = 0_usize;
    let mut inherited_role_count = 0_usize;
    let mut overridden_role_count = 0_usize;
    let mut missing_availability_count = 0_usize;
    let mut uncertain_availability_count = 0_usize;
    let mut unavailable_starter_details = Vec::new();
    for (team_name, lineup) in [
        (chain.home.team_name.as_str(), selected[0]),
        (chain.away.team_name.as_str(), selected[1]),
    ] {
        let Some(lineup) = lineup else {
            goalkeeper_details.push(format!("{team_name}尚未选定有效阵容"));
            continue;
        };
        let starters = lineup
            .players
            .iter()
            .filter(|player| player.is_starter)
            .collect::<Vec<_>>();
        let goalkeeper_count = starters
            .iter()
            .filter(|player| {
                player
                    .position_code
                    .as_deref()
                    .is_some_and(|code| code.eq_ignore_ascii_case("GK"))
            })
            .count();
        if goalkeeper_count != 1 {
            goalkeeper_details.push(format!(
                "{team_name}首发必须且只能包含 1 名门将，当前识别为 {goalkeeper_count} 名"
            ));
        }
        for player in starters {
            if player
                .position_code
                .as_deref()
                .is_none_or(|value| value.trim().is_empty())
            {
                missing_position_details.push(format!(
                    "{team_name}首发 {} 未填写实际位置",
                    player.player_name
                ));
            }
            if player
                .role_code
                .as_deref()
                .is_none_or(|value| value.trim().is_empty())
            {
                missing_role_count += 1;
            } else if player.role_origin == "player_position_default" {
                inherited_role_count += 1;
            } else if player.role_origin == "lineup_override" {
                overridden_role_count += 1;
            }
            match player.availability_status {
                None | Some(football_domain::AvailabilityStatus::Unknown) => {
                    missing_availability_count += 1;
                }
                Some(football_domain::AvailabilityStatus::Doubtful)
                | Some(football_domain::AvailabilityStatus::Returning) => {
                    uncertain_availability_count += 1;
                }
                Some(football_domain::AvailabilityStatus::Unavailable)
                | Some(football_domain::AvailabilityStatus::Injured)
                | Some(football_domain::AvailabilityStatus::Suspended)
                | Some(football_domain::AvailabilityStatus::Rested) => {
                    unavailable_starter_details.push(format!(
                        "{team_name}首发 {} 的本场状态为 {}",
                        player.player_name,
                        player
                            .availability_status
                            .map(football_domain::AvailabilityStatus::as_str)
                            .unwrap_or("unknown")
                    ));
                }
                Some(football_domain::AvailabilityStatus::Available) => {}
            }
        }
    }
    checks.push(readiness_check(
        ("starting_goalkeepers", "首发门将"),
        if goalkeeper_details.is_empty() {
            PredictionReadinessCheckStatus::Passed
        } else {
            PredictionReadinessCheckStatus::Blocked
        },
        10,
        if goalkeeper_details.is_empty() { 10 } else { 0 },
        if goalkeeper_details.is_empty() {
            "双方首发门将身份明确"
        } else {
            "首发门将身份不完整"
        },
        goalkeeper_details,
        Value::Null,
    ));

    let player_detail_status =
        if !missing_position_details.is_empty() || !unavailable_starter_details.is_empty() {
            PredictionReadinessCheckStatus::Blocked
        } else if missing_role_count > 0
            || missing_availability_count > 0
            || uncertain_availability_count > 0
        {
            PredictionReadinessCheckStatus::Warning
        } else {
            PredictionReadinessCheckStatus::Passed
        };
    let mut player_details = missing_position_details;
    player_details.extend(unavailable_starter_details);
    if missing_role_count > 0 {
        player_details.push(format!(
            "{missing_role_count} 名首发既没有本场角色覆盖，也没有可继承的球员位置默认角色"
        ));
    }
    if inherited_role_count > 0 {
        player_details.push(format!(
            "{inherited_role_count} 名首发已从球员位置档案自动继承默认战术角色"
        ));
    }
    if overridden_role_count > 0 {
        player_details.push(format!(
            "{overridden_role_count} 名首发使用本场或阵容预设角色覆盖"
        ));
    }
    if missing_availability_count > 0 {
        player_details.push(format!(
            "{missing_availability_count} 名首发缺少明确的本场可用状态快照"
        ));
    }
    if uncertain_availability_count > 0 {
        player_details.push(format!(
            "{uncertain_availability_count} 名首发处于存疑或恢复中状态"
        ));
    }
    checks.push(readiness_check(
        ("starter_context", "首发位置、角色与状态"),
        player_detail_status,
        10,
        match player_detail_status {
            PredictionReadinessCheckStatus::Passed => 10,
            PredictionReadinessCheckStatus::Warning => 7,
            PredictionReadinessCheckStatus::Blocked => 0,
        },
        match player_detail_status {
            PredictionReadinessCheckStatus::Passed => "双方首发位置、角色与可用状态完整",
            PredictionReadinessCheckStatus::Warning => "首发位置完整，但角色或可用状态仍有缺口",
            PredictionReadinessCheckStatus::Blocked => {
                "首发实际位置缺失或存在不可用球员，无法建立可靠的阵型与角色输入"
            }
        },
        player_details,
        json!({
            "missing_role_count": missing_role_count,
            "inherited_role_count": inherited_role_count,
            "overridden_role_count": overridden_role_count,
            "missing_availability_count": missing_availability_count,
            "uncertain_availability_count": uncertain_availability_count,
        }),
    ));
}

fn append_unavailable_prepared_input_checks(
    checks: &mut Vec<PredictionReadinessCheck>,
    history_reason: &str,
    input_reason: &str,
) {
    checks.push(readiness_check(
        ("team_history", "球队历史样本"),
        PredictionReadinessCheckStatus::Blocked,
        10,
        0,
        history_reason,
        Vec::new(),
        Value::Null,
    ));
    checks.push(readiness_check(
        ("model_input", "模型输入构建与质量"),
        PredictionReadinessCheckStatus::Blocked,
        5,
        0,
        "数据库事实尚不能构建确定性模型输入",
        vec![input_reason.to_string()],
        Value::Null,
    ));
}

fn append_prepared_input_checks(
    checks: &mut Vec<PredictionReadinessCheck>,
    shadow_reasons: &mut Vec<String>,
    prepared: &football_domain::PreparedMatchPredictionInput,
) {
    let home_history = nested_u64(
        &prepared.data_quality,
        &["home", "team_features", "history_match_count"],
    )
    .unwrap_or(0);
    let away_history = nested_u64(
        &prepared.data_quality,
        &["away", "team_features", "history_match_count"],
    )
    .unwrap_or(0);
    let history_status = if home_history >= 5 && away_history >= 5 {
        PredictionReadinessCheckStatus::Passed
    } else {
        PredictionReadinessCheckStatus::Warning
    };
    let mut history_details = Vec::new();
    if home_history < 5 {
        history_details.push(format!(
            "{} 截止当前窗口只有 {home_history} 场有效历史比赛",
            prepared.match_record.home_team_name
        ));
    }
    if away_history < 5 {
        history_details.push(format!(
            "{} 截止当前窗口只有 {away_history} 场有效历史比赛",
            prepared.match_record.away_team_name
        ));
    }
    if home_history == 0 || away_history == 0 {
        shadow_reasons.push("球队历史样本存在零覆盖，当前输入只允许进入影子推演".to_string());
    }
    checks.push(readiness_check(
        ("team_history", "球队历史样本"),
        history_status,
        10,
        if history_status == PredictionReadinessCheckStatus::Passed {
            10
        } else if home_history == 0 || away_history == 0 {
            2
        } else {
            6
        },
        if history_status == PredictionReadinessCheckStatus::Passed {
            "双方均具备至少 5 场截止时点前的有效历史样本"
        } else {
            "球队历史样本不足，相关强度已按置信度回归中性"
        },
        history_details,
        json!({"home_history_match_count": home_history, "away_history_match_count": away_history}),
    ));

    let quality_score = prepared
        .match_input
        .get("feature_quality_score")
        .and_then(Value::as_f64)
        .unwrap_or(0.0)
        .clamp(0.0, 1.0);
    let quality_status = if quality_score >= 0.65 {
        PredictionReadinessCheckStatus::Passed
    } else {
        PredictionReadinessCheckStatus::Warning
    };
    let mut quality_details = Vec::new();
    if let Some(warning) = prepared
        .data_quality
        .get("warning")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
    {
        quality_details.push(warning.to_string());
    }
    if quality_score < 0.40 {
        shadow_reasons.push(format!(
            "综合特征质量 {:.0}% 低于正式推演最低门槛 40%，当前仅允许影子推演",
            quality_score * 100.0
        ));
    }
    checks.push(readiness_check(
        ("model_input", "模型输入构建与质量"),
        quality_status,
        5,
        if quality_score >= 0.65 {
            5
        } else if quality_score >= 0.40 {
            3
        } else {
            1
        },
        if quality_score >= 0.65 {
            "确定性输入已生成，综合质量达到正式标准"
        } else if quality_score >= 0.40 {
            "确定性输入已生成，但综合质量需要在结果中保留警告"
        } else {
            "确定性输入已生成，但综合质量只适合影子验证"
        },
        quality_details,
        json!({
            "feature_quality_score": quality_score,
            "preparation_version": prepared.match_input.get("preparation_version"),
            "data_quality": &prepared.data_quality,
        }),
    ));
}

fn nested_u64(value: &Value, path: &[&str]) -> Option<u64> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    current.as_u64()
}

fn build_prediction_input_manifest(
    input: &Value,
    data_quality: &Value,
    match_record: &football_domain::MatchRecord,
    snapshot_type: &str,
    route_identity: Option<&Value>,
) -> Value {
    let mut canonical_input = input.clone();
    strip_runtime_prediction_input_identity(&mut canonical_input);
    json!({
        "audit_version": PREDICTION_INPUT_AUDIT_VERSION,
        "match": {
            "database_match_id": match_record.id,
            "match_key": match_record.external_key,
            "competition_id": match_record.competition_id,
            "season_id": match_record.season_id,
            "stage_id": match_record.stage_id,
            "home_team_id": match_record.home_team_id,
            "away_team_id": match_record.away_team_id,
            "kickoff_time": match_record.kickoff_time,
        },
        "snapshot_type": snapshot_type,
        "route_identity": route_identity,
        "model_input": canonical_input,
        "data_quality": data_quality,
    })
}

fn strip_runtime_prediction_input_identity(input: &mut Value) {
    let Some(object) = input.as_object_mut() else {
        return;
    };
    object.remove("feature_snapshot_id");
    object.remove("input_audit");
    if let Some(snapshot) = object.get_mut("snapshot").and_then(Value::as_object_mut) {
        snapshot.remove("snapshot_id");
        snapshot.remove("frozen_at");
    }
    if let Some(sources) = object.get_mut("sources").and_then(Value::as_array_mut) {
        for source in sources {
            if let Some(source) = source.as_object_mut() {
                source.remove("accessed_at");
            }
        }
    }
}

fn verify_prepared_input_matches_readiness(
    prepared: &football_domain::PreparedMatchPredictionInput,
    readiness: &MatchPredictionReadiness,
) -> ApplicationResult<()> {
    let expected_sha256 = readiness.input_manifest_sha256.as_deref().ok_or_else(|| {
        ApplicationError::Validation("完整度门禁没有生成输入指纹，禁止执行推演".to_string())
    })?;
    let manifest = build_prediction_input_manifest(
        &prepared.match_input,
        &prepared.data_quality,
        &prepared.match_record,
        &prepared.snapshot_type,
        readiness.route_identity.as_ref(),
    );
    let actual_sha256 = sha256_value(&manifest)?;
    if actual_sha256 != expected_sha256 {
        return Err(ApplicationError::Validation(
            "赛前数据在完整度检查后发生变化，请重新检查完整度再执行推演".to_string(),
        ));
    }
    Ok(())
}

fn attach_prediction_input_audit(
    input: &mut Value,
    readiness: &MatchPredictionReadiness,
) -> ApplicationResult<()> {
    let manifest = readiness.input_manifest.clone().ok_or_else(|| {
        ApplicationError::Validation("完整度门禁没有生成输入清单，禁止执行推演".to_string())
    })?;
    let manifest_sha256 = readiness.input_manifest_sha256.clone().ok_or_else(|| {
        ApplicationError::Validation("完整度门禁没有生成输入指纹，禁止执行推演".to_string())
    })?;
    let object = input
        .as_object_mut()
        .ok_or_else(|| ApplicationError::Model("模型输入必须是 JSON 对象".to_string()))?;
    object.insert(
        "input_audit".to_string(),
        json!({
            "audit_version": readiness.audit_version,
            "assessed_at": readiness.assessed_at,
            "readiness": {
                "level": readiness.level.as_str(),
                "score": readiness.score,
                "can_run_formal": readiness.can_run_formal,
                "can_run_shadow": readiness.can_run_shadow,
                "blockers": readiness.blockers,
                "warnings": readiness.warnings,
                "checks": readiness.checks,
            },
            "manifest": manifest,
            "manifest_sha256": manifest_sha256,
        }),
    );
    Ok(())
}

fn prediction_input_audit_summary(
    input: &Value,
    input_sha256: &str,
) -> ApplicationResult<Option<PredictionInputAuditSummary>> {
    let Some(audit) = input.get("input_audit") else {
        return Ok(None);
    };
    let audit_version = audit
        .get("audit_version")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| ApplicationError::Model("input_audit.audit_version 不能为空".to_string()))?;
    let manifest = audit
        .get("manifest")
        .ok_or_else(|| ApplicationError::Model("input_audit.manifest 不能为空".to_string()))?;
    let calculated_manifest_sha256 = sha256_value(manifest)?;
    let manifest_sha256 = audit
        .get("manifest_sha256")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            ApplicationError::Model("input_audit.manifest_sha256 不能为空".to_string())
        })?;
    if calculated_manifest_sha256 != manifest_sha256 {
        return Err(ApplicationError::Model(
            "赛前输入清单 SHA256 与实际清单不一致".to_string(),
        ));
    }
    Ok(Some(PredictionInputAuditSummary {
        audit_version: audit_version.to_string(),
        readiness_level: audit
            .get("readiness")
            .and_then(|value| value.get("level"))
            .and_then(Value::as_str)
            .unwrap_or("not_assessed")
            .to_string(),
        readiness_score: audit
            .get("readiness")
            .and_then(|value| value.get("score"))
            .and_then(Value::as_u64)
            .and_then(|value| u8::try_from(value).ok()),
        input_manifest_sha256: manifest_sha256.to_string(),
        input_sha256: input_sha256.to_string(),
    }))
}

fn sha256_value(value: &Value) -> ApplicationResult<String> {
    let bytes = serde_json::to_vec(value)?;
    Ok(hex::encode(Sha256::digest(bytes)))
}

#[derive(Debug, Clone)]
struct ModelSelection {
    family: &'static str,
    exact_model_id: Option<String>,
}

fn normalize_model_selection(value: &str) -> ApplicationResult<ModelSelection> {
    let normalized = value.trim().to_ascii_lowercase();
    let normalized = if normalized.is_empty() {
        "p4".to_string()
    } else {
        normalized
    };
    let family = if normalized == "p4" || normalized.starts_with("p4_") {
        "p4"
    } else if normalized == "p7" || normalized.starts_with("p7_") {
        "p7"
    } else {
        return Err(ApplicationError::Validation(format!(
            "不支持的模型：{normalized}；请选择已注册的 P4 或 P7 模型"
        )));
    };
    let exact_model_id = if normalized == family {
        None
    } else {
        Some(normalized)
    };
    Ok(ModelSelection {
        family,
        exact_model_id,
    })
}

fn ensure_model_selection_registered(
    registry: &crate::ModelRegistry,
    selection: &ModelSelection,
) -> ApplicationResult<()> {
    if let Some(model_id) = selection.exact_model_id.as_deref() {
        if registry.get(model_id).is_none() {
            return Err(ApplicationError::ModelNotFound(model_id.to_string()));
        }
    }
    Ok(())
}

fn parse_kickoff(raw: &str) -> ApplicationResult<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(raw)
        .map_err(|error| ApplicationError::InvalidKickoff(error.to_string()))
        .map(|value| value.with_timezone(&Utc))
}

fn required_string(value: &Value, key: &str) -> ApplicationResult<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .filter(|item| !item.trim().is_empty())
        .ok_or_else(|| ApplicationError::Model(format!("缺少字符串字段：{key}")))
}

fn nested_required_string(value: &Value, parent: &str, key: &str) -> ApplicationResult<String> {
    value
        .get(parent)
        .and_then(|parent_value| parent_value.get(key))
        .and_then(Value::as_str)
        .map(str::to_string)
        .filter(|item| !item.trim().is_empty())
        .ok_or_else(|| ApplicationError::Model(format!("缺少字符串字段：{parent}.{key}")))
}

#[cfg(test)]
mod tests {
    use super::*;

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
        ensure_model_selection_registered(&service.registry, &registered)
            .expect("已注册模型必须通过");

        let missing = normalize_model_selection("p7_not_registered").expect("格式合法");
        assert!(matches!(
            ensure_model_selection_registered(&service.registry, &missing),
            Err(ApplicationError::ModelNotFound(_))
        ));
    }

    #[test]
    fn input_manifest_ignores_runtime_snapshot_identity() {
        let match_record = football_domain::MatchRecord {
            id: Uuid::nil(),
            external_key: "MATCH-1".to_string(),
            competition_id: None,
            competition_name: None,
            season_id: None,
            stage_id: None,
            round_id: None,
            home_team_id: Uuid::from_u128(1),
            home_team_name: "Home".to_string(),
            away_team_id: Uuid::from_u128(2),
            away_team_name: "Away".to_string(),
            kickoff_time: Utc::now(),
            status: football_domain::MatchStatus::Scheduled,
            venue: None,
        };
        let route = json!({"model_id": "p4_league", "parameter_version": "p1"});
        let first = json!({
            "snapshot": {"snapshot_id": Uuid::new_v4(), "type": "T-1h", "data_cutoff_time": "2026-07-22T10:00:00Z", "frozen_at": "2026-07-22T10:01:00Z"},
            "feature_snapshot_id": Uuid::new_v4(),
            "preparation_version": "v1",
            "feature_quality_score": 0.8,
            "team_a": {"team_id": Uuid::from_u128(1), "lineup": {"lineup_id": Uuid::from_u128(3), "player_contributions": [{"player_id": Uuid::from_u128(5), "calculation_version": "v1", "effective_contribution": 71.0}]}},
            "team_b": {"team_id": Uuid::from_u128(2), "lineup": {"lineup_id": Uuid::from_u128(4), "player_contributions": []}},
            "sources": [{"source_id": "database", "accessed_at": "2026-07-22T10:01:00Z"}]
        });
        let mut second = first.clone();
        second["snapshot"]["snapshot_id"] = json!(Uuid::new_v4());
        second["snapshot"]["frozen_at"] = json!("2026-07-22T10:02:00Z");
        second["feature_snapshot_id"] = json!(Uuid::new_v4());
        second["sources"][0]["accessed_at"] = json!("2026-07-22T10:02:00Z");
        let quality = json!({"home": {}, "away": {}});
        let first_manifest =
            build_prediction_input_manifest(&first, &quality, &match_record, "T-1h", Some(&route));
        let second_manifest =
            build_prediction_input_manifest(&second, &quality, &match_record, "T-1h", Some(&route));
        assert_eq!(
            sha256_value(&first_manifest).unwrap(),
            sha256_value(&second_manifest).unwrap()
        );

        let mut changed = second;
        changed["team_a"]["lineup"]["player_contributions"][0]["effective_contribution"] =
            json!(72.0);
        let changed_manifest = build_prediction_input_manifest(
            &changed,
            &quality,
            &match_record,
            "T-1h",
            Some(&route),
        );
        assert_ne!(
            sha256_value(&first_manifest).unwrap(),
            sha256_value(&changed_manifest).unwrap()
        );
    }

    #[test]
    fn audit_summary_rejects_modified_manifest() {
        let manifest = json!({"match": "A"});
        let manifest_sha = sha256_value(&manifest).unwrap();
        let mut input = json!({
            "input_audit": {
                "audit_version": PREDICTION_INPUT_AUDIT_VERSION,
                "readiness": {"level": "formal_ready", "score": 100},
                "manifest": manifest,
                "manifest_sha256": manifest_sha
            }
        });
        assert!(prediction_input_audit_summary(&input, "input-hash")
            .unwrap()
            .is_some());
        input["input_audit"]["manifest"]["match"] = json!("B");
        assert!(prediction_input_audit_summary(&input, "input-hash").is_err());
    }
}
