use super::shared::routing::{
    ensure_model_selection_registered, normalize_model_selection, parse_kickoff,
};
use super::PredictionAccess;
use crate::model_registry::ModelRegistry;
use crate::{ApplicationError, ApplicationResult, RoutePreviewCommand};
use football_domain::{RouteDecision, RouteRequest};

pub(crate) async fn execute<P: PredictionAccess + ?Sized>(
    port: &P,
    registry: &ModelRegistry,
    command: RoutePreviewCommand,
) -> ApplicationResult<RouteDecision> {
    let kickoff_time = parse_kickoff(&command.kickoff_time)?;
    let store = port;
    let scope = store
        .resolve_competition_context(
            command.competition_id,
            command.season_id,
            command.stage_id,
            command.competition_kind,
        )
        .await?;
    let model_selection = normalize_model_selection(&command.model_family)?;
    ensure_model_selection_registered(&registry, &model_selection)?;
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
