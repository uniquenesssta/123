use super::shared::audit::{build_prediction_input_manifest, sha256_value};
use super::shared::readiness_checks::{
    append_lineup_readiness_checks, append_prepared_input_checks,
    append_unavailable_lineup_readiness_checks, append_unavailable_prepared_input_checks,
    readiness_check,
};
use super::shared::routing::{
    ensure_model_selection_registered, normalize_model_selection, route_identity_manifest,
    validate_snapshot_type,
};
use super::PredictionAccess;
use crate::model_registry::ModelRegistry;
use crate::ports::PortErrorKind;
use crate::{ApplicationError, ApplicationResult, StoredMatchPredictionCommand};
use chrono::Utc;
use football_domain::{
    CompetitionKind, MatchContext, MatchPredictionReadiness, PredictionReadinessCheck,
    PredictionReadinessCheckStatus, PredictionReadinessLevel, RouteRequest,
    PREDICTION_INPUT_AUDIT_VERSION,
};
use serde_json::{json, Value};

pub(crate) async fn execute<P: PredictionAccess + ?Sized>(
    port: &P,
    registry: &ModelRegistry,
    command: StoredMatchPredictionCommand,
) -> ApplicationResult<MatchPredictionReadiness> {
    let assessed_at = Utc::now();
    let model_selection = normalize_model_selection(&command.model_family)?;
    ensure_model_selection_registered(registry, &model_selection)?;
    let store = port;
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
        .read_match_chain_at(command.match_id, &command.snapshot_type, assessed_at)
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
        Err(error) if error.kind == PortErrorKind::InvalidState => {
            let message = error.message;
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
                let model = registry
                    .get(&decision.model_id)
                    .ok_or_else(|| ApplicationError::ModelNotFound(decision.model_id.clone()))?;
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
        Err(error) if error.kind == PortErrorKind::NotFound => checks.push(readiness_check(
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
            .prepare_match_input_at(
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
            Err(error) if error.kind == PortErrorKind::InvalidState => {
                let message = error.message;
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
