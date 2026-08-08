use super::shared::audit::{
    attach_prediction_input_audit, verify_prepared_input_matches_readiness,
};
use super::shared::routing::normalize_model_selection;
use super::PredictionAccess;
use crate::model_registry::ModelRegistry;
use crate::{
    ApplicationError, ApplicationResult, PredictionCommand, PredictionExecution,
    StoredMatchPredictionCommand,
};

pub(crate) async fn execute_formal<P: PredictionAccess + ?Sized>(
    port: &P,
    registry: &ModelRegistry,
    command: StoredMatchPredictionCommand,
) -> ApplicationResult<PredictionExecution> {
    execute_with_mode(port, registry, command, true).await
}

pub(crate) async fn execute_shadow<P: PredictionAccess + ?Sized>(
    port: &P,
    registry: &ModelRegistry,
    command: StoredMatchPredictionCommand,
) -> ApplicationResult<PredictionExecution> {
    execute_with_mode(port, registry, command, false).await
}

async fn execute_with_mode<P: PredictionAccess + ?Sized>(
    port: &P,
    registry: &ModelRegistry,
    command: StoredMatchPredictionCommand,
    persist_run: bool,
) -> ApplicationResult<PredictionExecution> {
    let readiness =
        super::inspect_match_prediction_readiness::execute(port, registry, command.clone()).await?;
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
    let store = port;
    let mut prepared = store
        .prepare_match_input_at(
            command.match_id,
            &command.snapshot_type,
            &model_family,
            readiness.assessed_at,
        )
        .await?;
    verify_prepared_input_matches_readiness(&prepared, &readiness)?;
    attach_prediction_input_audit(&mut prepared.match_input, &readiness)?;
    super::execute_prediction::execute_internal(
        port,
        registry,
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
