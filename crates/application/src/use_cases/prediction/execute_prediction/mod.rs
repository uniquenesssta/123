use super::shared::audit::{prediction_input_audit_summary, sha256_value};
use super::shared::routing::{
    ensure_match_input_id, ensure_model_selection_registered, match_context_from_command,
    normalize_model_selection, validate_snapshot_type, verify_route_identity_matches_input_audit,
};
use super::PredictionAccess;
use crate::model_registry::ModelRegistry;
use crate::{ApplicationError, ApplicationResult, PredictionCommand, PredictionExecution};
use football_domain::{ModelIdentity, RouteRequest};
use football_model_api::ModelRequest;
use serde_json::json;
use std::time::Instant;
use uuid::Uuid;

pub(crate) async fn execute<P: PredictionAccess + ?Sized>(
    port: &P,
    registry: &ModelRegistry,
    command: PredictionCommand,
) -> ApplicationResult<PredictionExecution> {
    execute_internal(port, registry, command, true).await
}

pub(crate) async fn execute_internal<P: PredictionAccess + ?Sized>(
    port: &P,
    registry: &ModelRegistry,
    command: PredictionCommand,
    persist_run: bool,
) -> ApplicationResult<PredictionExecution> {
    let store = port;
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
    ensure_model_selection_registered(&registry, &model_selection)?;
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
    let model = registry
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
